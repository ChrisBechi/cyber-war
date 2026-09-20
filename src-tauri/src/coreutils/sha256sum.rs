//! Incremental SHA-256 and GNU checksum records. No host or whole-input adapter.
use super::foundation_options;
use crate::terminal_io::Output;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct Options {
    pub check: bool,
    pub binary: Option<bool>,
    pub tag: bool,
    pub zero: bool,
    pub warn: bool,
    pub strict: bool,
    pub quiet: bool,
    pub status: bool,
    pub ignore_missing: bool,
}
impl Options {
    fn flag(&mut self, flag: char) {
        match flag {
            'c' => self.check = true,
            'b' => self.binary = Some(true),
            't' => self.binary = Some(false),
            'z' => self.zero = true,
            'g' => {
                self.tag = true;
                self.binary = Some(true);
            }
            'i' => self.ignore_missing = true,
            'r' => self.strict = true,
            'w' | 'q' | 's' => {
                self.warn = flag == 'w';
                self.quiet = flag == 'q';
                self.status = flag == 's';
            }
            _ => unreachable!(),
        }
    }
    fn validate(&self) -> Option<&'static str> {
        if self.tag && self.binary == Some(false) {
            return Some("--tag does not support --text mode");
        }
        if self.check {
            if self.zero {
                return Some("the --zero option is not supported when verifying checksums");
            }
            if self.tag {
                return Some("the --tag option is meaningless when verifying checksums");
            }
            if self.binary.is_some() {
                return Some(
                    "the --binary and --text options are meaningless when verifying checksums",
                );
            }
        } else {
            for (enabled, message) in [
                (
                    self.ignore_missing,
                    "the --ignore-missing option is meaningful only when verifying checksums",
                ),
                (
                    self.status,
                    "the --status option is meaningful only when verifying checksums",
                ),
                (
                    self.warn,
                    "the --warn option is meaningful only when verifying checksums",
                ),
                (
                    self.quiet,
                    "the --quiet option is meaningful only when verifying checksums",
                ),
                (
                    self.strict,
                    "the --strict option is meaningful only when verifying checksums",
                ),
            ] {
                if enabled {
                    return Some(message);
                }
            }
        }
        None
    }
}

