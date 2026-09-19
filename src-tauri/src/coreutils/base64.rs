//! Byte-only incremental Base64. GNU 9.7 probes define validation and diagnostics.
use super::foundation_options;
use crate::terminal_io::Output;

pub(crate) struct Base64 {
    pub file: String,
    pub opened: bool,
    pub directory: bool,
    pub handle: Option<u64>,
    pub transform: Transform,
    input: crate::shell::stdio::InputBlocks,
    output: crate::shell::stdio::OutputBuffer,
    decoded_pending: Vec<u8>,
    newline_pending: bool,
}

impl Base64 {
    pub fn new(invocation: &str, args: &[String], posix: bool) -> Result<Self, Box<Output>> {
        let opts = foundation_options::parse("base64", invocation, args, posix)?;
        if let Some(kind) = opts.special {
            return Err(Box::new(Output::success(
                super::foundation_messages::message("base64", invocation, kind),
            )));
        }
        if opts.files.len() > 1 {
            return Err(Box::new(foundation_options::error(
                "base64",
                invocation,
                &format!(
                    "extra operand {}",
                    foundation_options::quote(&opts.files[1])
                ),
                false,
            )));
        }
        Ok(Self {
            file: opts.files.first().cloned().unwrap_or_else(|| "-".into()),
            opened: false,
            directory: false,
            handle: None,
            transform: Transform::new(
                opts.flags.contains(&'d'),
                opts.flags.contains(&'i'),
                opts.wrap.unwrap_or(76),
            ),
            input: crate::shell::stdio::InputBlocks::new(if opts.flags.contains(&'d') {
                4096
            } else {
                30720
            }),
            output: Default::default(),
            decoded_pending: Vec::new(),
            newline_pending: false,
        })
    }

    pub fn push(&mut self, data: &[u8]) -> Vec<u8> {
        let filtered;
        let data = if self.transform.decode && self.transform.ignore {
            filtered = data
                .iter()
                .copied()
                .filter(|b| b.is_ascii_alphanumeric() || b"+/=".contains(b))
                .collect::<Vec<_>>();
            &filtered[..]
        } else {
            data
        };
        let mut out = Vec::new();
        for block in self.input.push(data) {
            let transformed = self.transform.push(&block);
            out.extend(self.publish(transformed, false));
            if self.transform.invalid() {
                break;
            }
        }
        if self.transform.invalid() {
            out.extend(self.output.flush());
        }
        out
    }

    pub fn finish(&mut self) -> Vec<u8> {
        let data = self.input.take();
        let mut transformed = self.transform.push(&data);
        transformed.extend(self.transform.finish());
        let mut out = self.publish(transformed, true);
        out.extend(self.output.flush());
        out
    }

    fn publish(&mut self, mut bytes: Vec<u8>, finishing: bool) -> Vec<u8> {
        if self.transform.decode {
            self.decoded_pending.append(&mut bytes);
            // GNU retains an incomplete quartet between decode blocks, but
            // publishes its valid prefix on EOF or an invalid-input error.
            let retained = if finishing || self.transform.invalid() {
                0
            } else {
                match self.transform.length {
                    2 | 4 => 1,
                    3 => 2,
                    _ => 0,
                }
            };
            let pending = self
                .decoded_pending
                .split_off(self.decoded_pending.len() - retained);
            let out = self.output.push(&self.decoded_pending);
            self.decoded_pending = pending;
            return out;
        }
        let mut out = Vec::new();
        if self.newline_pending {
            out.extend(self.output.push(b"\n"));
            self.newline_pending = false;
        }
        // wrap_write emits a boundary LF on the next write (or normal exit).
        if !finishing && bytes.last() == Some(&b'\n') {
            bytes.pop();
            self.newline_pending = true;
        }
        for part in bytes.split_inclusive(|b| *b == b'\n') {
            if let Some(line) = part.strip_suffix(b"\n") {
                out.extend(self.output.push(line));
                out.extend(self.output.push(b"\n"));
            } else {
                out.extend(self.output.push(part));
            }
        }
        out
    }
}

pub(super) fn wrap(value: &str) -> Result<u64, Box<Output>> {
    let trimmed = value.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let negative = trimmed.starts_with('-');
    let number = trimmed.strip_prefix(['+', '-']).unwrap_or(trimmed);
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Box::new(failure(format!(
            "invalid wrap size: {}",
            foundation_options::quote(value)
        ))));
    }
    let count = number.bytes().fold(0u64, |n, b| {
        n.saturating_mul(10).saturating_add(u64::from(b - b'0'))
    });
    if negative && count != 0 {
        return Err(Box::new(failure(format!(
            "invalid wrap size: {}",
            foundation_options::quote(value)
        ))));
    }
    // GNU uses SIZE_MAX as the no-wrap sentinel; saturated decimal overflow
    // reaches the same sentinel on the locked 64-bit baseline.
    Ok(if count == u64::MAX { 0 } else { count })
}

