//! Bounded stdout buffering for the pinned musl reference ABI. Byte endpoints
//! remain unbuffered; a program may opt into this FILE-style adapter.
pub struct InputBlocks {
    bytes: Vec<u8>,
    capacity: usize,
}
impl InputBlocks {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            bytes: Vec::new(),
            capacity,
        }
    }
    pub fn push(&mut self, mut input: &[u8]) -> Vec<Vec<u8>> {
        let mut blocks = Vec::new();
        while !input.is_empty() {
            let length = input.len().min(self.capacity - self.bytes.len());
            self.bytes.extend_from_slice(&input[..length]);
            input = &input[length..];
            if self.bytes.len() == self.capacity {
                blocks.push(self.take());
            }
        }
        blocks
    }
    pub fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}

#[derive(Default)]
pub struct OutputBuffer {
    bytes: Vec<u8>,
    established: bool,
}
impl OutputBuffer {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<u8> {
        const CAPACITY: usize = 1024;
        if bytes.len() > CAPACITY - self.bytes.len() {
            let mut output = std::mem::take(&mut self.bytes);
            output.extend_from_slice(bytes);
            self.established = true;
            return output;
        }
        let mut output = Vec::new();
        if !self.established {
            if let Some(end) = bytes.iter().rposition(|b| *b == b'\n') {
                output = std::mem::take(&mut self.bytes);
                output.extend_from_slice(&bytes[..=end]);
                self.established = true;
                self.bytes.extend_from_slice(&bytes[end + 1..]);
                return output;
            }
        }
        self.bytes.extend_from_slice(bytes);
        output
    }
    pub fn flush(&mut self) -> Vec<u8> {
        self.established = true;
        std::mem::take(&mut self.bytes)
    }
    pub fn clear(&mut self) {
        self.bytes.clear();
    }
}