pub(crate) fn escaped(name: &[u8], escape: bool) -> Vec<u8> {
    let mut result = Vec::with_capacity(name.len());
    for &byte in name {
        match (escape, byte) {
            (true, b'\\') => result.extend_from_slice(b"\\\\"),
            (true, b'\n') => result.extend_from_slice(b"\\n"),
            (true, b'\r') => result.extend_from_slice(b"\\r"),
            _ => result.push(byte),
        }
    }
    result
}
fn needs_escape(name: &[u8]) -> bool {
    name.iter().any(|b| b"\\\r\n".contains(b))
}
pub(crate) fn emit(name: &[u8], digest: &[u8], opts: &Options) -> Vec<u8> {
    let escape = !opts.zero && needs_escape(name);
    let label = escaped(name, escape);
    let mut out = Vec::new();
    if escape {
        out.push(b'\\');
    }
    if opts.tag {
        out.extend_from_slice(b"SHA256 (");
        out.extend(&label);
        out.extend_from_slice(b") = ");
    }
    for b in digest {
        out.extend_from_slice(format!("{b:02x}").as_bytes());
    }
    if !opts.tag {
        out.extend_from_slice(if opts.binary.unwrap_or(false) {
            b" *"
        } else {
            b"  "
        });
        out.extend(label);
    }
    out.push(if opts.zero { 0 } else { b'\n' });
    out
}
pub(crate) fn result(name: &[u8], verdict: &str) -> Vec<u8> {
    let escape = needs_escape(name);
    let mut out = if escape { vec![b'\\'] } else { Vec::new() };
    out.extend(escaped(name, escape));
    out.extend_from_slice(format!(": {verdict}\n").as_bytes());
    out
}
pub(crate) fn diagnostic(name: &str, reason: impl std::fmt::Display) -> Vec<u8> {
    diagnostic_bytes(name.as_bytes(), reason)
}
pub(crate) fn diagnostic_bytes(name: &[u8], reason: impl std::fmt::Display) -> Vec<u8> {
    format!(
        "sha256sum: {}: {}\n",
        foundation_options::shell_quote_bytes(name),
        super::wc::reason(reason)
    )
    .into_bytes()
}
pub(crate) fn message(text: &str) -> Vec<u8> {
    format!("sha256sum: {text}\n").into_bytes()
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Record {
    pub digest: [u8; 32],
    pub name: Vec<u8>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Line {
    Ignore,
    Malformed,
    Record(Record),
}
fn white(b: &u8) -> bool {
    matches!(b, b' ' | b'\t')
}
fn digest(bytes: &[u8]) -> Option<[u8; 32]> {
    if bytes.len() != 64 || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut value = [0; 32];
    for (slot, pair) in value.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *slot = (char::from(pair[0]).to_digit(16)? * 16 + char::from(pair[1]).to_digit(16)?) as u8;
    }
    Some(value)
}
fn unescape(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            0 => return None,
            b'\\' => {
                i += 1;
                out.push(match bytes.get(i)? {
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b'\\' => b'\\',
                    _ => return None,
                });
            }
            b => out.push(b),
        }
        i += 1;
    }
    Some(out)
}
/// Format selection persists across operands, as with GNU's reversed BSD lists.
#[derive(Default)]
pub(crate) struct Parser {
    reversed: Option<bool>,
}
impl Parser {
    pub fn parse(&mut self, bytes: &[u8]) -> Line {
        if bytes.first() == Some(&b'#') {
            return Line::Ignore;
        }
        let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);
        if bytes.is_empty() {
            return Line::Ignore;
        }
        self.record(bytes).map_or(Line::Malformed, Line::Record)
    }
    fn record(&mut self, bytes: &[u8]) -> Option<Record> {
        let bytes = &bytes[bytes.iter().take_while(|b| white(b)).count()..];
        let (escape, bytes) = bytes
            .strip_prefix(b"\\")
            .map_or((false, bytes), |s| (true, s));
        let (hash, name) = if let Some(tag) = bytes.strip_prefix(b"SHA256") {
            let tag = tag.strip_prefix(b" ").unwrap_or(tag).strip_prefix(b"(")?;
            let end = tag.iter().rposition(|b| *b == b')')?;
            let rest = &tag[end + 1..];
            let rest = &rest[rest.iter().take_while(|b| white(b)).count()..];
            let rest = rest.strip_prefix(b"=")?;
            let rest = &rest[rest.iter().take_while(|b| white(b)).count()..];
            (digest(rest)?, &tag[..end])
        } else {
            if bytes.len() < 66 {
                return None;
            }
            let end = bytes.iter().position(white)?;
            let hash = digest(&bytes[..end])?;
            let mut name = &bytes[end + 1..];
            if name.len() == 1 || !matches!(name.first(), Some(b' ' | b'*')) {
                if self.reversed == Some(false) {
                    return None;
                }
                self.reversed = Some(true);
            } else if self.reversed != Some(true) {
                self.reversed = Some(false);
                name = &name[1..];
            }
            (hash, name)
        };
        // In unescaped records GNU fopen sees the first NUL as C-string EOF.
        let name = if escape {
            unescape(name)?
        } else {
            name.split(|b| *b == 0).next()?.to_vec()
        };
        Some(Record { digest: hash, name })
    }
}

