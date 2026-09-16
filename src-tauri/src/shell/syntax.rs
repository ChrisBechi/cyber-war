//! A contextual lexer and an AST shared by interactive input and script files.
//! Expansion is deliberately absent here: substituted text cannot become syntax.
use std::fmt;

pub const MAX_SOURCE: usize = 64 * 1024;
pub const MAX_DEPTH: usize = 8;
pub const MAX_STAGES: usize = 16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
    pub incomplete: bool,
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bash: {} at byte {}", self.message, self.span.start)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Literal(String, bool),
    Parameter(String, bool),
    Substitution(Box<List>, bool),
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Word {
    pub segments: Vec<Segment>,
    pub span: Span,
}
impl Word {
    pub fn assignment(&self) -> Option<(String, Self)> {
        let Segment::Literal(first, false) = self.segments.first()? else {
            return None;
        };
        let (name, value) = first.split_once('=')?;
        if !super::identifier(name) {
            return None;
        }
        let mut rhs = self.clone();
        rhs.segments[0] = Segment::Literal(value.into(), false);
        Some((name.into(), rhs))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RedirectKind {
    Read,
    Write,
    Append,
    Duplicate,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redirect {
    pub fd: u8,
    pub kind: RedirectKind,
    pub target: Word,
    pub span: Span,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Command {
    pub words: Vec<Word>,
    pub redirects: Vec<Redirect>,
    pub span: Span,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    Always,
    Success,
    Failure,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AndOr {
    pub pipelines: Vec<(Condition, Pipeline)>,
    pub background: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct List {
    pub items: Vec<AndOr>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseResult {
    Complete(List),
    Incomplete(Diagnostic),
    Error(Diagnostic),
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Word(Word),
    Pipe,
    And,
    Or,
    Semi,
    Newline,
    Background,
    Redirect(u8, RedirectKind),
    Close,
}
#[derive(Clone, Debug)]
struct Token {
    kind: Kind,
    span: Span,
}

struct Lexer<'a> {
    text: &'a str,
    pos: usize,
    depth: usize,
}
impl Lexer<'_> {
    fn peek(&self) -> Option<char> {
        self.text[self.pos..].chars().next()
    }
    fn take(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn error(&self, start: usize, message: &str, incomplete: bool) -> Diagnostic {
        Diagnostic {
            span: Span {
                start,
                end: self.pos,
            },
            message: message.into(),
            incomplete,
        }
    }
    fn literal(word: &mut Word, c: &str, protected: bool) {
        if let Some(Segment::Literal(s, p)) = word.segments.last_mut() {
            if *p == protected {
                s.push_str(c);
                return;
            }
        }
        word.segments.push(Segment::Literal(c.into(), protected));
    }
    fn parameter(&mut self, word: &mut Word, quoted: bool) -> Result<(), Diagnostic> {
        let start = self.pos - 1;
        let name = if self.peek() == Some('(') {
            self.take();
            if self.depth >= MAX_DEPTH {
                return Err(self.error(start, "substitution depth limit", false));
            }
            self.depth += 1;
            let tokens = self.tokens(true)?;
            self.depth -= 1;
            let list = Parser {
                tokens,
                pos: 0,
                end: self.pos,
            }
            .list()?;
            word.segments
                .push(Segment::Substitution(Box::new(list), quoted));
            return Ok(());
        } else if self.peek() == Some('{') {
            self.take();
            let mut name = String::new();
            loop {
                match self.take() {
                    Some('}') => break,
                    Some(c) => name.push(c),
                    None => return Err(self.error(start, "unclosed parameter expansion", true)),
                }
            }
            name
        } else if self
            .peek()
            .is_some_and(|c| c.is_ascii_digit() || "@*#?$!".contains(c))
        {
            self.take().unwrap().to_string()
        } else {
            let mut name = String::new();
            while self
                .peek()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                name.push(self.take().unwrap());
            }
            name
        };
        if name.is_empty() {
            Self::literal(word, "$", quoted);
        } else if super::identifier(&name)
            || name.chars().all(|c| c.is_ascii_digit())
            || ["@", "*", "#", "?", "$", "!"].contains(&name.as_str())
        {
            word.segments.push(Segment::Parameter(name, quoted));
        } else {
            return Err(self.error(start, "unsupported parameter expansion", false));
        }
        Ok(())
    }
    fn word(&mut self) -> Result<Word, Diagnostic> {
        let start = self.pos;
        let mut word = Word::default();
        let mut quote = None;
        let mut quote_had_content = false;
        while let Some(c) = self.peek() {
            if quote.is_none() && (" \t\r\n".contains(c) || "|&;<>)(".contains(c)) {
                break;
            }
            self.take();
            if quote == Some('\'') {
                if c == '\'' {
                    if !quote_had_content {
                        word.segments.push(Segment::Literal(String::new(), true));
                    }
                    quote = None;
                } else {
                    Self::literal(&mut word, &c.to_string(), true);
                    quote_had_content = true;
                }
            } else if c == '\\' {
                let next = self
                    .take()
                    .ok_or_else(|| self.error(start, "trailing escape", true))?;
                if next == '\n' {
                    continue;
                }
                if quote == Some('"') && !['$', '`', '"', '\\'].contains(&next) {
                    Self::literal(&mut word, "\\", true);
                }
                Self::literal(&mut word, &next.to_string(), true);
                quote_had_content = true;
            } else if c == '"' || (c == '\'' && quote.is_none()) {
                if quote == Some(c) {
                    if !quote_had_content {
                        word.segments.push(Segment::Literal(String::new(), true));
                    }
                    quote = None;
                } else {
                    quote = Some(c);
                    quote_had_content = false;
                }
            } else if c == '$' {
                self.parameter(&mut word, quote.is_some())?;
                quote_had_content = true;
            } else if c == '`' {
                return Err(self.error(start, "backtick substitution is not implemented", false));
            } else {
                Self::literal(&mut word, &c.to_string(), quote.is_some());
                quote_had_content = true;
            }
        }
        if quote.is_some() {
            return Err(self.error(start, "unterminated quote", true));
        }
        word.span = Span {
            start,
            end: self.pos,
        };
        Ok(word)
    }
    fn tokens(&mut self, nested: bool) -> Result<Vec<Token>, Diagnostic> {
        let mut tokens = Vec::new();
        while let Some(c) = self.peek() {
            let start = self.pos;
            if c == ' ' || c == '\t' || c == '\r' {
                self.take();
                continue;
            }
            if c == '#' {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.take();
                }
                continue;
            }
            if c == ')' && nested {
                self.take();
                return Ok(tokens);
            }
            let remaining = &self.text[self.pos..];
            // Only a lexical, unquoted IO_NUMBER adjacent to a redirect is a descriptor.
            let digits = remaining.bytes().take_while(u8::is_ascii_digit).count();
            let after = &remaining[digits..];
            let redir = after.starts_with('>') || after.starts_with('<');
            let kind = if redir {
                let fd = if digits == 0 {
                    if after.starts_with('<') {
                        0
                    } else {
                        1
                    }
                } else {
                    remaining[..digits]
                        .parse::<u8>()
                        .map_err(|_| self.error(start, "unsupported descriptor", false))?
                };
                self.pos += digits;
                let op = self.take().unwrap();
                let kind = if self.peek() == Some(op) {
                    self.take();
                    if op == '<' {
                        return Err(self.error(start, "here documents are not implemented", false));
                    }
                    RedirectKind::Append
                } else if self.peek() == Some('&') {
                    self.take();
                    if op == '<' {
                        return Err(self.error(
                            start,
                            "input descriptor duplication is not implemented",
                            false,
                        ));
                    }
                    RedirectKind::Duplicate
                } else if op == '<' {
                    RedirectKind::Read
                } else {
                    RedirectKind::Write
                };
                if fd > 2 || (op == '<' && fd != 0) || (op == '>' && fd == 0) {
                    return Err(self.error(start, "unsupported descriptor", false));
                }
                Kind::Redirect(fd, kind)
            } else {
                match c {
                    '\n' => {
                        self.take();
                        Kind::Newline
                    }
                    ';' => {
                        self.take();
                        Kind::Semi
                    }
                    '|' => {
                        self.take();
                        if self.peek() == Some('|') {
                            self.take();
                            Kind::Or
                        } else {
                            Kind::Pipe
                        }
                    }
                    '&' => {
                        self.take();
                        if self.peek() == Some('&') {
                            self.take();
                            Kind::And
                        } else {
                            Kind::Background
                        }
                    }
                    ')' => {
                        self.take();
                        Kind::Close
                    }
                    '(' => {
                        return Err(self.error(start, "subshell groups are not implemented", false))
                    }
                    _ => Kind::Word(self.word()?),
                }
            };
            tokens.push(Token {
                kind,
                span: Span {
                    start,
                    end: self.pos,
                },
            });
            if tokens.len() > 8192 {
                return Err(self.error(start, "token limit", false));
            }
        }
        if nested {
            return Err(self.error(self.pos, "unclosed command substitution", true));
        }
        Ok(tokens)
    }
}
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    end: usize,
}
impl Parser {
    fn peek(&self) -> Option<&Kind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }
    fn error(&self, message: &str) -> Diagnostic {
        Diagnostic {
            span: self.tokens.get(self.pos).map_or(
                Span {
                    start: self.end,
                    end: self.end,
                },
                |t| t.span,
            ),
            message: message.into(),
            incomplete: self.pos == self.tokens.len(),
        }
    }
    fn newlines(&mut self) {
        while self.peek() == Some(&Kind::Newline) {
            self.pos += 1;
        }
    }
    fn command(&mut self) -> Result<Command, Diagnostic> {
        let mut command = Command {
            span: self
                .tokens
                .get(self.pos)
                .map_or(Span::default(), |t| t.span),
            ..Command::default()
        };
        loop {
            match self.peek().cloned() {
                Some(Kind::Word(word)) => {
                    command.words.push(word);
                    self.pos += 1;
                }
                Some(Kind::Redirect(fd, kind)) => {
                    let span = self.tokens[self.pos].span;
                    self.pos += 1;
                    let Some(Kind::Word(target)) = self.peek().cloned() else {
                        return Err(self.error("expected redirection target"));
                    };
                    if kind == RedirectKind::Duplicate
                        && target
                            .segments
                            .iter()
                            .all(|s| matches!(s, Segment::Literal(_, _)))
                    {
                        let text: String = target
                            .segments
                            .iter()
                            .filter_map(|s| {
                                if let Segment::Literal(s, _) = s {
                                    Some(s.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if text != "1" && text != "2" {
                            return Err(self.error("unsupported file descriptor redirection"));
                        }
                    }
                    command.redirects.push(Redirect {
                        fd,
                        kind,
                        target,
                        span,
                    });
                    self.pos += 1;
                }
                _ => break,
            }
        }
        if command.words.is_empty() && command.redirects.is_empty() {
            return Err(self.error("expected command"));
        }
        command.span.end = self.tokens[self.pos - 1].span.end;
        if command.words.first().is_some_and(|word| matches!(word.segments.as_slice(), [Segment::Literal(name, false)] if ["set", "if", "then", "fi", "for", "while", "until", "case", "function", "eval", "exec"].contains(&name.as_str()))) {
            return Err(Diagnostic { span: command.span, message: "control syntax/set is not implemented".into(), incomplete: false });
        }
        Ok(command)
    }
    fn pipeline(&mut self) -> Result<Pipeline, Diagnostic> {
        let mut commands = vec![self.command()?];
        while self.peek() == Some(&Kind::Pipe) {
            self.pos += 1;
            self.newlines();
            commands.push(self.command()?);
            if commands.len() > MAX_STAGES {
                return Err(self.error("pipeline stage limit: 16"));
            }
        }
        Ok(Pipeline { commands })
    }
    fn list(&mut self) -> Result<List, Diagnostic> {
        let mut list = List::default();
        self.newlines();
        while self.peek().is_some() {
            let mut item = AndOr {
                pipelines: vec![(Condition::Always, self.pipeline()?)],
                background: false,
            };
            while let Some(op @ (Kind::And | Kind::Or)) = self.peek().cloned() {
                self.pos += 1;
                self.newlines();
                item.pipelines.push((
                    if op == Kind::And {
                        Condition::Success
                    } else {
                        Condition::Failure
                    },
                    self.pipeline()?,
                ));
            }
            match self.peek() {
                None => {}
                Some(Kind::Background) => {
                    item.background = true;
                    self.pos += 1;
                }
                Some(Kind::Semi | Kind::Newline) => {
                    self.pos += 1;
                }
                _ => return Err(self.error("unexpected token")),
            }
            list.items.push(item);
            self.newlines();
        }
        Ok(list)
    }
}
pub fn parse(text: &str) -> ParseResult {
    let result = (|| {
        if text.len() > MAX_SOURCE || text.chars().any(|c| c == '\0' || c == '\x1b') {
            return Err(Diagnostic {
                span: Span {
                    start: 0,
                    end: text.len(),
                },
                message: "source limit or unsupported control character".into(),
                incomplete: false,
            });
        }
        let tokens = Lexer {
            text,
            pos: 0,
            depth: 0,
        }
        .tokens(false)?;
        Parser {
            tokens,
            pos: 0,
            end: text.len(),
        }
        .list()
    })();
    match result {
        Ok(list) => ParseResult::Complete(list),
        Err(e) if e.incomplete => ParseResult::Incomplete(e),
        Err(e) => ParseResult::Error(e),
    }
}
