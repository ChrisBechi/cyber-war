//! Incremental counters for the certified C locale. No text-sized allocation.
use super::foundation_options;
use crate::{terminal_io::Output, world::WorldState};
use std::collections::VecDeque;
pub(crate) fn quote_path(value: &str) -> String {
    foundation_options::shell_quote(value)
}
pub(crate) fn quote_always(value: &str) -> String {
    foundation_options::shell_quote_always(value)
}
pub(crate) fn manual() -> String {
    super::foundation_messages::message("wc", "wc", "help")
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Total {
    #[default]
    Auto,
    Always,
    Only,
    Never,
}
impl Total {
    pub fn parse(value: &str, invocation: &str) -> Result<Self, Box<Output>> {
        let modes = [
            ("auto", Self::Auto),
            ("always", Self::Always),
            ("only", Self::Only),
            ("never", Self::Never),
        ];
        if let Some((_, mode)) = modes.iter().find(|(name, _)| *name == value) {
            return Ok(*mode);
        }
        let found: Vec<_> = modes
            .iter()
            .filter(|(name, _)| name.starts_with(value))
            .collect();
        if found.len() == 1 {
            return Ok(found[0].1);
        }
        Err(Box::new(Output { status: 1, stderr: format!("wc: {} argument {} for '--total'\nValid arguments are:\n  - 'auto'\n  - 'always'\n  - 'only'\n  - 'never'\nTry '{invocation} --help' for more information.\n", if found.is_empty() { "invalid" } else { "ambiguous" }, foundation_options::quote(value)), ..Default::default() }))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counts {
    pub values: [u64; 5], // LF, words, characters, bytes, maximum columns
    pub overflow: [bool; 4],
}
impl Counts {
    fn add(&mut self, i: usize, value: u64) {
        let (sum, overflow) = self.values[i].overflowing_add(value);
        self.overflow[i] |= overflow;
        self.values[i] = if overflow { u64::MAX } else { sum };
    }
    pub fn merge(&mut self, other: &Self) {
        for i in 0..4 {
            self.add(i, other.values[i]);
            self.overflow[i] |= other.overflow[i];
        }
        self.values[4] = self.values[4].max(other.values[4]);
    }
}

pub(crate) struct Counter {
    pub counts: Counts,
    column: u64,
    in_word: bool,
    utf8: bool,
    posix: bool,
    pending: [u8; 4],
    pending_len: usize,
}
impl Counter {
    pub fn new(utf8: bool, posix: bool) -> Self {
        Self {
            counts: Counts::default(),
            column: 0,
            in_word: false,
            utf8,
            posix,
            pending: [0; 4],
            pending_len: 0,
        }
    }
    fn word(&mut self, nonspace: bool) {
        if nonspace && !self.in_word {
            self.counts.add(1, 1);
        }
        self.in_word = nonspace;
    }
    fn character(&mut self, c: char) {
        self.counts.add(2, 1);
        if c == '\n' {
            self.counts.add(0, 1);
        }
        // The locked C classification is exactly SP, HT, LF, VT, FF, CR.
        // Retain the pre-existing non-certified UTF-8 word/character profile.
        let space = if self.utf8 {
            c.is_whitespace() || (!self.posix && c == '\u{2060}')
        } else {
            matches!(c, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r')
        };
        self.word(!space);
        match c {
            '\n' | '\r' | '\x0c' => {
                self.counts.values[4] = self.counts.values[4].max(self.column);
                self.column = 0;
            }
            '\t' => self.column = self.column.saturating_add(8 - self.column % 8),
            ' '..='~' => self.column = self.column.saturating_add(1),
            _ => {}
        }
    }
    pub fn push(&mut self, bytes: &[u8]) {
        self.counts.add(3, bytes.len() as u64);
        for &b in bytes {
            if !self.utf8 {
                self.character(char::from(b));
                continue;
            }
            self.pending[self.pending_len] = b;
            self.pending_len += 1;
            loop {
                match std::str::from_utf8(&self.pending[..self.pending_len]) {
                    Ok(text) => {
                        let c = text.chars().next().expect("nonempty decoder state");
                        self.character(c);
                        self.pending_len = 0;
                        break;
                    }
                    Err(e) if e.error_len().is_none() => break,
                    Err(_) => {
                        // Invalid encoding bytes remain non-whitespace, but are not characters.
                        self.word(true);
                        self.pending.copy_within(1..self.pending_len, 0);
                        self.pending_len -= 1;
                        if self.pending_len == 0 {
                            break;
                        }
                    }
                }
            }
        }
    }
    pub fn finish(&mut self) {
        if self.pending_len > 0 {
            self.word(true);
            self.pending_len = 0;
        }
        self.counts.values[4] = self.counts.values[4].max(self.column);
    }
}

/// A single partial filename plus one bounded input block. A NUL is never a
/// textual newline; a trailing delimiter does not create an extra filename.
#[derive(Default)]
pub(crate) struct Names {
    pub block: VecDeque<u8>,
    partial: Vec<u8>,
    pub eof: bool,
}
impl Names {
    pub fn next(&mut self) -> Result<Option<String>, &'static str> {
        while let Some(b) = self.block.pop_front() {
            if b == 0 {
                return self.take().map(Some);
            }
            if self.partial.len() >= 4096 {
                return Err("File name too long");
            }
            self.partial.push(b);
        }
        if self.eof && !self.partial.is_empty() {
            self.take().map(Some)
        } else {
            Ok(None)
        }
    }
    fn take(&mut self) -> Result<String, &'static str> {
        String::from_utf8(std::mem::take(&mut self.partial))
            .map_err(|_| "Invalid or incomplete multibyte or wide character")
    }
}

pub(crate) struct Wc {
    pub files: VecDeque<String>,
    pub current: Option<String>,
    pub handle: Option<u64>,
    pub directory: bool,
    pub counter: Counter,
    pub sum: Counts,
    pub total: Total,
    pub selected: Vec<usize>,
    pub width: usize,
    pub initialized: bool,
    pub implicit: bool,
    pub count: u64,
    pub list: Option<String>,
    pub list_handle: Option<u64>,
    pub list_directory: bool,
    pub names: Names,
    pub scan: bool,
    pub scan_count: u64,
    pub regular_total: u64,
    pub minimum_width: usize,
    pub list_offset: usize,
    utf8: bool,
    posix: bool,
}
impl Wc {
    pub fn new(
        invocation: &str,
        args: &[String],
        posix: bool,
        world: &WorldState,
    ) -> Result<Self, Box<Output>> {
        let opts = foundation_options::parse("wc", invocation, args, posix)?;
        if let Some(kind) = opts.special {
            return Err(Box::new(Output::success(
                super::foundation_messages::message("wc", invocation, kind),
            )));
        }
        if opts.files0.is_some() && !opts.files.is_empty() {
            return Err(Box::new(Output { status: 1, stderr: format!("wc: extra operand {}\nfile operands cannot be combined with --files0-from\nTry '{invocation} --help' for more information.\n", foundation_options::shell_quote_always(&opts.files[0])), ..Default::default() }));
        }
        let selected = "lwmcL"
            .chars()
            .enumerate()
            .filter_map(|(i, c)| {
                (opts.flags.contains(&c) || (opts.flags.is_empty() && matches!(c, 'l' | 'w' | 'c')))
                    .then_some(i)
            })
            .collect();
        let env = crate::terminal::virtual_env(world, &world.terminal.user);
        let locale = env
            .get("LC_ALL")
            .filter(|s| !s.is_empty())
            .or_else(|| env.get("LC_CTYPE").filter(|s| !s.is_empty()))
            .or_else(|| env.get("LANG"))
            .map(String::as_str)
            .unwrap_or("C.UTF-8");
        let utf8 = matches!(locale, "C.UTF-8" | "C.utf8");
        let implicit = opts.files.is_empty() && opts.files0.is_none();
        Ok(Self {
            files: if implicit {
                ["-".into()].into()
            } else {
                opts.files.into()
            },
            current: None,
            handle: None,
            directory: false,
            counter: Counter::new(utf8, posix),
            sum: Counts::default(),
            total: opts.wc_total,
            selected,
            width: 1,
            initialized: false,
            implicit,
            count: 0,
            list: opts.files0,
            list_handle: None,
            list_directory: false,
            names: Names::default(),
            scan: false,
            scan_count: 0,
            regular_total: 0,
            minimum_width: 1,
            list_offset: 0,
            utf8,
            posix,
        })
    }
    pub fn reset_counter(&mut self) {
        self.counter = Counter::new(self.utf8, self.posix);
    }
    pub fn format(&self, counts: &Counts, label: Option<&str>) -> Vec<u8> {
        let mut text = self
            .selected
            .iter()
            .map(|i| format!("{:>width$}", counts.values[*i], width = self.width))
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(label) = label {
            text.push(' ');
            text.push_str(&if label.contains('\n') {
                foundation_options::shell_quote(label)
            } else {
                label.into()
            });
        }
        text.push('\n');
        text.into_bytes()
    }
    pub fn finish_file(&mut self) -> Vec<u8> {
        self.counter.finish();
        self.sum.merge(&self.counter.counts);
        let output = if self.total == Total::Only {
            Vec::new()
        } else {
            self.format(
                &self.counter.counts,
                if self.implicit {
                    None
                } else {
                    self.current.as_deref()
                },
            )
        };
        self.current = None;
        output
    }
    pub fn finish(&self) -> (Vec<u8>, Vec<u8>) {
        if self.total == Total::Never || (self.total == Total::Auto && self.count <= 1) {
            return (Vec::new(), Vec::new());
        }
        let mut errors = Vec::new();
        for (i, name) in ["lines", "words", "characters", "bytes"].iter().enumerate() {
            if self.sum.overflow[i] {
                errors.extend(
                    format!("wc: total {name}: Value too large for defined data type\n").bytes(),
                );
            }
        }
        (
            self.format(&self.sum, (self.total != Total::Only).then_some("total")),
            errors,
        )
    }
}
pub(crate) fn reason(error: impl std::fmt::Display) -> String {
    let reason = crate::terminal_io::error_reason(error);
    if reason == "Too many levels of symbolic links" {
        "Symbolic link loop".into()
    } else {
        reason
    }
}
pub(crate) fn diagnostic(name: &str, error: impl std::fmt::Display) -> Vec<u8> {
    let reason = reason(error);
    format!("wc: {}: {reason}\n", foundation_options::shell_quote(name)).into_bytes()
}
