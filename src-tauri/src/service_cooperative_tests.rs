use super::*;
use crate::{
    cli_contract::Event,
    shell::{
        control,
        wait::{VfsWait, WaitSet},
    },
    terminal_sessions,
    vfs::{OpenFlags, WatchTarget},
};
use parking_lot::Mutex;
use std::sync::mpsc::{channel, Receiver};

fn setup() -> Mutex<GameService> {
    let mut game = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
    game.new_game(1, "kali", "pc", false).unwrap();
    game.mutate(
        |_, w, _| {
            terminal_sessions::open(w, "A")?;
            terminal_sessions::open(w, "B")?;
            w.vfs.write("/home/kali/log", "first", "kali")?;
            Ok(())
        },
        false,
    )
    .unwrap();
    Mutex::new(game)
}
fn channel_control(key: &str) -> (control::Registration, Receiver<Event>) {
    let (tx, rx) = channel();
    let output = tauri::ipc::Channel::new(move |body| {
        let _ = tx.send(body.deserialize::<Event>().unwrap());
        Ok(())
    });
    (control::register_output(key, Some(output)), rx)
}
fn barrier(rx: &Receiver<Event>) {
    assert_eq!(
        rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap(),
        Event::WaitingForChange
    );
}
fn watch_once(w: &mut WorldState) -> GameResult<String> {
    let epoch = control::event_epoch();
    let pid = crate::shell::next_pid();
    control::register_process(pid);
    let watch = w
        .vfs
        .subscribe(WatchTarget::Path("/home/kali/log".into()))?;
    let handle = w.vfs.open(
        "/home/kali/log",
        OpenFlags {
            read: true,
            ..Default::default()
        },
        0,
        "kali",
    )?;
    let timer = w.scheduler.timer(pid, 1000)?;
    let wait = WaitSet {
        vfs: vec![VfsWait {
            host: None,
            watch,
            revision: w.vfs.watch_revision(watch).unwrap(),
        }],
        timers: vec![timer],
        ..Default::default()
    };
    w.scheduler.waiting.insert(pid, wait.clone());
    control::wait_world(w, wait, epoch);
    let data = w.vfs.read("/home/kali/log", "kali")?.to_owned();
    w.vfs.close(handle)?;
    w.vfs.unsubscribe(watch);
    w.scheduler.finish(pid);
    control::unregister_process(pid);
    Ok(data)
}

#[test]
fn suspended_consumer_releases_service_for_writer_and_preserves_both_sessions() {
    let game = setup();
    let (registration, rx) = channel_control("cooperative-writer");
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            GameService::transact_cooperatively(&game, &registration, |w| {
                terminal_sessions::with_session(w, Some("A"), |w| {
                    w.terminal.env.insert("LOCAL".into(), "reader".into());
                    let value = watch_once(w)?;
                    assert!(w.flags.contains("other-action"));
                    assert_eq!(w.terminal.env["LOCAL"], "reader");
                    Ok(value)
                })
            })
        });
        barrier(&rx);
        {
            let mut game = game.lock();
            assert_eq!(game.world().unwrap().vfs.watcher_count(), 1);
            assert_eq!(game.world().unwrap().scheduler.waiting.len(), 1);
            game.mutate(
                |_, w, _| {
                    terminal_sessions::with_session(w, Some("B"), |w| {
                        w.flags.insert("other-action".into());
                        w.terminal.env.insert("LOCAL".into(), "writer".into());
                        w.vfs.write("/home/kali/log", "second", "kali")
                    })
                },
                false,
            )
            .unwrap();
        }
        assert_eq!(worker.join().unwrap().unwrap(), "second");
    });
    let game = game.lock();
    let w = game.world().unwrap();
    assert_eq!(w.terminal_sessions["A"].env["LOCAL"], "reader");
    assert_eq!(w.terminal_sessions["B"].env["LOCAL"], "writer");
    assert!(!w.terminal.env.contains_key("LOCAL"));
    assert_eq!(w.vfs.watcher_count(), 0);
    assert_eq!(w.vfs.open_handle_count(), 0);
    assert_eq!(w.scheduler.timer_count(), 0);
    assert!(w.scheduler.waiting.is_empty());
}

