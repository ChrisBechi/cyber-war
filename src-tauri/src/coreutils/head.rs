//! Incremental prefix selection. Retain only the suffix needed by negative counts.
use super::foundation_options;
use crate::terminal_io::Output;
use std::collections::VecDeque;

#[derive(Clone, Copy)]
pub(crate) struct Selection {
    pub bytes: bool,
    pub negative: bool,
    pub count: u64,
}
impl Default for Selection {
    fn default() -> Self {
        Self {
            bytes: false,
            negative: false,
            count: 10,
        }
    }
}
impl Selection {
    pub fn parse(kind: char, value: &str) -> Result<Self, Box<Output>> {
        Self::parse_for("head", kind, value)
    }
    pub(super) fn parse_for(name: &str, kind: char, value: &str) -> Result<Self, Box<Output>> {
        let negative = value.starts_with('-');
        let value = if negative { &value[1..] } else { value };
        let number = value.trim_start_matches(|c: char| c.is_ascii_whitespace());
        let number = number.strip_prefix('+').unwrap_or(number);
        let digits = number.bytes().take_while(u8::is_ascii_digit).count();
        let suffix = &number[digits..];
        let scale = match suffix {
            "" => Some((1u64, 0)),
            "b" => Some((512, 1)),
            _ if suffix.is_ascii() => {
                let unit = suffix.as_bytes()[0];
                let exponent = match unit {
                    b'k' | b'K' => 1,
                    b'm' | b'M' => 2,
                    b'G' => 3,
                    b'T' => 4,
                    b'P' => 5,
                    b'E' => 6,
                    b'Z' => 7,
                    b'Y' => 8,
                    b'R' => 9,
                    b'Q' => 10,
                    _ => 0,
                };
                match (&suffix[1..], exponent) {
                    ("", 1..) | ("iB", 1..) => Some((1024, exponent)),
                    ("B", 1..) => Some((1000, exponent)),
                    _ => None,
                }
            }
            _ => None,
        };
        let Some((base, power)) = scale.filter(|_| digits > 0) else {
            return Err(Box::new(Output {
                stderr: format!(
                    "{name}: invalid number of {}: {}\n",
                    if kind == 'c' { "bytes" } else { "lines" },
                    foundation_options::quote(value)
                ),
                status: 1,
                ..Default::default()
            }));
        };
        let count = number[..digits].bytes().fold(0u64, |n, b| {
            n.saturating_mul(10).saturating_add(u64::from(b - b'0'))
        });
        let count = (0..power).fold(count, |n, _| n.saturating_mul(base));
        Ok(Self {
            bytes: kind == 'c',
            negative,
            count,
        })
    }
}

pub(crate) struct Head {
    pub files: VecDeque<String>,
    pub current: Option<String>,
    pub handle: Option<u64>,
    pub directory: bool,
    pub source_remaining: Option<u64>,
    pub transform: Transform,
    selection: Selection,
    delimiter: u8,
    headers: bool,
    header_seen: bool,
}
impl Head {
    pub fn new(invocation: &str, args: &[String], posix: bool) -> Result<Self, Box<Output>> {
        let args = historical(invocation, args)?;
        let opts = foundation_options::parse("head", invocation, &args, posix)?;
        if let Some(kind) = opts.special {
            return Err(Box::new(Output::success(
                super::foundation_messages::message("head", invocation, kind),
            )));
        }
        let selection = opts.selection.unwrap_or_default();
        let delimiter = if opts.flags.contains(&'z') { 0 } else { b'\n' };
        let headers = opts
            .flags
            .iter()
            .rev()
            .find(|c| matches!(c, 'q' | 'v'))
            .map_or(opts.files.len() > 1, |c| *c == 'v');
        Ok(Self {
            files: if opts.files.is_empty() {
                ["-".into()].into()
            } else {
                opts.files.into()
            },
            current: None,
            handle: None,
            directory: false,
            source_remaining: None,
            selection,
            delimiter,
            headers,
            header_seen: false,
            transform: Transform::new(selection, delimiter),
        })
    }
    pub fn start(&mut self, file: String) -> Vec<u8> {
        self.source_remaining = None;
        self.transform = Transform::new(self.selection, self.delimiter);
        let header = if self.headers {
            let title = if file == "-" { "standard input" } else { &file };
            let text = format!(
                "{}==> {title} <==\n",
                if self.header_seen { "\n" } else { "" }
            );
            self.header_seen = true;
            text.into_bytes()
        } else {
            Vec::new()
        };
        self.current = Some(file);
        header
    }
    pub fn regular_source(&mut self, remaining: u64) {
        if self.selection.negative {
            self.source_remaining = Some(remaining);
        }
    }
    pub fn read_limit(&self) -> usize {
        self.source_remaining
            .map_or(self.transform.read_limit(), |left| {
                self.transform.read_limit().min(left as usize)
            })
    }
}

