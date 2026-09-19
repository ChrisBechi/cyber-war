//! wc's operand/filename-list state machine on the shared scheduler inputs.
use super::*;
use crate::coreutils::wc::{diagnostic, Names, Total, Wc};
use crate::vfs::OpenFlags;

fn open(world: &mut WorldState, file: &str) -> GameResult<u64> {
    let actor = world.terminal.user.clone();
    let path = normalize(file, &world.terminal.cwd)?;
    let fs = world.fs()?;
    let node = fs.stat(&path, &actor)?;
    if node.kind == "directory" && !fs.allowed(node, &actor, 4) {
        return Err(crate::vfs::domain("Permission denied"));
    }
    world.fs_mut()?.open(
        &path,
        OpenFlags {
            read: true,
            ..Default::default()
        },
        0,
        &actor,
    )
}
fn stat_width(world: &WorldState, input: &Input, wc: &mut Wc, name: &str) {
    if name.is_empty() {
        return;
    }
    let metadata = if name == "-" {
        match input {
            Input::File(handle) => world
                .fs()
                .and_then(|fs| fs.handle_identity(*handle))
                .map(|(_, _, size, regular)| (size, regular)),
            _ => Ok((0, false)),
        }
    } else {
        normalize(name, &world.terminal.cwd).and_then(|p| {
            world
                .fs()?
                .stat(&p, &world.terminal.user)
                .map(|n| (n.logical_size(), n.kind == "file"))
        })
    };
    if let Ok((size, regular)) = metadata {
        if regular {
            wc.regular_total = wc.regular_total.saturating_add(size);
        } else {
            wc.minimum_width = 7;
        }
    }
}
fn width(wc: &mut Wc, count: u64) {
    wc.width = if wc.total == Total::Only || count == 0 || (count == 1 && wc.selected.len() == 1) {
        1
    } else {
        wc.regular_total.to_string().len().max(wc.minimum_width)
    };
}

