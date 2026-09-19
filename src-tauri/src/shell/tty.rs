//! Minimal canonical byte input. Echo belongs to the terminal UI, independently
//! of program stdout. A canonical EOF is a read event, not a persistent close.
use std::collections::VecDeque;
pub const CAPACITY: usize = 64 * 1024;
#[derive(Debug, PartialEq, Eq)]
pub enum Read {
    Data(Vec<u8>),
    Pending,
    Eof,
    Interrupted,
}
#[derive(Default)]
pub struct VirtualTty {
    line: Vec<u8>,
    ready: VecDeque<Read>,
    bytes: usize,
}

impl VirtualTty {
    pub fn input(&mut self, bytes: &[u8]) -> bool {
        if self.bytes + bytes.len() > CAPACITY {
            return false;
        }
        self.bytes += bytes.len();
        for &byte in bytes {
            self.line.push(byte);
            if byte == b'\n' {
                self.flush();
            }
        }
        true
    }
    fn flush(&mut self) {
        if !self.line.is_empty() {
            self.ready
                .push_back(Read::Data(std::mem::take(&mut self.line)));
        }
    }
    pub fn eof(&mut self) -> bool {
        if self.ready.len() >= CAPACITY {
            return false;
        }
        if self.line.is_empty() {
            self.ready.push_back(Read::Eof);
        } else {
            self.flush();
        }
        true
    }
    #[cfg(test)]
    pub fn read(&mut self) -> Read {
        self.read_limit(usize::MAX)
    }
    pub fn read_limit(&mut self, limit: usize) -> Read {
        let Some(mut read) = self.ready.pop_front() else {
            return Read::Pending;
        };
        if let Read::Data(bytes) = &mut read {
            if bytes.len() > limit {
                self.ready.push_front(Read::Data(bytes.split_off(limit)));
            }
            self.bytes -= bytes.len();
        }
        read
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_pending_partial_eof_and_repeated_reads() {
        let mut tty = VirtualTty::default();
        assert_eq!(tty.read(), Read::Pending);
        assert!(tty.input(b"ab\0"));
        assert_eq!(tty.read(), Read::Pending);
        assert!(tty.eof());
        assert_eq!(tty.read(), Read::Data(b"ab\0".to_vec()));
        assert_eq!(tty.read(), Read::Pending);
        assert!(tty.eof());
        assert_eq!(tty.read(), Read::Eof);
        assert!(tty.input(b"next\n"));
        assert_eq!(tty.read(), Read::Data(b"next\n".to_vec()));
        assert!(!tty.input(&vec![1; CAPACITY + 1]));
        assert_eq!(tty.read(), Read::Pending);
    }
}
