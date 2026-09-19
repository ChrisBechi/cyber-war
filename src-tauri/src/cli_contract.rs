//! Shared extension contract for future virtual interactive engines. No OS IO.
//! Existing CommandResult remains the completed-command IPC contract; archive
//! jobs and nano keep their existing lifecycles until adapted explicitly.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum Input {
    Stdin(String),
    Eof,
    Cancel,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum Event {
    Stdout(String),
    Stderr(String),
    WaitingForInput,
    WaitingForChange,
    Exited(i32),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tty {
    #[serde(rename = "isTTY")]
    pub is_tty: bool,
    pub columns: usize,
    pub rows: usize,
    pub ansi_support: bool,
    pub interactive: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lifecycle {
    Running,
    WaitingForInput,
    CancellationRequested,
    Exited(i32),
}
/// Implementations must use the existing WorldState VFS/network/packages and
/// virtual process records. This interface grants no host capability.
pub trait VirtualInteractiveEngine {
    fn state(&self) -> Lifecycle;
    fn input(&mut self, input: Input) -> Result<Vec<Event>, String>;
    fn tty(&self) -> &Tty;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stream_contract_distinguishes_eof_cancel_streams_and_exit() {
        for (value, expected) in [(Input::Eof, "eof"), (Input::Cancel, "cancel")] {
            assert_eq!(serde_json::to_value(value).unwrap()["kind"], expected);
        }
        let stdout = serde_json::to_value(Event::Stdout("text".into())).unwrap();
        let stderr = serde_json::to_value(Event::Stderr("text".into())).unwrap();
        assert_ne!(stdout["kind"], stderr["kind"]);
        assert_eq!(
            serde_json::to_value(Event::Exited(130)).unwrap()["data"],
            130
        );
    }
}
