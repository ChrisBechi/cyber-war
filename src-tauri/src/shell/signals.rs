//! Process-local virtual signals. These never deliver signals to the host OS.
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU16, AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualSignal {
    #[serde(rename = "SIGINT")]
    Int = 2,
    #[serde(rename = "SIGKILL")]
    Kill = 9,
    #[serde(rename = "SIGPIPE")]
    Pipe = 13,
    #[serde(rename = "SIGTERM")]
    Term = 15,
}
impl VirtualSignal {
    pub fn from_number(value: u8) -> Option<Self> {
        match value {
            2 => Some(Self::Int),
            9 => Some(Self::Kill),
            13 => Some(Self::Pipe),
            15 => Some(Self::Term),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Termination {
    Exit { code: i32 },
    Signal { signal: VirtualSignal },
}
impl Termination {
    pub fn status(self) -> i32 {
        match self {
            Self::Exit { code } => code,
            Self::Signal { signal } => 128 + signal as i32,
        }
    }
}
/// Default terminating or ignored dispositions. No host signal handlers.
#[derive(Default)]
pub struct ProcessSignalState {
    pending: AtomicU8,
    ignored: AtomicU16,
}
impl ProcessSignalState {
    pub fn pending(&self) -> bool {
        self.pending.load(Ordering::SeqCst) != 0
    }
    pub fn ignore(&self, signal: VirtualSignal) {
        if signal != VirtualSignal::Kill {
            self.ignored.fetch_or(1 << signal as u8, Ordering::SeqCst);
        }
    }
    pub fn ignored(&self, signal: VirtualSignal) -> bool {
        self.ignored.load(Ordering::SeqCst) & (1 << signal as u8) != 0
    }
    pub fn send(&self, signal: VirtualSignal) {
        if self.ignored(signal) {
            return;
        }
        let _ = self
            .pending
            .compare_exchange(0, signal as u8, Ordering::SeqCst, Ordering::SeqCst);
    }
    pub fn take(&self) -> Option<VirtualSignal> {
        VirtualSignal::from_number(self.pending.swap(0, Ordering::SeqCst))
    }
}
