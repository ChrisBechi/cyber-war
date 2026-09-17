//! Out-of-band virtual stdin/cancellation. Never needs the world/service lock.
use parking_lot::{Condvar, Mutex};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
        Arc, LazyLock,
    },
    time::Duration,
};

const INPUT_CAPACITY: usize = 64 * 1024;
#[derive(Default)]
pub struct Control {
    cancelled: AtomicBool,
    waiting: AtomicBool,
    signal: AtomicU8,
    processes: Mutex<BTreeMap<u32, Arc<super::signals::ProcessSignalState>>>,
    input: Mutex<super::tty::VirtualTty>,
    wake: Condvar,
    output: Option<tauri::ipc::Channel<crate::cli_contract::Event>>,
    in_flight: AtomicUsize,
}
static CONTROLS: LazyLock<Mutex<BTreeMap<String, Arc<Control>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
thread_local! { static ACTIVE: RefCell<Option<Arc<Control>>> = const { RefCell::new(None) }; }
thread_local! { static CAPTURE_DEPTH: Cell<usize> = const { Cell::new(0) }; }
thread_local! { static CURRENT_PROCESS: RefCell<Option<Arc<super::signals::ProcessSignalState>>> = const { RefCell::new(None) }; }
pub fn with_process<T>(
    process: &Arc<super::signals::ProcessSignalState>,
    work: impl FnOnce() -> T,
) -> T {
    struct Restore(Option<Arc<super::signals::ProcessSignalState>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            CURRENT_PROCESS.with(|c| *c.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(CURRENT_PROCESS.with(|c| c.replace(Some(process.clone()))));
    work()
}
fn interrupted(control: &Control) -> bool {
    control.cancelled.load(Ordering::Relaxed)
        || CURRENT_PROCESS.with(|c| c.borrow().as_ref().is_some_and(|s| s.pending()))
}
pub struct Registration {
    key: String,
    control: Arc<Control>,
}
impl Drop for Registration {
    fn drop(&mut self) {
        let mut controls = CONTROLS.lock();
        if controls
            .get(&self.key)
            .is_some_and(|c| Arc::ptr_eq(c, &self.control))
        {
            controls.remove(&self.key);
        }
    }
}
#[cfg(test)]
pub fn register(key: &str) -> Registration {
    register_output(key, None)
}
pub fn register_output(
    key: &str,
    output: Option<tauri::ipc::Channel<crate::cli_contract::Event>>,
) -> Registration {
    let control = Arc::new(Control {
        output,
        ..Control::default()
    });
    if let Some(old) = CONTROLS.lock().insert(key.into(), control.clone()) {
        old.cancelled.store(true, Ordering::Relaxed);
        old.wake.notify_all();
    }
    Registration {
        key: key.into(),
        control,
    }
}
pub fn acknowledge(key: &str, bytes: usize) {
    if let Some(control) = CONTROLS.lock().get(key) {
        let _ = control
            .in_flight
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                Some(old.saturating_sub(bytes))
            });
        control.wake.notify_all();
    }
}
pub fn capture<T>(work: impl FnOnce() -> T) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            CAPTURE_DEPTH.with(|depth| depth.set(depth.get() - 1));
        }
    }
    CAPTURE_DEPTH.with(|depth| depth.set(depth.get() + 1));
    let _restore = Restore;
    work()
}
pub fn emit(fd: u8, text: &str) {
    let mut start = 0;
    while start < text.len() {
        let mut end = (start + 4096).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        if !emit_chunk(fd, &text[start..end]) {
            break;
        }
        start = end;
    }
}
/// Reserve a whole presentation chunk before publishing it. The byte stream
/// commits its corresponding raw bytes only when this delivery succeeds.
pub fn emit_chunk(fd: u8, text: &str) -> bool {
    assert!(text.len() <= INPUT_CAPACITY);
    if CAPTURE_DEPTH.with(Cell::get) > 0 {
        return true;
    }
    let control = ACTIVE.with(|active| active.borrow().clone());
    let Some(control) = control else {
        return true;
    };
    let Some(output) = &control.output else {
        return true;
    };
    let bytes = text.len();
    let mut input = control.input.lock();
    while control.in_flight.load(Ordering::Relaxed) + bytes > INPUT_CAPACITY
        && !interrupted(&control)
    {
        control.wake.wait_for(&mut input, Duration::from_millis(10));
    }
    drop(input);
    if interrupted(&control) {
        return false;
    }
    control.in_flight.fetch_add(bytes, Ordering::Relaxed);
    let event = if fd == 1 {
        crate::cli_contract::Event::Stdout(text.into())
    } else {
        crate::cli_contract::Event::Stderr(text.into())
    };
    if output.send(event).is_err() {
        control.cancelled.store(true, Ordering::Relaxed);
        return false;
    }
    true
}
pub fn run<T>(registration: &Registration, work: impl FnOnce() -> T) -> T {
    struct Restore(Option<Arc<Control>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ACTIVE.with(|a| *a.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(ACTIVE.with(|a| a.replace(Some(registration.control.clone()))));
    work()
}
pub fn cancel(key: &str) {
    signal(key, None, super::signals::VirtualSignal::Int);
}
pub fn signal(key: &str, pid: Option<u32>, signal: super::signals::VirtualSignal) -> bool {
    if let Some(control) = CONTROLS.lock().get(key) {
        let processes = control.processes.lock();
        if let Some(pid) = pid {
            let Some(process) = processes.get(&pid) else {
                return false;
            };
            process.send(signal);
        } else {
            control.signal.store(signal as u8, Ordering::SeqCst);
            control.cancelled.store(true, Ordering::Relaxed);
            for process in processes.values() {
                process.send(signal);
            }
        }
        control.wake.notify_all();
        return true;
    }
    false
}
pub fn register_process(pid: u32) -> Arc<super::signals::ProcessSignalState> {
    let state = Arc::new(super::signals::ProcessSignalState::default());
    ACTIVE.with(|a| {
        if let Some(control) = a.borrow().as_ref() {
            if let Some(signal) = termination_signal() {
                state.send(signal);
            }
            control.processes.lock().insert(pid, state.clone());
        }
    });
    state
}
pub fn unregister_process(pid: u32) {
    ACTIVE.with(|a| {
        if let Some(c) = a.borrow().as_ref() {
            c.processes.lock().remove(&pid);
        }
    });
}
pub fn termination_signal() -> Option<super::signals::VirtualSignal> {
    ACTIVE.with(|a| {
        a.borrow().as_ref().and_then(|c| {
            super::signals::VirtualSignal::from_number(c.signal.load(Ordering::SeqCst)).or_else(
                || {
                    c.cancelled
                        .load(Ordering::Relaxed)
                        .then_some(super::signals::VirtualSignal::Int)
                },
            )
        })
    })
}
pub fn cancel_all() {
    for control in CONTROLS.lock().values() {
        control.cancelled.store(true, Ordering::Relaxed);
        control.wake.notify_all();
    }
}
pub fn input(key: &str, text: Option<String>) -> bool {
    let controls = CONTROLS.lock();
    let Some(control) = controls.get(key) else {
        return false;
    };
    let mut input = control.input.lock();
    control.waiting.store(false, Ordering::Relaxed);
    if control.cancelled.load(Ordering::Relaxed) {
        return false;
    }
    let accepted = match text {
        Some(text) => input.input(text.as_bytes()),
        None => input.eof(),
    };
    control.wake.notify_all();
    accepted
}
pub fn cancelled() -> bool {
    ACTIVE.with(|a| {
        a.borrow()
            .as_ref()
            .is_some_and(|c| c.cancelled.load(Ordering::Relaxed))
    })
}
pub use super::tty::Read;
pub fn read() -> Read {
    ACTIVE.with(|a| {
        let a = a.borrow();
        let Some(control) = a.as_ref() else {
            return Read::Eof;
        };
        if control.cancelled.load(Ordering::Relaxed) {
            return Read::Interrupted;
        }
        let read = control.input.lock().read();
        read
    })
}
pub fn wait() {
    ACTIVE.with(|a| {
        if let Some(control) = a.borrow().as_ref() {
            let mut input = control.input.lock();
            if input.pending() && !control.cancelled.load(Ordering::Relaxed) {
                if !control.waiting.swap(true, Ordering::Relaxed) {
                    if let Some(output) = &control.output {
                        let _ = output.send(crate::cli_contract::Event::WaitingForInput);
                    }
                }
                control.wake.wait_for(&mut input, Duration::from_millis(10));
            }
        }
    });
}

#[cfg(test)]
pub fn process_ids(key: &str) -> Vec<u32> {
    CONTROLS
        .lock()
        .get(key)
        .map(|c| c.processes.lock().keys().copied().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn process_signal_wakes_output_backpressure_without_signalling_peer() {
        use super::super::signals::VirtualSignal;
        let (tx, rx) = std::sync::mpsc::channel();
        let output = tauri::ipc::Channel::new(move |_| {
            tx.send(()).unwrap();
            Ok(())
        });
        let registration = register_output("process-backpressure", Some(output));
        let worker = std::thread::spawn(move || {
            run(&registration, || {
                let target = register_process(71);
                let peer = register_process(72);
                with_process(&target, || emit(1, &"x".repeat(INPUT_CAPACITY * 2)));
                assert_eq!(target.take(), Some(VirtualSignal::Term));
                assert_eq!(peer.take(), None);
                assert!(!cancelled());
                unregister_process(71);
                unregister_process(72);
            })
        });
        for _ in 0..16 {
            rx.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        assert!(signal(
            "process-backpressure",
            Some(71),
            VirtualSignal::Term
        ));
        worker.join().unwrap();
    }
    #[test]
    fn xterm_backpressure_waits_for_acknowledgements_and_can_cancel() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let output = tauri::ipc::Channel::new(move |_| {
            sender.send(()).unwrap();
            Ok(())
        });
        let registration = register_output("output-cancel", Some(output));
        let worker = std::thread::spawn(move || {
            run(&registration, || emit(1, &"x".repeat(INPUT_CAPACITY * 2)))
        });
        for _ in 0..16 {
            receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        assert!(receiver.recv_timeout(Duration::from_millis(30)).is_err());
        cancel("output-cancel");
        worker.join().unwrap();
        assert!(!input("output-cancel", None));

        let (sender, receiver) = std::sync::mpsc::channel();
        let output = tauri::ipc::Channel::new(move |_| {
            sender.send(()).unwrap();
            Ok(())
        });
        let registration = register_output("output-ack", Some(output));
        let worker =
            std::thread::spawn(move || run(&registration, || emit(2, &"é".repeat(INPUT_CAPACITY))));
        for _ in 0..32 {
            receiver.recv_timeout(Duration::from_secs(5)).unwrap();
            acknowledge("output-ack", 4096);
        }
        worker.join().unwrap();
        assert!(!input("output-ack", None));
    }
}
