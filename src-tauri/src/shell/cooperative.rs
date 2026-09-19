//! Preserve the synchronous virtual shell stack while yielding world ownership.
//! The coordinator commits one quantum under the service transaction, then
//! releases its mutex before parking. The next quantum gets the latest world.
use super::{control::Registration, wait::WaitSet};
use crate::{
    error::GameResult,
    vfs::domain,
    world::{TerminalSession, WorldState},
};
use std::{
    cell::RefCell,
    sync::mpsc::{channel, Receiver, Sender},
};

struct Packet {
    world: WorldState,
    waiting: bool,
}
struct Port {
    publish: Sender<Packet>,
    resume: Receiver<Option<WorldState>>,
}
#[derive(Clone)]
struct SessionContext {
    id: String,
    default: TerminalSession,
}
thread_local! {
    static PORT: RefCell<Option<Port>> = const { RefCell::new(None) };
    static SESSION: RefCell<Option<SessionContext>> = const { RefCell::new(None) };
}

pub struct SessionScope {
    previous: Option<SessionContext>,
    active: bool,
}
impl SessionScope {
    pub fn enter(id: &str, default: &TerminalSession) -> Self {
        Self {
            previous: SESSION.with(|s| {
                s.replace(Some(SessionContext {
                    id: id.into(),
                    default: default.clone(),
                }))
            }),
            active: true,
        }
    }
    pub fn finish(mut self, fallback: TerminalSession) -> TerminalSession {
        self.active = false;
        SESSION
            .with(|s| s.replace(self.previous.take()))
            .map_or(fallback, |s| s.default)
    }
}
impl Drop for SessionScope {
    fn drop(&mut self) {
        if self.active {
            SESSION.with(|s| *s.borrow_mut() = self.previous.take());
        }
    }
}

pub fn checkpoint(world: &mut WorldState, _condition: WaitSet) -> bool {
    PORT.with(|slot| {
        let port = slot.borrow();
        let Some(port) = port.as_ref() else {
            return false;
        };
        let current = world.terminal.clone();
        let mut snapshot = world.clone();
        SESSION.with(|s| {
            if let Some(context) = s.borrow().as_ref() {
                snapshot
                    .terminal_sessions
                    .insert(context.id.clone(), current.clone());
                snapshot.terminal = context.default.clone();
            }
        });
        if port
            .publish
            .send(Packet {
                world: snapshot,
                waiting: true,
            })
            .is_err()
        {
            return true;
        }
        if let Ok(Some(mut resumed)) = port.resume.recv() {
            SESSION.with(|s| {
                if let Some(context) = s.borrow_mut().as_mut() {
                    context.default = resumed.terminal.clone();
                    resumed.terminal_sessions.remove(&context.id);
                }
            });
            resumed.terminal = current;
            *world = resumed;
        }
        true
    })
}

/// `transact` must invoke its callback once against an exclusively owned world,
/// publish the result atomically, and return with that ownership released.
/// DEV drivers can use the same handoff without a database or application UI.
pub fn drive<T: Send>(
    registration: &Registration,
    work: impl FnOnce(&mut WorldState) -> T + Send,
    mut transact: impl FnMut(&mut dyn FnMut(&mut WorldState) -> GameResult<bool>) -> GameResult<bool>,
    mut cleanup: impl FnMut(&WorldState, &WorldState),
) -> GameResult<T> {
    std::thread::scope(|scope| {
        let (resume, resumed) = channel();
        let (publish, updates) = channel::<Packet>();
        let (finish, finished) = channel();
        let worker = scope.spawn(move || {
            let Ok(Some(mut world)) = resumed.recv() else {
                return;
            };
            PORT.with(|p| {
                *p.borrow_mut() = Some(Port {
                    publish,
                    resume: resumed,
                })
            });
            let result = super::control::run(registration, || work(&mut world));
            let port = PORT.with(|p| p.take()).expect("cooperative port");
            let _ = port.publish.send(Packet {
                world,
                waiting: false,
            });
            let _ = finish.send(result);
        });
        let result = loop {
            let mut complete = false;
            let mut baseline = None;
            let mut latest = None;
            let outcome = transact(&mut |world| {
                baseline = Some(world.clone());
                resume
                    .send(Some(world.clone()))
                    .map_err(|_| domain("virtual worker disconnected"))?;
                let update = updates
                    .recv()
                    .map_err(|_| domain("virtual worker disconnected"))?;
                latest = Some(update.world.clone());
                *world = update.world;
                complete = !update.waiting;
                Ok(complete)
            });
            match outcome {
                Ok(true) => {
                    break finished
                        .recv()
                        .map_err(|_| domain("virtual worker disconnected"))
                }
                Ok(false) => registration.park(),
                Err(error) => {
                    registration.cancel();
                    if !complete && resume.send(None).is_ok() {
                        while let Ok(update) = updates.recv() {
                            latest = Some(update.world);
                            if !update.waiting {
                                break;
                            }
                            if resume.send(None).is_err() {
                                break;
                            }
                        }
                    }
                    if let (Some(before), Some(after)) = (&baseline, &latest) {
                        cleanup(before, after);
                    }
                    break Err(error);
                }
            }
        };
        drop(resume);
        worker.join().map_err(|_| domain("virtual worker failed"))?;
        result
    })
}
