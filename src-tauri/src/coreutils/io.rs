//! Byte IO at the World/VFS boundary. Blob payloads are immutable and virtual.
use crate::{
    error::GameResult,
    vfs::{domain, normalize, OpenFlags},
    world::WorldState,
};

pub(crate) const LIMIT: usize = 4 * 1024 * 1024;

pub(crate) fn stdin(world: &WorldState) -> Vec<u8> {
    world.terminal.stdin_bytes.clone().unwrap_or_else(|| {
        world
            .terminal
            .stdin
            .as_deref()
            .unwrap_or("")
            .as_bytes()
            .to_vec()
    })
}
pub(crate) fn read(
    world: &mut WorldState,
    file: &str,
    actor: &str,
    input: &mut Option<Vec<u8>>,
) -> GameResult<Vec<u8>> {
    if file == "-" {
        if world.terminal.stdin.is_none()
            && world.terminal.stdin_bytes.is_none()
            && world.terminal.io.stdin_tty
        {
            return Err(domain(
                "interactive stdin requires a streaming process; this mode is unsupported",
            ));
        }
        return Ok(input.take().unwrap_or_default());
    }
    let path = normalize(file, &world.terminal.cwd)?;
    let node = world.fs()?.readable(&path, actor)?;
    if node.kind == "charDevice" && node.metadata.get("device").is_some_and(|s| s == "zero") {
        return Err(domain(
            "unbounded device input is outside the supported subset",
        ));
    }
    if node.logical_size() > LIMIT as u64 {
        return Err(domain("virtual command input limit: 4 MiB"));
    }
    let handle = world.fs_mut()?.open(
        &path,
        OpenFlags {
            read: true,
            ..Default::default()
        },
        0,
        actor,
    )?;
    let blobs = world.blobs.clone();
    let result = world.fs_mut()?.read_handle_bytes(handle, LIMIT, &blobs);
    let closed = world.fs_mut()?.close(handle);
    closed?;
    result
}
pub(crate) fn write_handle(world: &mut WorldState, handle: u64, data: &[u8]) -> GameResult<()> {
    let mut blobs = world.blobs.clone();
    world
        .fs_mut()?
        .write_handle_bytes(handle, data, &mut blobs)?;
    world.blobs = blobs;
    Ok(())
}
pub(crate) fn write(
    world: &mut WorldState,
    file: &str,
    data: &[u8],
    actor: &str,
    append: bool,
) -> GameResult<()> {
    let path = normalize(file, &world.terminal.cwd)?;
    let h = world.fs_mut()?.open(
        &path,
        OpenFlags {
            write: true,
            create: true,
            truncate: !append,
            append,
            ..Default::default()
        },
        0o666,
        actor,
    )?;
    let result = write_handle(world, h, data);
    let closed = world.fs_mut()?.close(h);
    closed?;
    result
}
pub(crate) fn send(out: &mut crate::terminal_io::Output, fd: u8, bytes: Vec<u8>) -> GameResult<()> {
    let size: usize = out.byte_ordered.iter().map(|(_, b)| b.len()).sum();
    if size.saturating_add(bytes.len()) > LIMIT {
        return Err(domain("virtual command output limit: 4 MiB"));
    }
    if fd == 1 {
        out.binary
            .get_or_insert_with(Vec::new)
            .extend_from_slice(&bytes);
    } else {
        out.stderr
            .push_str(std::str::from_utf8(&bytes).map_err(|_| domain("non-text diagnostic"))?);
    }
    out.byte_ordered.push((fd, bytes));
    Ok(())
}