/// One record plus one scheduler chunk. Two escaped bytes per VFS path byte,
/// plus header room; oversized records are drained without growing memory.
pub(crate) const RECORD_LIMIT: usize = 2 * 4096 + 128;
#[derive(Default)]
pub(crate) struct Lines {
    pub block: VecDeque<u8>,
    pub eof: bool,
    partial: Vec<u8>,
    overflow: bool,
}
impl Lines {
    pub fn next(&mut self) -> Option<Option<Vec<u8>>> {
        while let Some(b) = self.block.pop_front() {
            if b == b'\n' {
                return Some(self.take());
            }
            // Leading SP/HT has no semantic length. Keep one byte so blank
            // whitespace and indented comments retain GNU's interpretation.
            if white(&b) && self.partial.len() == 1 && white(&self.partial[0]) {
                continue;
            }
            if self.partial.len() < RECORD_LIMIT {
                self.partial.push(b);
            } else {
                self.overflow = true;
            }
        }
        if self.eof && (!self.partial.is_empty() || self.overflow) {
            Some(self.take())
        } else {
            None
        }
    }
    fn take(&mut self) -> Option<Vec<u8>> {
        let overflow = std::mem::take(&mut self.overflow);
        let bytes = std::mem::take(&mut self.partial);
        if overflow && bytes.first() != Some(&b'#') {
            None
        } else {
            Some(bytes)
        }
    }
    #[cfg(test)]
    pub fn buffered(&self) -> usize {
        self.partial.len() + self.block.len()
    }
}
#[derive(Default)]
pub(crate) struct Summary {
    pub valid: bool,
    pub matched: bool,
    pub malformed: u64,
    pub failed: u64,
    pub mismatched: u64,
    pub line: u64,
}
impl Summary {
    pub fn finish(&self, name: &str, opts: &Options) -> (bool, Vec<u8>) {
        let mut err = Vec::new();
        if !self.valid {
            err.extend(diagnostic(
                name,
                "no properly formatted checksum lines found",
            ));
        } else if !opts.status {
            for (count, singular, plural) in [
                (
                    self.malformed,
                    "line is improperly formatted",
                    "lines are improperly formatted",
                ),
                (
                    self.failed,
                    "listed file could not be read",
                    "listed files could not be read",
                ),
                (
                    self.mismatched,
                    "computed checksum did NOT match",
                    "computed checksums did NOT match",
                ),
            ] {
                if count > 0 {
                    err.extend(message(&format!(
                        "WARNING: {count} {}",
                        if count == 1 { singular } else { plural }
                    )));
                }
            }
            if opts.ignore_missing && !self.matched {
                err.extend(diagnostic(name, "no file was verified"));
            }
        }
        (
            self.valid
                && self.matched
                && self.failed == 0
                && self.mismatched == 0
                && (!opts.strict || self.malformed == 0),
            err,
        )
    }
}
pub(crate) struct Checksum {
    pub opts: Options,
    pub files: VecDeque<String>,
    pub list: Option<String>,
    pub list_handle: Option<u64>,
    pub list_directory: bool,
    pub current: Option<Vec<u8>>,
    pub handle: Option<u64>,
    pub directory: bool,
    pub expected: Option<[u8; 32]>,
    pub hash: Sha256,
    pub lines: Lines,
    pub parser: Parser,
    pub summary: Summary,
}
impl Checksum {
    pub fn new(invocation: &str, args: &[String], posix: bool) -> Result<Self, Box<Output>> {
        let parsed = foundation_options::parse("sha256sum", invocation, args, posix)?;
        if let Some(kind) = parsed.special {
            return Err(Box::new(Output::success(
                super::foundation_messages::message("sha256sum", invocation, kind),
            )));
        }
        let mut opts = Options::default();
        for flag in parsed.flags {
            opts.flag(flag);
        }
        if let Some(reason) = opts.validate() {
            return Err(Box::new(foundation_options::error(
                "sha256sum",
                invocation,
                reason,
                false,
            )));
        }
        Ok(Self {
            opts,
            files: if parsed.files.is_empty() {
                ["-".into()].into()
            } else {
                parsed.files.into()
            },
            list: None,
            list_handle: None,
            list_directory: false,
            current: None,
            handle: None,
            directory: false,
            expected: None,
            hash: Sha256::new(),
            lines: Lines::default(),
            parser: Parser::default(),
            summary: Summary::default(),
        })
    }
    pub fn list_name(&self) -> &str {
        match self.list.as_deref() {
            Some("-") => "standard input",
            Some(name) => name,
            None => "",
        }
    }
}