pub(super) fn step(
    world: &mut WorldState,
    process: &mut Process,
    pipes: &mut [Pipe],
) -> GameResult<bool> {
    let Engine::Wc(wc) = &mut process.engine else {
        unreachable!()
    };
    if !wc.initialized {
        wc.initialized = true;
        if let Some(list) = wc.list.clone() {
            if list != "-" {
                match open(world, &list) {
                    Ok(handle) => wc.list_handle = Some(handle),
                    Err(error) if crate::terminal_io::error_reason(&error) == "Is a directory" => {
                        wc.list_directory = true
                    }
                    Err(error) => {
                        process.status = 1;
                        let reason = crate::coreutils::wc::reason(error);
                        let quoted = crate::coreutils::wc::quote_always(&list);
                        chunks(
                            &mut process.pending,
                            2,
                            format!("wc: cannot open {quoted} for reading: {reason}\n")
                                .into_bytes(),
                        );
                        process.input.close(pipes);
                        process.engine = Engine::Finished;
                        return Ok(true);
                    }
                }
            }
            let handle = wc.list_handle.or(match process.input {
                Input::File(h) => Some(h),
                _ => None,
            });
            if let Some(h) = handle {
                let (_, offset, size, regular) = world.fs()?.handle_identity(h)?;
                wc.scan = regular && size <= 10 * 1024 * 1024;
                wc.list_offset = offset;
            }
        } else {
            for file in wc.files.clone() {
                stat_width(world, &process.input, wc, &file);
            }
            width(wc, wc.files.len() as u64);
        }
    }
    if wc.current.is_none() {
        let file = if wc.list.is_some() {
            match wc.names.next() {
                Ok(Some(file)) if wc.scan => {
                    wc.scan_count += 1;
                    stat_width(world, &process.input, wc, &file);
                    return Ok(true);
                }
                Ok(Some(file)) => Some(file),
                Ok(None) if wc.names.eof && wc.scan => {
                    width(wc, wc.scan_count);
                    let handle = wc
                        .list_handle
                        .or(match process.input {
                            Input::File(h) => Some(h),
                            _ => None,
                        })
                        .expect("regular filename source");
                    world.fs_mut()?.seek(handle, wc.list_offset)?;
                    wc.scan = false;
                    wc.names = Names::default();
                    return Ok(true);
                }
                Ok(None) if wc.names.eof => None,
                Ok(None) => {
                    let read = if wc.list_directory {
                        Err(crate::vfs::domain("Is a directory"))
                    } else if let Some(h) = wc.list_handle {
                        Input::File(h).read(world, pipes)
                    } else {
                        process.input.read(world, pipes)
                    };
                    match read {
                        Ok(Read::Data(data)) => wc.names.block.extend(data),
                        Ok(Read::Eof) => wc.names.eof = true,
                        Ok(Read::Pending | Read::Interrupted) => return Ok(false),
                        Err(error) => {
                            let list = wc.list.as_deref().unwrap();
                            chunks(
                                &mut process.pending,
                                2,
                                diagnostic(
                                    list,
                                    format!(
                                        "read error: {}",
                                        crate::terminal_io::error_reason(error)
                                    ),
                                ),
                            );
                            process.status = 1;
                            wc.names.eof = true;
                            wc.scan = false;
                        }
                    }
                    return Ok(true);
                }
                Err(reason) => {
                    chunks(
                        &mut process.pending,
                        2,
                        diagnostic(wc.list.as_deref().unwrap(), reason),
                    );
                    process.status = 1;
                    close_operand(world, &mut process.engine)?;
                    process.input.close(pipes);
                    process.engine = Engine::Finished;
                    return Ok(true);
                }
            }
        } else {
            wc.files.pop_front()
        };
        let Some(file) = file else {
            let (out, err) = wc.finish();
            if !err.is_empty() {
                process.status = 1;
            }
            chunks(&mut process.pending, 1, out);
            chunks(&mut process.pending, 2, err);
            close_operand(world, &mut process.engine)?;
            process.input.close(pipes);
            process.engine = Engine::Finished;
            return Ok(true);
        };
        wc.count += 1;
        if file.is_empty() || (file == "-" && wc.list.as_deref() == Some("-")) {
            let text = if file.is_empty() {
                if let Some(list) = &wc.list {
                    format!(
                        "wc: {}:{}: invalid zero-length file name\n",
                        crate::coreutils::wc::quote_path(list),
                        wc.count
                    )
                } else {
                    "wc: invalid zero-length file name\n".into()
                }
            } else {
                "wc: when reading file names from stdin, no file name of '-' allowed\n".into()
            };
            chunks(&mut process.pending, 2, text.into_bytes());
            process.status = 1;
            return Ok(true);
        }
        wc.directory = false;
        if file != "-" {
            match open(world, &file) {
                Ok(handle) => wc.handle = Some(handle),
                Err(error) if crate::terminal_io::error_reason(&error) == "Is a directory" => {
                    wc.directory = true
                }
                Err(error) => {
                    process.status = 1;
                    chunks(&mut process.pending, 2, diagnostic(&file, error));
                    return Ok(true);
                }
            }
        }
        wc.reset_counter();
        wc.current = Some(file);
    }
    let read = if wc.directory {
        Err(crate::vfs::domain("Is a directory"))
    } else if let Some(h) = wc.handle {
        Input::File(h).read(world, pipes)
    } else {
        process.input.read(world, pipes)
    };
    match read {
        Ok(Read::Data(data)) => wc.counter.push(&data),
        Ok(Read::Pending | Read::Interrupted) => return Ok(false),
        ending => {
            let error = ending
                .err()
                .map(|e| diagnostic(wc.current.as_deref().unwrap_or("standard input"), e));
            if let Some(h) = wc.handle.take() {
                world.fs_mut()?.close(h)?;
            }
            chunks(&mut process.pending, 1, wc.finish_file());
            if let Some(err) = error {
                process.status = 1;
                chunks(&mut process.pending, 2, err);
            }
        }
    }
    Ok(true)
}