#[test]
fn cancellation_save_load_and_sql_failure_do_not_leak_or_restore_old_world() {
    for mode in ["signal", "load", "sql-failure"] {
        let game = setup();
        let key = format!("cooperative-{mode}");
        let (registration, rx) = channel_control(&key);
        std::thread::scope(|scope| {
            let worker = scope.spawn(|| {
                GameService::transact_cooperatively(&game, &registration, |w| {
                    let result = watch_once(w)?;
                    w.flags.insert("finished-old-worker".into());
                    Ok(result)
                })
            });
            barrier(&rx);
            {
                let mut game = game.lock();
                game.save(true).unwrap();
                match mode {
                    "load" => {
                        game.load(1, true).unwrap();
                    }
                    "sql-failure" => {
                        game.connection.execute_batch("CREATE TRIGGER reject_runtime_save BEFORE UPDATE ON save_slots BEGIN SELECT RAISE(ABORT,'controlled failure'); END;").unwrap();
                    }
                    _ => {}
                }
                control::cancel(&key);
            }
            let result = worker.join().unwrap();
            assert_eq!(result.is_ok(), mode == "signal");
        });
        let game = game.lock();
        let w = game.world().unwrap();
        assert_eq!(w.vfs.watcher_count(), 0, "{mode}");
        assert_eq!(w.vfs.open_handle_count(), 0, "{mode}");
        assert_eq!(w.scheduler.timer_count(), 0, "{mode}");
        assert!(w.scheduler.waiting.is_empty(), "{mode}");
        assert_eq!(w.flags.contains("finished-old-worker"), mode == "signal");
    }
}

#[test]
fn tail_observes_other_terminal_writes_and_committed_kill_without_orphans() {
    let game = setup();
    let key = "actual-tail-service";
    let (registration, rx) = channel_control(key);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            GameService::transact_cooperatively(&game, &registration, |w| {
                terminal_sessions::with_session(w, Some("A"), |w| {
                    Ok(crate::terminal::execute(w, "tail -n0 -F log"))
                })
            })
        });
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            barrier(&rx);
            game.lock()
                .mutate(
                    |_, w, _| {
                        terminal_sessions::with_session(w, Some("B"), |w| {
                            let result =
                                crate::terminal::execute(w, "echo one >> log; echo two >> log");
                            assert_eq!(result.exit_code, 0);
                            Ok(())
                        })
                    },
                    false,
                )
                .unwrap();
            let text = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            assert_eq!(text, Event::Stdout("one\ntwo\n".into()));
            control::acknowledge(key, 8);
            barrier(&rx);
            game.lock()
                .mutate(
                    |_, w, _| {
                        let pid = w
                            .processes
                            .iter()
                            .find(|p| p.name == "tail" && p.running)
                            .unwrap()
                            .pid;
                        crate::terminal::signal_local_process(w, pid, "kali", 15)
                    },
                    false,
                )
                .unwrap();
        }));
        if outcome.is_err() {
            control::cancel(key);
        }
        let result = worker.join().unwrap().unwrap();
        if let Err(error) = outcome {
            std::panic::resume_unwind(error);
        }
        assert_eq!(result.exit_code, 143);
        assert_eq!(result.stdout, "one\ntwo\n");
    });
    let game = game.lock();
    let world = game.world().unwrap();
    assert_eq!(world.vfs.watcher_count(), 0);
    assert_eq!(world.vfs.open_handle_count(), 0);
    assert!(world.scheduler.waiting.is_empty());
    assert!(world.scheduler.signals.is_empty());
    assert_eq!(
        world.vfs.read("/home/kali/log", "kali").unwrap(),
        "firstone\ntwo\n"
    );
}