pub(crate) fn failure(message: String) -> Output {
    Output {
        stderr: format!("base64: {message}\n"),
        status: 1,
        ..Default::default()
    }
}

pub(crate) fn diagnostic(file: &str, error: impl std::fmt::Display, opening: bool) -> Vec<u8> {
    let reason = crate::terminal_io::error_reason(error);
    let reason = if reason == "Too many levels of symbolic links" {
        "Symbolic link loop"
    } else {
        &reason
    };
    format!(
        "base64: {}: {reason}\n",
        if opening {
            foundation_options::shell_quote(file)
        } else {
            "read error".into()
        }
    )
    .into_bytes()
}

pub(crate) struct Transform {
    decode: bool,
    ignore: bool,
    wrap: u64,
    column: u64,
    carry: [u8; 3],
    length: usize,
    previous: u8,
    invalid: bool,
}
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
impl Transform {
    pub fn new(decode: bool, ignore: bool, wrap: u64) -> Self {
        Self {
            decode,
            ignore,
            wrap,
            column: 0,
            carry: [0; 3],
            length: 0,
            previous: 0,
            invalid: false,
        }
    }
    pub fn invalid(&self) -> bool {
        self.invalid
    }
    pub fn push(&mut self, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len().saturating_mul(3));
        for &byte in data {
            if self.invalid {
                break;
            }
            if self.decode {
                if byte == b'\n' {
                    continue;
                }
                let value = match byte {
                    b'A'..=b'Z' => Some(byte - b'A'),
                    b'a'..=b'z' => Some(byte - b'a' + 26),
                    b'0'..=b'9' => Some(byte - b'0' + 52),
                    b'+' => Some(62),
                    b'/' => Some(63),
                    _ => None,
                };
                if let Some(value) = value {
                    match self.length {
                        0 => {}
                        1 => out.push((self.previous << 2) | (value >> 4)),
                        2 => out.push((self.previous << 4) | (value >> 2)),
                        3 => out.push((self.previous << 6) | value),
                        _ => {
                            self.invalid = true;
                            continue;
                        }
                    }
                    self.previous = value;
                    self.length = (self.length + 1) % 4;
                } else if byte == b'=' {
                    match self.length {
                        2 if self.previous & 15 == 0 => self.length = 4,
                        3 if self.previous & 3 == 0 => self.length = 0,
                        4 => self.length = 0,
                        _ => self.invalid = true,
                    }
                } else if !self.ignore {
                    self.invalid = true;
                }
            } else {
                self.carry[self.length] = byte;
                self.length += 1;
                if self.length == 3 {
                    self.quantum(&mut out);
                    self.length = 0;
                }
            }
        }
        out
    }
    fn emit(&mut self, byte: u8, out: &mut Vec<u8>) {
        out.push(byte);
        if self.wrap != 0 {
            self.column += 1;
            if self.column == self.wrap {
                out.push(b'\n');
                self.column = 0;
            }
        }
    }
    fn quantum(&mut self, out: &mut Vec<u8>) {
        let [a, b, c] = self.carry;
        self.emit(ALPHABET[(a >> 2) as usize], out);
        self.emit(
            ALPHABET[(((a & 3) << 4) | if self.length > 1 { b >> 4 } else { 0 }) as usize],
            out,
        );
        self.emit(
            if self.length > 1 {
                ALPHABET[(((b & 15) << 2) | if self.length > 2 { c >> 6 } else { 0 }) as usize]
            } else {
                b'='
            },
            out,
        );
        self.emit(
            if self.length > 2 {
                ALPHABET[(c & 63) as usize]
            } else {
                b'='
            },
            out,
        );
    }
    pub fn finish(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        if self.decode {
            self.invalid |= match self.length {
                0 => false,
                2 => self.previous & 15 != 0,
                3 => self.previous & 3 != 0,
                _ => true,
            };
        } else {
            if self.length != 0 {
                self.quantum(&mut out);
                self.length = 0;
            }
            if self.column != 0 {
                out.push(b'\n');
                self.column = 0;
            }
        }
        out
    }
}
