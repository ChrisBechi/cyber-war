use crate::{
    error::GameResult,
    vfs::{domain, normalize, parent},
    world::WorldState,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NanoOptions {
    pub smart_home: bool,
    pub backup: bool,
    pub backup_dir: Option<String>,
    pub bold_text: bool,
    pub tabs_to_spaces: bool,
    pub new_buffer: bool,
    pub locking: bool,
    pub history_log: bool,
    pub ignore_rcfiles: bool,
    pub guide_stripe: Option<u16>,
    pub raw_sequences: bool,
    pub no_newlines: bool,
    pub trim_blanks: bool,
    pub no_convert: bool,
    pub book_style: bool,
    pub position_log: bool,
    pub quote_string: Option<String>,
    pub restricted: bool,
    pub softwrap: bool,
    pub tab_size: u16,
    pub quick_blank: bool,
    pub word_bounds: bool,
    pub word_chars: Option<String>,
    pub syntax: Option<String>,
    pub zap: bool,
    pub at_blanks: bool,
    pub break_long_lines: bool,
    pub constant_show: bool,
    pub rebind_delete: bool,
    pub empty_line: bool,
    pub rcfile: Option<String>,
    pub show_cursor: bool,
    pub auto_indent: bool,
    pub jumpy_scrolling: bool,
    pub cut_from_cursor: bool,
    pub line_numbers: bool,
    pub mouse: bool,
    pub no_read: bool,
    pub operating_dir: Option<String>,
    pub preserve: bool,
    pub indicator: bool,
    pub fill: Option<u16>,
    pub speller: Option<String>,
    pub unix: bool,
    pub view: bool,
    pub no_wrap: bool,
    pub no_help: bool,
    pub after_ends: bool,
    pub magic: bool,
    pub colon_parsing: bool,
    pub state_flags: bool,
    pub minibar: bool,
    pub zero: bool,
    pub solo_side_scroll: bool,
    pub modern_bindings: bool,
}

impl Default for NanoOptions {
    fn default() -> Self {
        Self {
            smart_home: false,
            backup: false,
            backup_dir: None,
            bold_text: false,
            tabs_to_spaces: false,
            new_buffer: false,
            locking: false,
            history_log: false,
            ignore_rcfiles: false,
            guide_stripe: None,
            raw_sequences: false,
            no_newlines: false,
            trim_blanks: false,
            no_convert: false,
            book_style: false,
            position_log: false,
            quote_string: None,
            restricted: false,
            softwrap: false,
            tab_size: 8,
            quick_blank: false,
            word_bounds: false,
            word_chars: None,
            syntax: None,
            zap: false,
            at_blanks: false,
            break_long_lines: false,
            constant_show: false,
            rebind_delete: false,
            empty_line: false,
            rcfile: None,
            show_cursor: false,
            auto_indent: false,
            jumpy_scrolling: false,
            cut_from_cursor: false,
            line_numbers: false,
            mouse: false,
            no_read: false,
            operating_dir: None,
            preserve: false,
            indicator: false,
            fill: None,
            speller: None,
            unix: false,
            view: false,
            no_wrap: false,
            no_help: false,
            after_ends: false,
            magic: false,
            colon_parsing: false,
            state_flags: false,
            minibar: false,
            zero: false,
            solo_side_scroll: false,
            modern_bindings: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NanoSession {
    pub path: String,
    pub display_name: String,
    pub actor: String,
    pub original_content: Option<String>,
    pub options: NanoOptions,
    pub starting_line: usize,
    pub starting_column: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NanoLaunch {
    pub path: String,
    pub display_name: String,
    pub content: String,
    pub expected_content: Option<String>,
    pub options: NanoOptions,
    pub starting_line: usize,
    pub starting_column: usize,
}

pub const VERSION: &str = "Cyber War nano compatibility editor 0.4.2 (partial GNU nano interface)";

pub fn command(world: &mut WorldState, args: &[String], actor: &str) -> GameResult<String> {
    let (mut options, line, column, files) = parse_args(args)?;
    if options.help_requested() {
        return Ok(help());
    }
    if options.version_requested() {
        return Ok(format!("{VERSION}\n"));
    }
    let unsupported = unsupported_options(&options);
    if !unsupported.is_empty() {
        return Err(domain(format!(
            "nano: options not implemented in this virtual editor: {}",
            unsupported.join(", ")
        )));
    }
    if files.len() > 1 {
        return Err(domain("nano: multiple buffers are not implemented"));
    }
    let file = files
        .first()
        .ok_or_else(|| domain("nano: missing file operand"))?;
    if let Some(root) = &options.operating_dir {
        let root = normalize(root, &world.terminal.cwd)?;
        world.fs()?.directory(&root, actor)?;
        options.operating_dir = Some(root);
    }
    let path = normalize(
        file,
        options
            .operating_dir
            .as_deref()
            .unwrap_or(&world.terminal.cwd),
    )?;
    if let Some(root) = options.operating_dir.as_deref() {
        let root = normalize(root, &world.terminal.cwd)?;
        if path != root && !path.starts_with(&format!("{root}/")) {
            return Err(domain("nano: file is outside the operating directory"));
        }
    }
    let original = match world.fs()?.read(&path, actor) {
        Ok(content) => Some(content),
        Err(error) if error.to_string().contains("no such file") && !options.view => {
            world.fs()?.directory(parent(&path), actor)?;
            None
        }
        Err(error) => return Err(error),
    };
    let content = if options.no_read {
        String::new()
    } else {
        original.clone().unwrap_or_default()
    };
    let available_lines =
        content.lines().count().max(1) + if content.ends_with('\n') { 1 } else { 0 };
    if line > 0 && line > available_lines {
        return Err(domain("nano: line number is beyond the end of the file"));
    }
    world.terminal.nano = Some(NanoSession {
        path,
        display_name: file.clone(),
        actor: actor.into(),
        original_content: original,
        options,
        starting_line: line,
        starting_column: column,
    });
    world.terminal.foreground = Some("nano".into());
    Ok(String::new())
}

impl NanoOptions {
    fn help_requested(&self) -> bool {
        self.syntax.as_deref() == Some("__help__")
    }

    fn version_requested(&self) -> bool {
        self.syntax.as_deref() == Some("__version__")
    }
}

fn unsupported_options(o: &NanoOptions) -> Vec<&'static str> {
    [
        (o.bold_text, "--boldtext"),
        (o.new_buffer, "--newbuffer/--multibuffer"),
        (o.locking, "--locking"),
        (o.history_log, "--historylog"),
        (o.guide_stripe.is_some(), "--guidestripe"),
        (o.raw_sequences, "--rawsequences"),
        (o.trim_blanks, "--trimblanks"),
        (o.no_convert, "--noconvert"),
        (o.book_style, "--bookstyle"),
        (o.position_log, "--positionlog"),
        (o.quote_string.is_some(), "--quotestr"),
        (o.quick_blank, "--quickblank"),
        (o.word_bounds, "--wordbounds"),
        (o.word_chars.is_some(), "--wordchars"),
        (o.syntax.is_some(), "--syntax/--listsyntaxes"),
        (o.zap, "--zap"),
        (o.at_blanks, "--atblanks"),
        (o.break_long_lines, "--breaklonglines"),
        (o.rebind_delete, "--rebinddelete"),
        (o.empty_line, "--emptyline"),
        (o.rcfile.is_some(), "--rcfile"),
        (o.show_cursor, "--showcursor"),
        (o.jumpy_scrolling, "--jumpyscrolling"),
        (o.mouse, "--mouse"),
        (o.preserve, "--preserve"),
        (o.indicator, "--indicator"),
        (o.fill.is_some(), "--fill"),
        (o.speller.is_some(), "--speller"),
        (o.unix, "--unix"),
        (o.after_ends, "--afterends"),
        (o.magic, "--magic"),
        (o.colon_parsing, "--colonparsing"),
        (o.state_flags, "--stateflags"),
        (o.minibar, "--minibar"),
        (o.zero, "--zero"),
        (o.solo_side_scroll, "--solosidescroll"),
        (o.modern_bindings, "--modernbindings"),
    ]
    .into_iter()
    .filter_map(|(enabled, name)| enabled.then_some(name))
    .collect()
}

pub fn validate_access(session: &NanoSession, path: &str) -> GameResult<()> {
    if session.options.restricted && path != session.path {
        return Err(domain(
            "nano: restricted mode allows only the original file",
        ));
    }
    if let Some(root) = &session.options.operating_dir {
        if path != root && !path.starts_with(&format!("{}/", root.trim_end_matches('/'))) {
            return Err(domain("nano: file is outside the operating directory"));
        }
    }
    Ok(())
}

fn help() -> String {
    format!(
        "{}\nUsage: nano [OPTIONS] [+LINE,COLUMN] FILE\n\n\
Edit one UTF-8 file in the virtual Cyber War filesystem.\n\
  -h, --help                 display this help and exit\n\
  -V, --version              display version information and exit\n\
  -v, --view                 view file (read-only)\n\
  -w, --nowrap               do not wrap long lines\n\
  -S, --softwrap             display soft-wrapped lines\n\
  -T, --tabsize=NUMBER       set the tab size\n\
  -E, --tabstospaces         convert typed tabs to spaces\n\
  -l, --linenumbers           show line numbers\n\
  -i, --autoindent            automatically indent new lines\n\
  -n, --noread                do not read the file into the buffer\n\
  -R, --restricted            restricted mode\n\
  -x, --nohelp                hide the two help lines\n\
  -c, --constantshow          constantly show cursor position\n\
  -A, --smarthome             smart Home navigation\n\
  -B, --backup               save the previous version as FILE~\n\
  -C, --backupdir=DIR        numbered backups when -B is active\n\
  -L, --nonewlines           do not add a missing final newline\n\
  -k, --cutfromcursor        cut from cursor to end of line\n\
  -o, --operatingdir=DIR     restrict file access to this directory\n\
  -I, --ignorercfiles        no rcfiles are loaded in this simulator\n\n\
This is a partial compatibility editor, not GNU nano itself.\n\
Options for mouse, syntax, nanorc, locking, multiple buffers, external tools,\n\
hard wrapping and other unimplemented features are rejected explicitly.\n",
        VERSION
    )
}

fn parse_args(args: &[String]) -> GameResult<(NanoOptions, usize, usize, Vec<String>)> {
    let mut options = NanoOptions::default();
    let mut files = Vec::new();
    let mut line = 0;
    let mut column = 0;
    let mut options_done = false;
    let mut index = 0;
    while index < args.len() {
        let value = &args[index];
        if options_done || !value.starts_with('-') || value == "-" {
            if !options_done && value.starts_with('+') {
                (line, column) = parse_position(value)?;
            } else {
                files.push(value.clone());
            }
            index += 1;
            continue;
        }
        if value == "--" {
            options_done = true;
            index += 1;
            continue;
        }
        if let Some(long) = value.strip_prefix("--") {
            let (name, inline) = long.split_once('=').unwrap_or((long, ""));
            match name {
                "help" => options.syntax = Some("__help__".into()),
                "version" => options.syntax = Some("__version__".into()),
                "listsyntaxes" => options.syntax = Some("__list__".into()),
                "smarthome" => options.smart_home = true,
                "backup" => options.backup = true,
                "backupdir" => {
                    options.backup_dir = Some(value_arg(name, inline, args, &mut index)?)
                }
                "boldtext" => options.bold_text = true,
                "tabstospaces" => options.tabs_to_spaces = true,
                "newbuffer" | "multibuffer" => options.new_buffer = true,
                "locking" => options.locking = true,
                "historylog" => options.history_log = true,
                "ignorercfiles" => options.ignore_rcfiles = true,
                "guidestripe" => {
                    options.guide_stripe = Some(number_arg(name, inline, args, &mut index)?)
                }
                "rawsequences" => options.raw_sequences = true,
                "nonewlines" => options.no_newlines = true,
                "trimblanks" => options.trim_blanks = true,
                "noconvert" => options.no_convert = true,
                "bookstyle" => options.book_style = true,
                "positionlog" => options.position_log = true,
                "quotestr" => {
                    options.quote_string = Some(value_arg(name, inline, args, &mut index)?)
                }
                "restricted" => options.restricted = true,
                "softwrap" => options.softwrap = true,
                "tabsize" => options.tab_size = number_arg(name, inline, args, &mut index)?,
                "quickblank" => options.quick_blank = true,
                "wordbounds" => options.word_bounds = true,
                "wordchars" => {
                    options.word_chars = Some(value_arg(name, inline, args, &mut index)?)
                }
                "syntax" => options.syntax = Some(value_arg(name, inline, args, &mut index)?),
                "zap" => options.zap = true,
                "atblanks" => options.at_blanks = true,
                "breaklonglines" => options.break_long_lines = true,
                "constantshow" => options.constant_show = true,
                "rebinddelete" => options.rebind_delete = true,
                "emptyline" => options.empty_line = true,
                "rcfile" => options.rcfile = Some(value_arg(name, inline, args, &mut index)?),
                "showcursor" => options.show_cursor = true,
                "autoindent" => options.auto_indent = true,
                "jumpyscrolling" => options.jumpy_scrolling = true,
                "cutfromcursor" => options.cut_from_cursor = true,
                "linenumbers" => options.line_numbers = true,
                "mouse" => options.mouse = true,
                "noread" => options.no_read = true,
                "operatingdir" => {
                    options.operating_dir = Some(value_arg(name, inline, args, &mut index)?)
                }
                "preserve" => options.preserve = true,
                "indicator" => options.indicator = true,
                "fill" => options.fill = Some(number_arg(name, inline, args, &mut index)?),
                "speller" => options.speller = Some(value_arg(name, inline, args, &mut index)?),
                "unix" => options.unix = true,
                "view" => options.view = true,
                "nowrap" => options.no_wrap = true,
                "nohelp" => options.no_help = true,
                "afterends" => options.after_ends = true,
                "magic" => options.magic = true,
                "colonparsing" => options.colon_parsing = true,
                "stateflags" => options.state_flags = true,
                "minibar" => options.minibar = true,
                "zero" => options.zero = true,
                "solosidescroll" => options.solo_side_scroll = true,
                "modernbindings" => options.modern_bindings = true,
                _ => return Err(invalid_option(value)),
            }
            index += 1;
            continue;
        }
        let short = value.strip_prefix('-').unwrap_or_default();
        if short.is_empty() {
            return Err(invalid_option(value));
        }
        let mut chars = short.chars().peekable();
        while let Some(flag) = chars.next() {
            let needs_value = matches!(
                flag,
                'C' | 'J' | 'Q' | 'T' | 'X' | 'Y' | 'f' | 'o' | 'r' | 's'
            );
            if needs_value {
                let attached: String = chars.by_ref().collect();
                let value = if !attached.is_empty() {
                    attached
                } else {
                    index += 1;
                    args.get(index).cloned().ok_or_else(|| missing_arg(flag))?
                };
                match flag {
                    'C' => options.backup_dir = Some(value),
                    'J' => options.guide_stripe = Some(parse_number("guidestripe", &value)?),
                    'Q' => options.quote_string = Some(value),
                    'T' => options.tab_size = parse_number("tabsize", &value)?,
                    'X' => options.word_chars = Some(value),
                    'Y' => options.syntax = Some(value),
                    'f' => options.rcfile = Some(value),
                    'o' => options.operating_dir = Some(value),
                    'r' => options.fill = Some(parse_number("fill", &value)?),
                    's' => options.speller = Some(value),
                    _ => unreachable!(),
                }
                break;
            }
            match flag {
                'A' => options.smart_home = true,
                'B' => options.backup = true,
                'D' => options.bold_text = true,
                'E' => options.tabs_to_spaces = true,
                'F' => options.new_buffer = true,
                'G' => options.locking = true,
                'H' => options.history_log = true,
                'I' => options.ignore_rcfiles = true,
                'K' => options.raw_sequences = true,
                'L' => options.no_newlines = true,
                'M' => options.trim_blanks = true,
                'N' => options.no_convert = true,
                'O' => options.book_style = true,
                'P' => options.position_log = true,
                'R' => options.restricted = true,
                'S' => options.softwrap = true,
                'U' => options.quick_blank = true,
                'V' => options.syntax = Some("__version__".into()),
                'W' => options.word_bounds = true,
                'Z' => options.zap = true,
                'z' => options.syntax = Some("__list__".into()),
                'a' => options.at_blanks = true,
                'b' => options.break_long_lines = true,
                'c' => options.constant_show = true,
                'd' => options.rebind_delete = true,
                'e' => options.empty_line = true,
                'g' => options.show_cursor = true,
                'h' => options.syntax = Some("__help__".into()),
                'i' => options.auto_indent = true,
                'j' => options.jumpy_scrolling = true,
                'k' => options.cut_from_cursor = true,
                'l' => options.line_numbers = true,
                'm' => options.mouse = true,
                'n' => options.no_read = true,
                'p' => options.preserve = true,
                'q' => options.indicator = true,
                'u' => options.unix = true,
                'v' => options.view = true,
                'w' => options.no_wrap = true,
                'x' => options.no_help = true,
                'y' => options.after_ends = true,
                '!' => options.magic = true,
                '@' => options.colon_parsing = true,
                '%' => options.state_flags = true,
                '_' => options.minibar = true,
                '0' => options.zero = true,
                '1' => options.solo_side_scroll = true,
                '/' => options.modern_bindings = true,
                _ => return Err(invalid_option(value)),
            }
        }
        index += 1;
    }
    Ok((options, line, column, files))
}

fn parse_position(value: &str) -> GameResult<(usize, usize)> {
    let value = value.strip_prefix('+').unwrap_or(value);
    let (line, column) = value.split_once(',').unwrap_or((value, "1"));
    let line = line
        .parse::<usize>()
        .map_err(|_| domain("nano: invalid line number"))?;
    let column = column
        .parse::<usize>()
        .map_err(|_| domain("nano: invalid column number"))?;
    if line == 0 || column == 0 {
        return Err(domain("nano: line and column numbers start at 1"));
    }
    Ok((line, column))
}

fn value_arg(name: &str, inline: &str, args: &[String], index: &mut usize) -> GameResult<String> {
    if !inline.is_empty() {
        return Ok(inline.into());
    }
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| domain(format!("nano: option '--{name}' requires an argument")))
}

fn number_arg(name: &str, inline: &str, args: &[String], index: &mut usize) -> GameResult<u16> {
    parse_number(name, &value_arg(name, inline, args, index)?)
}

fn parse_number(name: &str, value: &str) -> GameResult<u16> {
    let number = value
        .parse::<u16>()
        .map_err(|_| domain(format!("nano: invalid number for {name}: {value}")))?;
    if number == 0 {
        return Err(domain(format!("nano: {name} must be greater than zero")));
    }
    Ok(number)
}

fn invalid_option(value: &str) -> crate::error::GameError {
    domain(format!("nano: unrecognized option '{value}'"))
}

fn missing_arg(flag: char) -> crate::error::GameError {
    domain(format!("nano: option '-{flag}' requires an argument"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{terminal, world::WorldState};

    fn world() -> WorldState {
        WorldState::new("neo", "pc").expect("world")
    }

    fn args(value: &str) -> Vec<String> {
        terminal::tokenize(value).expect("tokens")[1..].to_vec()
    }

    #[test]
    fn opens_file_and_preserves_options() {
        let mut world = world();
        world
            .vfs
            .write("/home/kali/note.txt", "hello\n", "kali")
            .expect("file");
        let output = command(&mut world, &args("nano -l -T4 +2,3 note.txt"), "kali").expect("open");
        assert!(output.is_empty());
        let session = world.terminal.nano.as_ref().expect("session");
        assert_eq!(session.starting_line, 2);
        assert_eq!(session.starting_column, 3);
        assert!(session.options.line_numbers);
        assert_eq!(session.options.tab_size, 4);
    }

    #[test]
    fn accepts_long_options_and_rejects_unknown_options() {
        let mut world = world();
        command(
            &mut world,
            &args("nano --softwrap --tabstospaces --linenumbers note.txt"),
            "kali",
        )
        .expect("open new");
        let session = world.terminal.nano.as_ref().expect("session");
        assert!(session.options.softwrap);
        assert!(session.options.tabs_to_spaces);
        assert!(session.options.line_numbers);
        assert!(
            command(&mut world, &args("nano --guidestripe=80 note.txt"), "kali")
                .unwrap_err()
                .to_string()
                .contains("not implemented")
        );
        assert!(command(&mut world, &args("nano --not-a-real-flag note.txt"), "kali").is_err());
    }

    #[test]
    fn help_and_version_are_noninteractive() {
        let mut world = world();
        assert!(command(&mut world, &args("nano --help"), "kali")
            .expect("help")
            .contains("Usage: nano"));
        assert_eq!(
            command(&mut world, &args("nano -V"), "kali").expect("version"),
            format!("{VERSION}\n")
        );
        assert!(world.terminal.nano.is_none());
    }
}
