//! DEV-only adapter for schema-2 barriers. Uses the production scheduler handoff.
#[cfg(not(test))]
compile_error!("Follow fixture driver is development-only");
use super::*;
use crate::{
    cli_contract::Event,
    shell::{control, cooperative},
};

fn mutation(world: &mut WorldState, step: &InteractionStep) {
    let InteractionStep::Mutate {
        operation,
        path,
        target,
        hex,
        size,
        mode,
    } = step
    else {
        unreachable!()
    };
    assert!(path.starts_with("/home/kali/") && !path.split('/').any(|c| c == ".."));
    let actor = "kali";
    match operation.as_str() {
        "write" | "append" => {
            let handle = world
                .vfs
                .open(
                    path,
                    crate::vfs::OpenFlags {
                        write: true,
                        create: true,
                        append: operation == "append",
                        truncate: operation == "write",
                        ..Default::default()
                    },
                    0o666,
                    actor,
                )
                .unwrap();
            world
                .vfs
                .write_handle_bytes(handle, &unhex(hex.as_ref().unwrap()), &mut world.blobs)
                .unwrap();
            world.vfs.close(handle).unwrap();
        }
        "truncate" => world
            .vfs
            .truncate_bytes(path, size.unwrap(), actor, &mut world.blobs)
            .unwrap(),
        "rename" => world
            .vfs
            .rename_entry(path, target.as_ref().unwrap(), actor)
            .unwrap(),
        "unlink" => world.vfs.unlink(path, actor).unwrap(),
        "chmod" => world.vfs.chmod(path, actor, mode.unwrap()).unwrap(),
        "mkdir" => world.vfs.mkdir(path, actor).unwrap(),
        "hardlink" => world
            .vfs
            .link(path, target.as_ref().unwrap(), actor)
            .unwrap(),
        "symlink" => world
            .vfs
            .symlink(path, target.as_ref().unwrap(), actor)
            .unwrap(),
        _ => panic!("Unknown mutation operation"),
    }
}

pub(super) fn run(
    key: &str,
    registration: &control::Registration,
    interaction: &InteractionFixture,
    events: std::sync::mpsc::Receiver<Event>,
    observations: &mut Vec<Value>,
    world: &mut WorldState,
    work: impl FnOnce(&mut WorldState) -> terminal::CommandResult + Send,
) -> terminal::CommandResult {
    let state = parking_lot::Mutex::new(world);
    let raw = registration.observe_bytes();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            cooperative::drive(
                registration,
                work,
                |callback| {
                    let mut world = state.lock();
                    let complete = callback(&mut world)?;
                    control::notify_world(&world);
                    Ok(complete)
                },
                |before, after| state.lock().release_runtime_closed(before, after),
            )
            .unwrap()
        });
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut waiting = false;
        let receive = |stdout: &mut Vec<u8>, stderr: &mut Vec<u8>, waiting: &mut bool| match events
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("follow barrier deadline")
        {
            Event::Stdout(text) => {
                control::acknowledge(key, text.len());
                let (fd, bytes) = raw
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .expect("raw stdout");
                assert_eq!(fd, 1);
                stdout.extend(bytes);
            }
            Event::Stderr(text) => {
                control::acknowledge(key, text.len());
                let (fd, bytes) = raw
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .expect("raw stderr");
                assert_eq!(fd, 2);
                stderr.extend(bytes);
            }
            Event::WaitingForChange | Event::WaitingForInput => *waiting = true,
            _ => {}
        };
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for step in &interaction.steps {
                match step {
                    InteractionStep::Await {
                        stdout_bytes,
                        stderr_bytes,
                    } => {
                        while stdout.len() < *stdout_bytes || stderr.len() < *stderr_bytes {
                            receive(&mut stdout, &mut stderr, &mut waiting);
                        }
                    }
                    InteractionStep::Wait => {
                        while !waiting {
                            receive(&mut stdout, &mut stderr, &mut waiting);
                        }
                        // Acquiring the world is the commit barrier after the wait event.
                        drop(state.lock());
                    }
                    InteractionStep::Mutate { .. } => {
                        let mut world = state.lock();
                        mutation(&mut world, step);
                        control::notify_world(&world);
                        waiting = false;
                        continue;
                    }
                    InteractionStep::MutateBatch { steps } => {
                        let mut world = state.lock();
                        for step in steps {
                            mutation(&mut world, step);
                        }
                        control::notify_world(&world);
                        waiting = false;
                        continue;
                    }
                    InteractionStep::StopWriter { writer, signal } => {
                        let mut world = state.lock();
                        let index = interaction
                            .writers
                            .iter()
                            .position(|w| w == writer)
                            .expect("declared writer");
                        let pid = 100_000 + index as u32;
                        if let Some(signal) = signal {
                            crate::terminal::signal_local_process(
                                &mut world,
                                pid,
                                "kali",
                                *signal as u8,
                            )
                            .unwrap();
                        }
                        world.processes.retain(|p| p.pid != pid);
                        control::notify_world(&world);
                        waiting = false;
                        continue;
                    }
                    InteractionStep::Signal { signal, .. } => {
                        while !waiting {
                            receive(&mut stdout, &mut stderr, &mut waiting);
                        }
                        let pids = control::process_ids(key);
                        assert_eq!(pids.len(), 1);
                        assert!(control::signal(key, Some(pids[0]), *signal));
                        continue;
                    }
                    _ => panic!("TTY step in follow protocol"),
                }
                observations.push(json!({"stdoutHex":stdout.iter().map(|b|format!("{b:02x}")).collect::<String>(),
                    "stderrHex":stderr.iter().map(|b|format!("{b:02x}")).collect::<String>(),"running":!worker.is_finished()}));
            }
        }));
        if outcome.is_err() {
            control::cancel(key);
        }
        let result = worker.join().unwrap();
        state
            .lock()
            .processes
            .retain(|p| !(100_000..100_000 + interaction.writers.len() as u32).contains(&p.pid));
        if let Err(error) = outcome {
            std::panic::resume_unwind(error);
        }
        result
    })
}
