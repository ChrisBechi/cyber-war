//! Out-of-band virtual stdin/cancellation. Never needs the world/service lock.
use parking_lot::{Condvar, Mutex};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, LazyLock,
    },
    time::Duration,
};

const INPUT_CAPACITY: usize = 64 * 1024;
#[derive(Default)]
pub struct Control {
    cancelled: AtomicBool,
    input: Mutex<(VecDeque<String>, usize, bool)>,
    wake: Condvar,
    output: Option<tauri::ipc::Channel<crate::cli_contract::Event>>,
    in_flight: AtomicUsize,
}
static CONTROLS: LazyLock<Mutex<BTreeMap<String, Arc<Control>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
thread_local! { static ACTIVE: RefCell<Option<Arc<Control>>> = const { RefCell::new(None) }; }
thread_local! { static CAPTURE_DEPTH: Cell<usize> = const { Cell::new(0) }; }
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
    if CAPTURE_DEPTH.with(Cell::get) > 0 {
        return;
    }
    let control = ACTIVE.with(|active| active.borrow().clone());
    let Some(control) = control else {
        return;
    };
    let Some(output) = &control.output else {
        return;
    };
    let mut start = 0;
    while start < text.len() && !control.cancelled.load(Ordering::Relaxed) {
        let mut end = (start + 4096).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let bytes = end - start;
        let mut input = control.input.lock();
        while control.in_flight.load(Ordering::Relaxed) + bytes > INPUT_CAPACITY
            && !control.cancelled.load(Ordering::Relaxed)
        {
            control.wake.wait_for(&mut input, Duration::from_millis(10));
        }
        drop(input);
        if control.cancelled.load(Ordering::Relaxed) {
            return;
        }
        control.in_flight.fetch_add(bytes, Ordering::Relaxed);
        let event = if fd == 1 {
            crate::cli_contract::Event::Stdout(text[start..end].into())
        } else {
            crate::cli_contract::Event::Stderr(text[start..end].into())
        };
        if output.send(event).is_err() {
            control.cancelled.store(true, Ordering::Relaxed);
            return;
        }
        start = end;
    }
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
    if let Some(control) = CONTROLS.lock().get(key) {
        control.cancelled.store(true, Ordering::Relaxed);
        control.wake.notify_all();
    }
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
    if input.2 {
        return false;
    }
    if let Some(text) = text {
        if input.1 + text.len() > INPUT_CAPACITY || text.contains('\0') {
            return false;
        }
        input.1 += text.len();
        input.0.push_back(text);
    } else {
        input.2 = true;
    }
    control.wake.notify_all();
    true
}
pub fn cancelled() -> bool {
    ACTIVE.with(|a| {
        a.borrow()
            .as_ref()
            .is_some_and(|c| c.cancelled.load(Ordering::Relaxed))
    })
}
pub enum Read {
    Data(String),
    Eof,
    Pending,
}
pub fn read() -> Read {
    ACTIVE.with(|a| {
        let a = a.borrow();
        let Some(control) = a.as_ref() else {
            return Read::Eof;
        };
        let mut input = control.input.lock();
        if let Some(text) = input.0.pop_front() {
            input.1 -= text.len();
            Read::Data(text)
        } else if input.2 || control.cancelled.load(Ordering::Relaxed) {
            Read::Eof
        } else {
            Read::Pending
        }
    })
}
pub fn wait() {
    ACTIVE.with(|a| {
        if let Some(control) = a.borrow().as_ref() {
            let mut input = control.input.lock();
            if input.0.is_empty() && !input.2 && !control.cancelled.load(Ordering::Relaxed) {
                control.wake.wait_for(&mut input, Duration::from_millis(10));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
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