fn historical(invocation: &str, args: &[String]) -> Result<Vec<String>, Box<Output>> {
    let Some(first) = args
        .first()
        .filter(|s| s.starts_with('-') && s.as_bytes().get(1).is_some_and(u8::is_ascii_digit))
    else {
        return Ok(args.to_vec());
    };
    let end = 1 + first[1..].bytes().take_while(u8::is_ascii_digit).count();
    let mut kind = 'n';
    let mut multiplier = "";
    let mut flags = String::from("-");
    for byte in first[end..].bytes() {
        let ch = byte as char;
        match ch {
            'c' => {
                kind = 'c';
                multiplier = "";
            }
            'b' => {
                kind = 'c';
                multiplier = "b";
            }
            'k' => {
                kind = 'c';
                multiplier = "k";
            }
            'm' => {
                kind = 'c';
                multiplier = "m";
            }
            'l' => {
                kind = 'n';
                multiplier = "";
            }
            'q' | 'v' | 'z' => flags.push(ch),
            _ => {
                let mut data = b"head: invalid trailing option -- ".to_vec();
                data.push(byte);
                data.extend_from_slice(
                    format!("\nTry '{invocation} --help' for more information.\n").as_bytes(),
                );
                return Err(Box::new(Output {
                    stderr: String::from_utf8_lossy(&data).into_owned(),
                    byte_ordered: vec![(2, data)],
                    status: 1,
                    ..Default::default()
                }));
            }
        }
    }
    let mut result = vec![
        format!("-{kind}"),
        format!("{}{multiplier}", &first[1..end]),
    ];
    if flags.len() > 1 {
        result.push(flags);
    }
    result.extend_from_slice(&args[1..]);
    Ok(result)
}

pub(crate) struct Transform {
    selection: Selection,
    delimiter: u8,
    remaining: u64,
    suffix: VecDeque<u8>,
    records: u64,
    line_chunks: VecDeque<(Vec<u8>, u64)>,
    read_buffer: Vec<u8>,
}
impl Transform {
    pub fn new(selection: Selection, delimiter: u8) -> Self {
        Self {
            remaining: selection.count,
            selection,
            delimiter,
            suffix: VecDeque::new(),
            records: 0,
            line_chunks: VecDeque::new(),
            read_buffer: Vec::new(),
        }
    }
    pub fn done(&self) -> bool {
        !self.selection.negative && self.remaining == 0
    }
    // GNU 9.7 in the locked musl environment reads 1024-byte buffers.
    // These read boundaries are observable with repeated pipe operands/signals;
    // scheduler output chunks and bounded channel capacity remain unchanged.
    fn window(&self) -> usize {
        if self.selection.count <= 1024 * 1024 {
            1024 + self.selection.count as usize
        } else {
            1024
        }
    }
    pub fn read_limit(&self) -> usize {
        if self.selection.bytes {
            if self.selection.negative {
                (self.window() - self.read_buffer.len()).min(1024)
            } else {
                self.remaining.min(1024) as usize
            }
        } else {
            1024
        }
    }
    pub fn retained(&self) -> usize {
        if self.selection.negative {
            self.suffix.len() + self.line_chunks.iter().map(|(b, _)| b.len()).sum::<usize>()
        } else {
            0
        }
    }
    fn suffix_bytes(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        for &b in bytes {
            self.suffix.push_back(b);
            if self.suffix.len() as u64 > self.selection.count {
                out.push(self.suffix.pop_front().unwrap());
            }
        }
        out
    }
    /// Returns output and the number of bytes logically consumed (regular files
    /// can seek back after a line read; a pipe cannot undo read-ahead).
    pub fn push(&mut self, bytes: &[u8]) -> (Vec<u8>, usize) {
        if !self.selection.negative {
            let mut end = 0;
            for &b in bytes {
                if self.remaining == 0 {
                    break;
                }
                end += 1;
                if self.selection.bytes || b == self.delimiter {
                    self.remaining -= 1;
                }
            }
            return (bytes[..end].to_vec(), end);
        }
        let mut out = Vec::new();
        if self.selection.bytes {
            let mut bytes = bytes;
            while !bytes.is_empty() {
                let take = bytes.len().min(self.window() - self.read_buffer.len());
                self.read_buffer.extend_from_slice(&bytes[..take]);
                bytes = &bytes[take..];
                if self.read_buffer.len() == self.window() {
                    let block = std::mem::take(&mut self.read_buffer);
                    out.extend(self.suffix_bytes(&block));
                }
            }
        } else if self.selection.count == 0 {
            out.extend_from_slice(bytes);
        } else {
            let records = bytes.iter().filter(|b| **b == self.delimiter).count() as u64;
            self.records += records;
            if let Some((last, count)) = self
                .line_chunks
                .back_mut()
                .filter(|(last, _)| last.len() + bytes.len() < 1024)
            {
                last.extend_from_slice(bytes);
                *count += records;
            } else {
                self.line_chunks.push_back((bytes.to_vec(), records));
                if let Some((_, count)) = self.line_chunks.front() {
                    if self.records - count > self.selection.count {
                        let (block, count) = self.line_chunks.pop_front().unwrap();
                        self.records -= count;
                        out.extend(block);
                    }
                }
            }
        }
        (out, bytes.len())
    }
    pub fn finish(&mut self) -> Vec<u8> {
        if self.selection.negative && self.selection.bytes {
            let remainder = std::mem::take(&mut self.read_buffer);
            return self.suffix_bytes(&remainder);
        }
        let mut out = Vec::new();
        if self.selection.negative && !self.selection.bytes {
            let bytes: Vec<u8> = self.line_chunks.drain(..).flat_map(|(b, _)| b).collect();
            let records =
                self.records + u64::from(bytes.last().is_some_and(|b| *b != self.delimiter));
            let mut keep = records.saturating_sub(self.selection.count);
            let mut end = 0;
            while keep > 0 && end < bytes.len() {
                if bytes[end] == self.delimiter {
                    keep -= 1;
                }
                end += 1;
            }
            out.extend_from_slice(&bytes[..end]);
            self.suffix.extend(&bytes[end..]);
        }
        out
    }
}

