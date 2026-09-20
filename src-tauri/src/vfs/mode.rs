//! Shared virtual permission-mode evaluation. No host chmod or filesystem access.
#[derive(Clone, Copy, Debug)]
pub struct ModeChange {
    pub mode: u16,
    pub affected: u16,
}
impl ModeChange {
    /// Evaluate an octal or symbolic mode against the existing mode. Directory
    /// set-ID bits survive assignments unless explicitly mentioned.
    pub fn parse(text: &str, initial: u16, umask: u16, directory: bool) -> Option<Self> {
        if text.is_empty() {
            return None;
        }
        let (op, digits) = match text.as_bytes()[0] {
            b'+' | b'-' | b'=' => (text.as_bytes()[0], &text[1..]),
            _ => (b'=', text),
        };
        if !digits.is_empty() && digits.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
            let value = u16::from_str_radix(digits, 8).ok()?;
            if value > 0o7777 {
                return None;
            }
            let affected = if op == b'=' {
                if directory && digits.len() < 5 && !text.starts_with('=') {
                    0o1777 | value
                } else {
                    0o7777
                }
            } else {
                value
            };
            let mode = match op {
                b'+' => initial | value,
                b'-' => initial & !value,
                _ => (initial & !affected) | value,
            };
            return Some(Self { mode, affected });
        }
        let mut mode = initial;
        let mut affected = 0;
        for clause in text.split(',') {
            let bytes = clause.as_bytes();
            let mut i = 0;
            let mut who = 0;
            while let Some(c) = bytes.get(i) {
                who |= match c {
                    b'u' => 0o4700,
                    b'g' => 0o2070,
                    b'o' => 0o1007,
                    b'a' => 0o7777,
                    _ => break,
                };
                i += 1;
            }
            let omitted = who == 0;
            if omitted {
                who = 0o7777;
            }
            if i == bytes.len() {
                return None;
            }
            while i < bytes.len() {
                let op = bytes[i];
                if !b"+-=".contains(&op) {
                    return None;
                }
                i += 1;
                let mut value = 0;
                let mut explicit_special = 0;
                if let Some(c @ (b'u' | b'g' | b'o')) = bytes.get(i) {
                    let bits = (mode
                        >> match c {
                            b'u' => 6,
                            b'g' => 3,
                            _ => 0,
                        })
                        & 7;
                    value = bits | (bits << 3) | (bits << 6);
                    i += 1;
                } else {
                    while let Some(c) = bytes.get(i) {
                        value |= match c {
                            b'r' => 0o444,
                            b'w' => 0o222,
                            b'x' => 0o111,
                            b'X' => {
                                if directory || mode & 0o111 != 0 {
                                    0o111
                                } else {
                                    0
                                }
                            }
                            b's' => {
                                explicit_special |= 0o6000;
                                0o6000
                            }
                            b't' => {
                                explicit_special |= 0o1000;
                                0o1000
                            }
                            _ => break,
                        };
                        i += 1;
                    }
                }
                let preserve = if directory {
                    0o6000 & !explicit_special
                } else {
                    0
                };
                let mask = who & !preserve;
                value &= mask & if omitted { !umask } else { 0o7777 };
                let changed = if op == b'=' { mask } else { value };
                affected |= changed;
                mode = match op {
                    b'+' => mode | value,
                    b'-' => mode & !value,
                    _ => (mode & !mask) | value,
                };
            }
        }
        Some(Self { mode, affected })
    }
}
