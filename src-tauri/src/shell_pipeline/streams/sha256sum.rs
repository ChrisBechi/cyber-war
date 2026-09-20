//! Check-list and data descriptors stay distinct; stdin is never rewound.
use super::*;
use crate::coreutils::sha256sum::{
    diagnostic, diagnostic_bytes, emit, result, Checksum, Line, Lines, Summary,
};
use sha2::{Digest, Sha256};

fn open(world: &mut WorldState, name: &str) -> GameResult<u64> {
    let path = normalize(name, &world.terminal.cwd)?;
    let actor = world.terminal.user.clone();
    let fs = world.fs()?;
    let node = fs.stat(&path, &actor)?;
    if node.kind == "directory" && !fs.allowed(node, &actor, 4) {
        return Err(crate::vfs::domain("Permission denied"));
    }
    world.fs_mut()?.open(
        &path,
        crate::vfs::OpenFlags {
            read: true,
            ..Default::default()
        },
        0,
        &actor,
    )
}
fn close(world: &mut WorldState, handle: &mut Option<u64>) -> GameResult<()> {
    if let Some(h) = handle.take() {
        world.fs_mut()?.close(h)?;
    }
    Ok(())
}
fn data_error(
    sum: &mut Checksum,
    pending: &mut VecDeque<(u8, Vec<u8>)>,
    status: &mut i32,
    name: &[u8],
    reason: impl std::fmt::Display,
) {
    let reason = crate::coreutils::wc::reason(reason);
    if sum.opts.check && sum.opts.ignore_missing && reason == "No such file or directory" {
        return;
    }
    chunks(pending, 2, diagnostic_bytes(name, reason));
    if sum.opts.check {
        sum.summary.failed += 1;
        if !sum.opts.status {
            chunks(pending, 1, result(name, "FAILED open or read"));
        }
    } else {
        *status = 1;
    }
}
pub(super) fn step(
    world: &mut WorldState,
    process: &mut Process,
    pipes: &mut [Pipe],
) -> GameResult<bool> {
    let Engine::Sha256sum(sum) = &mut process.engine else {
        unreachable!()
    };
    if sum.current.is_none() {
        let next = if sum.opts.check {
            if sum.list.is_none() {
                let Some(name) = sum.files.pop_front() else {
                    process.input.close(pipes);
                    process.engine = Engine::Finished;
                    return Ok(true);
                };
                sum.list_directory = false;
                if name != "-" {
                    match open(world, &name) {
                        Ok(h) => sum.list_handle = Some(h),
                        Err(e) if crate::terminal_io::error_reason(&e) == "Is a directory" => {
                            sum.list_directory = true
                        }
                        Err(e) => {
                            chunks(&mut process.pending, 2, diagnostic(&name, e));
                            process.status = 1;
                            return Ok(true);
                        }
                    }
                }
                sum.list = Some(name);
                sum.lines = Lines::default();
                sum.summary = Summary::default();
            }
            if let Some(line) = sum.lines.next() {
                sum.summary.line += 1;
                let parsed = line
                    .as_ref()
                    .map_or(Line::Malformed, |bytes| sum.parser.parse(bytes));
                match parsed {
                    Line::Ignore => return Ok(true),
                    Line::Record(record)
                        if !(sum.list.as_deref() == Some("-") && record.name == b"-") =>
                    {
                        sum.summary.valid = true;
                        sum.expected = Some(record.digest);
                        record.name
                    }
                    _ => {
                        sum.summary.malformed += 1;
                        if sum.opts.warn {
                            chunks(
                                &mut process.pending,
                                2,
                                diagnostic(
                                    sum.list_name(),
                                    format!(
                                        "{}: improperly formatted SHA256 checksum line",
                                        sum.summary.line
                                    ),
                                ),
                            );
                        }
                        return Ok(true);
                    }
                }
            } else if sum.lines.eof {
                let (ok, err) = sum.summary.finish(sum.list_name(), &sum.opts);
                if !ok {
                    process.status = 1;
                }
                chunks(&mut process.pending, 2, err);
                close(world, &mut sum.list_handle)?;
                sum.list = None;
                return Ok(true);
            } else {
                let read = if sum.list_directory {
                    Err(crate::vfs::domain("Is a directory"))
                } else if let Some(h) = sum.list_handle {
                    Input::File(h).read(world, pipes)
                } else {
                    process.input.read(world, pipes)
                };
                match read {
                    Ok(Read::Data(bytes)) => sum.lines.block.extend(bytes),
                    Ok(Read::Eof) => sum.lines.eof = true,
                    Ok(Read::Pending | Read::Interrupted) => return Ok(false),
                    Err(_) => {
                        chunks(
                            &mut process.pending,
                            2,
                            diagnostic(sum.list_name(), "read error"),
                        );
                        process.status = 1;
                        close(world, &mut sum.list_handle)?;
                        sum.list = None;
                    }
                }
                return Ok(true);
            }
        } else {
            let Some(name) = sum.files.pop_front() else {
                process.input.close(pipes);
                process.engine = Engine::Finished;
                return Ok(true);
            };
            name.into_bytes()
        };
        sum.directory = false;
        if next != b"-" {
            let opened = std::str::from_utf8(&next)
                .map_err(|_| crate::vfs::domain("No such file or directory"))
                .and_then(|name| open(world, name));
            match opened {
                Ok(h) => sum.handle = Some(h),
                Err(e) if crate::terminal_io::error_reason(&e) == "Is a directory" => {
                    sum.directory = true
                }
                Err(e) => {
                    data_error(sum, &mut process.pending, &mut process.status, &next, e);
                    return Ok(true);
                }
            }
        }
        sum.hash = Sha256::new();
        sum.current = Some(next);
    }
    let read = if sum.directory {
        Err(crate::vfs::domain("Is a directory"))
    } else if let Some(h) = sum.handle {
        Input::File(h).read(world, pipes)
    } else {
        process.input.read(world, pipes)
    };
    match read {
        Ok(Read::Data(bytes)) => sum.hash.update(bytes),
        Ok(Read::Pending | Read::Interrupted) => return Ok(false),
        Ok(Read::Eof) => {
            let name = sum.current.take().unwrap();
            let hash = sum.hash.finalize_reset();
            if sum.opts.check {
                let matched = hash[..] == sum.expected.take().unwrap();
                if matched {
                    sum.summary.matched = true;
                } else {
                    sum.summary.mismatched += 1;
                }
                if !sum.opts.status && (!matched || !sum.opts.quiet) {
                    chunks(
                        &mut process.pending,
                        1,
                        result(&name, if matched { "OK" } else { "FAILED" }),
                    );
                }
            } else {
                chunks(&mut process.pending, 1, emit(&name, &hash, &sum.opts));
            }
            close(world, &mut sum.handle)?;
        }
        Err(e) => {
            let name = sum.current.take().unwrap();
            data_error(sum, &mut process.pending, &mut process.status, &name, e);
            close(world, &mut sum.handle)?;
        }
    }
    Ok(true)
}
