//! Process-local virtual signals. These never deliver signals to the host OS.
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualSignal {
    #[serde(rename = "SIGINT")]
    Int = 2,
    #[serde(rename = "SIGPIPE")]
    Pipe = 13,
    #[serde(rename = "SIGTERM")]
    Term = 15,
}
impl VirtualSignal {
    pub fn from_number(value: u8) -> Option<Self> {
        match value {
            2 => Some(Self::Int),
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
/// The supported signals have the default terminating disposition. Handler
/// installation, masks, stopped processes and job control are outside this model.
#[derive(Default)]
pub struct ProcessSignalState {
    pending: AtomicU8,
}
impl ProcessSignalState {
    pub fn pending(&self) -> bool {
        self.pending.load(Ordering::SeqCst) != 0
    }
    pub fn send(&self, signal: VirtualSignal) {
        let _ = self
            .pending
            .compare_exchange(0, signal as u8, Ordering::SeqCst, Ordering::SeqCst);
    }
    pub fn take(&self) -> Option<VirtualSignal> {
        VirtualSignal::from_number(self.pending.swap(0, Ordering::SeqCst))
    }
}