pub(crate) fn diagnostic(file: &str, reason: impl std::fmt::Display, opening: bool) -> Vec<u8> {
    let reason = crate::terminal_io::error_reason(reason);
    let reason = if reason == "Too many levels of symbolic links" {
        "Symbolic link loop"
    } else {
        &reason
    };
    format!(
        "head: {} {}{}: {reason}\n",
        if opening {
            "cannot open"
        } else {
            "error reading"
        },
        foundation_options::shell_quote_always(file),
        if opening { " for reading" } else { "" }
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefix_and_suffix_selection_is_independent_of_chunks() {
        let mut input: Vec<u8> = (0..=255).cycle().take(9000).collect();
        input.extend_from_slice(b"a\nb\nlast");
        for delimiter in [0, b'\n'] {
            for bytes in [false, true] {
                for negative in [false, true] {
                    for count in [0, 1, 2, 9, 8192, u64::MAX] {
                        let selection = Selection {
                            bytes,
                            negative,
                            count,
                        };
                        let n = usize::try_from(count).unwrap_or(usize::MAX);
                        let expected = if bytes {
                            input[..if negative {
                                input.len().saturating_sub(n)
                            } else {
                                n.min(input.len())
                            }]
                                .to_vec()
                        } else {
                            let records: Vec<_> =
                                input.split_inclusive(|b| *b == delimiter).collect();
                            let keep = if negative {
                                records.len().saturating_sub(n)
                            } else {
                                n.min(records.len())
                            };
                            records[..keep].concat()
                        };
                        for size in [1, 2, 3, 7, 31, 255, 4096] {
                            let mut transform = Transform::new(selection, delimiter);
                            let mut actual = Vec::new();
                            for chunk in input.chunks(size) {
                                actual.extend(transform.push(chunk).0);
                            }
                            actual.extend(transform.finish());
                            assert_eq!(
                                actual, expected,
                                "bytes={bytes}, negative={negative}, count={count}, chunk={size}"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn negative_byte_retention_is_bounded_by_suffix_not_stream_length() {
        let mut transform = Transform::new(
            Selection {
                bytes: true,
                negative: true,
                count: 17,
            },
            b'\n',
        );
        for _ in 0..4096 {
            transform.push(&[7; 4096]);
            assert_eq!(transform.suffix.len(), 17);
        }
    }
    #[test]
    fn tty_prefix_completes_without_eof_and_releases_resources() {
        use crate::shell::control;
        use std::time::Duration;
        for (i, command) in ["head -n 1", "head -c 1"].iter().enumerate() {
            let key = format!("head-early-{i}");
            let registration = control::register_output(&key, None);
            let (tx, rx) = std::sync::mpsc::channel();
            let worker = std::thread::spawn(move || {
                control::run(&registration, || {
                    let mut w = crate::world::WorldState::new("kali", "lifeos").unwrap();
                    let result = crate::terminal::execute(&mut w, command);
                    assert_eq!(w.vfs.open_handle_count(), 0);
                    assert!(!w.processes.iter().any(|p| p.name == "head"));
                    tx.send(result).unwrap();
                })
            });
            assert!(control::input(&key, Some("abc\n".into())));
            let result = rx.recv_timeout(Duration::from_secs(5));
            control::cancel(&key);
            worker.join().unwrap();
            let result = result.expect("head waited for EOF after sufficient input");
            assert_eq!(result.stdout, if i == 0 { "abc\n" } else { "a" });
            assert_eq!(result.exit_code, 0);
            assert!(control::process_ids(&key).is_empty());
        }
    }
    #[test]
    fn repeated_regular_stdin_and_package_resolution() {
        let mut w = crate::world::WorldState::new("kali", "lifeos").unwrap();
        w.vfs.write("/home/kali/a", "a\nb\nc", "kali").unwrap();
        let result = crate::terminal::execute(&mut w, "head -qn1 - - < a");
        assert_eq!(result.stdout, "a\nb\n");
        assert_eq!(w.vfs.open_handle_count(), 0);
        w.packages.installed.remove("coreutils");
        assert_eq!(crate::terminal::execute(&mut w, "head a").exit_code, 126);
    }
}