#[test]
fn tail_consumes_a_thousand_committed_appends_and_preserves_session_contexts() {
    let game = setup();
    let key = "actual-tail-burst";
    let (registration, rx) = channel_control(key);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            GameService::transact_cooperatively(&game, &registration, |w| {
                terminal_sessions::with_session(w, Some("A"), |w| {
                    w.terminal.env.insert("LOCAL".into(), "reader".into());
                    Ok(crate::terminal::execute(w, "tail -n0 -f log"))
                })
            })
        });
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            barrier(&rx);
            game.lock()
                .mutate(
                    |_, w, _| {
                        terminal_sessions::with_session(w, Some("B"), |w| {
                            w.terminal.env.insert("LOCAL".into(), "writer".into());
                            for _ in 0..1000 {
                                let result = crate::terminal::execute(w, "echo byte >> log");
                                assert_eq!(result.exit_code, 0);
                            }
                            assert_eq!(w.vfs.watcher_count(), 1);
                            Ok(())
                        })
                    },
                    false,
                )
                .unwrap();
            let mut output = String::new();
            loop {
                match rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap() {
                    Event::Stdout(text) => {
                        control::acknowledge(key, text.len());
                        output.push_str(&text);
                    }
                    Event::WaitingForChange => break,
                    event => panic!("unexpected burst event: {event:?}"),
                }
            }
            assert_eq!(output, "byte\n".repeat(1000));
            control::cancel(key);
        }));
        if outcome.is_err() {
            control::cancel(key);
        }
        let result = worker.join().unwrap().unwrap();
        if let Err(error) = outcome {
            std::panic::resume_unwind(error);
        }
        assert_eq!(result.stdout, "byte\n".repeat(1000));
    });
    let game = game.lock();
    let w = game.world().unwrap();
    assert_eq!(w.terminal_sessions["A"].env["LOCAL"], "reader");
    assert_eq!(w.terminal_sessions["B"].env["LOCAL"], "writer");
    assert_eq!(
        w.vfs.read("/home/kali/log", "kali").unwrap(),
        format!("first{}", "byte\n".repeat(1000))
    );
    assert_eq!(w.vfs.watcher_count(), 0);
    assert_eq!(w.vfs.open_handle_count(), 0);
    assert_eq!(w.scheduler.timer_count(), 0);
    assert!(w.scheduler.waiting.is_empty());
    assert!(w.scheduler.signals.is_empty());
}

#[test]
fn actual_tail_signals_load_and_failed_commit_release_runtime_without_replaying_changes() {
    for mode in ["int", "term", "load", "sql-failure"] {
        let game = setup();
        let key = format!("actual-tail-lifecycle-{mode}");
        let (registration, rx) = channel_control(&key);
        std::thread::scope(|scope| {
            let worker = scope.spawn(|| {
                GameService::transact_cooperatively(&game, &registration, |w| {
                    terminal_sessions::with_session(w, Some("A"), |w| {
                        Ok(crate::terminal::execute(w, "tail -n0 -F log"))
                    })
                })
            });
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                barrier(&rx);
                let mut game = game.lock();
                let identity = game.world().unwrap().runtime_id;
                game.mutate(
                    |_, w, _| {
                        w.flags.insert("committed-peer-change".into());
                        Ok(())
                    },
                    false,
                )
                .unwrap();
                game.save(true).unwrap();
                if mode == "load" {
                    // The command layer cancels registered terminals before load.
                    control::cancel(&key);
                    game.load(1, true).unwrap();
                    assert_ne!(game.world().unwrap().runtime_id, identity);
                } else {
                    if mode == "sql-failure" {
                        game.connection.execute_batch("CREATE TRIGGER reject_tail_save BEFORE UPDATE ON save_slots BEGIN SELECT RAISE(ABORT,'controlled failure'); END;").unwrap();
                    }
                    let signal = if mode == "int" {
                        crate::shell::signals::VirtualSignal::Int
                    } else {
                        crate::shell::signals::VirtualSignal::Term
                    };
                    assert!(control::signal(&key, None, signal));
                }
            }));
            if outcome.is_err() {
                control::cancel(&key);
            }
            let result = worker.join().unwrap();
            if let Err(error) = outcome {
                std::panic::resume_unwind(error);
            }
            if matches!(mode, "int" | "term") {
                assert_eq!(
                    result.unwrap().exit_code,
                    if mode == "int" { 130 } else { 143 }
                );
            } else {
                assert!(result.is_err(), "{mode}");
            }
        });
        let game = game.lock();
        let w = game.world().unwrap();
        assert!(w.flags.contains("committed-peer-change"), "{mode}");
        assert_eq!(w.vfs.read("/home/kali/log", "kali").unwrap(), "first");
        assert_eq!(w.vfs.watcher_count(), 0, "{mode}");
        assert_eq!(w.vfs.open_handle_count(), 0, "{mode}");
        assert_eq!(w.scheduler.timer_count(), 0, "{mode}");
        assert!(w.scheduler.waiting.is_empty(), "{mode}");
        assert!(w.scheduler.signals.is_empty(), "{mode}");
        assert!(
            w.processes.iter().all(|p| p.name != "tail" || !p.running),
            "{mode}"
        );
    }
}
