//! Quote-aware expansion. Only parameter/substitution results undergo IFS splitting.
use super::syntax::{Segment, Word};
use crate::{
    error::GameResult,
    terminal_io::Output,
    vfs::{domain, normalize},
    world::WorldState,
};

#[derive(Clone, Copy)]
struct Character {
    value: char,
    protected: bool,
    split: bool,
}
#[derive(Default)]
struct Field {
    chars: Vec<Character>,
    keep: bool,
}
#[derive(Default)]
pub struct Expansion {
    pub values: Vec<String>,
    pub side_output: Output,
    pub substitution_status: Option<i32>,
}

pub fn word(world: &mut WorldState, word: &Word, assignment: bool) -> GameResult<Expansion> {
    let mut expansion = Expansion::default();
    let mut fields = vec![Field::default()];
    let mut expanded_bytes = 0usize;
    let env = crate::terminal::virtual_env(world, &world.terminal.user);
    let ifs = env.get("IFS").map(String::as_str).unwrap_or(" \t\n");
    for (index, segment) in word.segments.iter().enumerate() {
        let (values, protected, split, at) = match segment {
            Segment::Literal(text, protected) => {
                let text = if index == 0 && !protected && (text == "~" || text.starts_with("~/")) {
                    format!(
                        "{}{}",
                        env.get("HOME")
                            .map(String::as_str)
                            .unwrap_or(crate::vfs::HOME),
                        &text[1..]
                    )
                } else {
                    text.clone()
                };
                (vec![text], *protected, false, false)
            }
            Segment::Parameter(name, protected) => {
                let value = match name.as_str() {
                    "?" => world.terminal.last_status.to_string(),
                    "$" => world.terminal.shell.pid.to_string(),
                    "!" => world
                        .terminal
                        .shell
                        .last_background
                        .map(|pid| pid.to_string())
                        .unwrap_or_default(),
                    "#" => world.terminal.shell.args.len().to_string(),
                    "0" => world.terminal.shell.name.clone(),
                    "@" if *protected && !assignment => {
                        let values = world.terminal.shell.args.clone();
                        expanded_bytes += values.iter().map(String::len).sum::<usize>();
                        if expanded_bytes > 1024 * 1024 || fields.len() + values.len() > 16384 {
                            return Err(domain("shell: expanded argument limit"));
                        }
                        if values.is_empty() {
                            continue;
                        }
                        for (i, value) in values.iter().enumerate() {
                            if i > 0 {
                                fields.push(Field::default());
                            }
                            let field = fields.last_mut().unwrap();
                            field.keep = true;
                            field.chars.extend(value.chars().map(|value| Character {
                                value,
                                protected: true,
                                split: false,
                            }));
                        }
                        continue;
                    }
                    "@" | "*" => world.terminal.shell.args.join(
                        &ifs.chars()
                            .next()
                            .map(|c| c.to_string())
                            .unwrap_or_default(),
                    ),
                    _ if name.chars().all(|c| c.is_ascii_digit()) => name
                        .parse::<usize>()
                        .ok()
                        .and_then(|i| i.checked_sub(1))
                        .and_then(|i| world.terminal.shell.args.get(i))
                        .cloned()
                        .unwrap_or_default(),
                    _ => env.get(name).cloned().unwrap_or_default(),
                };
                (vec![value], *protected, !protected && !assignment, false)
            }
            Segment::Substitution(list, protected) => {
                if world.terminal.shell_depth >= super::syntax::MAX_DEPTH {
                    return Err(domain("shell: maximum nesting depth exceeded"));
                }
                let original = world.terminal.clone();
                world.terminal.shell_depth += 1;
                world.terminal.shell.flow = None;
                world.terminal.io.stdout_tty = false;
                let result = super::control::capture(|| super::executor::execute_list(world, list));
                world.terminal = original;
                let output = result?;
                expansion.substitution_status = Some(output.status);
                expansion.side_output.stderr.push_str(&output.stderr);
                expansion
                    .side_output
                    .ordered
                    .extend(output.ordered.into_iter().filter(|(fd, _)| *fd == 2));
                (
                    vec![output.stdout.trim_end_matches('\n').to_string()],
                    *protected,
                    !protected && !assignment,
                    false,
                )
            }
        };
        for (i, value) in values.into_iter().enumerate() {
            expanded_bytes += value.len();
            if expanded_bytes > 1024 * 1024 {
                return Err(domain("shell: expanded argument limit"));
            }
            if value.chars().any(|c| matches!(c, '\0' | '\x1b')) {
                return Err(domain("shell: unsupported control character in expansion"));
            }
            if at && i > 0 {
                fields.push(Field::default());
            }
            let field = fields.last_mut().unwrap();
            field.keep |= protected || assignment;
            field.chars.extend(value.chars().map(|value| Character {
                value,
                protected,
                split,
            }));
        }
    }
    for field in fields {
        let fields = if assignment {
            vec![field]
        } else {
            split_field(field, ifs)
        };
        for field in fields {
            let text: String = field.chars.iter().map(|c| c.value).collect();
            if !assignment
                && field
                    .chars
                    .iter()
                    .any(|c| !c.protected && "*?[".contains(c.value))
            {
                expansion.values.extend(glob(world, &field.chars, &text)?);
            } else {
                expansion.values.push(text);
            }
            check_limit(&expansion.values)?;
        }
    }
    check_limit(&expansion.values)?;
    Ok(expansion)
}
pub fn check_limit(values: &[String]) -> GameResult<()> {
    if values.len() > 16384 || values.iter().map(String::len).sum::<usize>() > 1024 * 1024 {
        Err(domain("shell: expanded argument limit"))
    } else {
        Ok(())
    }
}
fn split_field(field: Field, ifs: &str) -> Vec<Field> {
    let is_delim = |c: &Character| c.split && ifs.contains(c.value);
    let is_white = |c: &Character| is_delim(c) && " \t\n".contains(c.value);
    let mut result = Vec::new();
    let mut current = Field {
        chars: Vec::new(),
        keep: field.keep,
    };
    let chars = field.chars;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if !is_delim(&c) {
            current.chars.push(c);
            i += 1;
            continue;
        }
        if is_white(&c) {
            while i < chars.len() && is_white(&chars[i]) {
                i += 1;
            }
            if i < chars.len() && is_delim(&chars[i]) {
                continue;
            }
            if !current.chars.is_empty() || current.keep {
                result.push(std::mem::take(&mut current));
            }
        } else {
            result.push(std::mem::take(&mut current));
            i += 1;
            while i < chars.len() && is_white(&chars[i]) {
                i += 1;
            }
        }
    }
    if !current.chars.is_empty() || current.keep {
        result.push(current);
    }
    result
}

fn glob(world: &WorldState, pattern: &[Character], original: &str) -> GameResult<Vec<String>> {
    if pattern.len() > 4096 {
        return Err(domain("shell: glob pattern limit: 4096 characters"));
    }
    let mut work_budget = 8_000_000usize;
    let absolute = original.starts_with('/');
    let mut paths = vec![if absolute {
        "/".to_string()
    } else {
        String::new()
    }];
    for component in pattern.split(|c| c.value == '/') {
        if component.is_empty() {
            continue;
        }
        let mut next = Vec::new();
        for parent in paths {
            if !component
                .iter()
                .any(|c| !c.protected && "*?[".contains(c.value))
            {
                let name: String = component.iter().map(|c| c.value).collect();
                let candidate = format!(
                    "{parent}{}{name}",
                    if parent.is_empty() || parent.ends_with('/') {
                        ""
                    } else {
                        "/"
                    }
                );
                if normalize(&candidate, &world.terminal.cwd)
                    .ok()
                    .and_then(|p| world.fs().ok()?.stat(&p, &world.terminal.user).ok())
                    .is_some()
                {
                    next.push(candidate);
                }
                continue;
            }
            let path = normalize(
                if parent.is_empty() { "." } else { &parent },
                &world.terminal.cwd,
            )?;
            let Ok(names) = world.fs()?.child_names(&path, &world.terminal.user) else {
                continue;
            };
            for name in names {
                if name.starts_with('.') && component[0].value != '.' {
                    continue;
                }
                work_budget = work_budget
                    .checked_sub(component.len().saturating_mul(name.len()))
                    .ok_or_else(|| domain("shell: glob work limit"))?;
                if matches(component, &name) {
                    next.push(format!(
                        "{parent}{}{name}",
                        if parent.is_empty() || parent.ends_with('/') {
                            ""
                        } else {
                            "/"
                        }
                    ));
                }
            }
        }
        paths = next;
        if paths.is_empty() {
            return Ok(vec![original.into()]);
        }
        check_limit(&paths)?;
    }
    if original.ends_with('/') {
        paths.retain(|p| {
            normalize(p, &world.terminal.cwd)
                .ok()
                .and_then(|p| world.fs().ok()?.stat(&p, &world.terminal.user).ok())
                .is_some_and(|n| n.kind == "directory")
        });
        for path in &mut paths {
            path.push('/');
        }
    }
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        paths.push(original.into());
    }
    Ok(paths)
}
fn matches(pattern: &[Character], text: &str) -> bool {
    let text: Vec<char> = text.chars().collect();
    let mut reachable = vec![false; text.len() + 1];
    reachable[0] = true;
    let mut i = 0;
    while i < pattern.len() {
        let p = pattern[i];
        if !p.protected && p.value == '*' {
            for j in 1..reachable.len() {
                reachable[j] |= reachable[j - 1];
            }
            i += 1;
            continue;
        }
        let class_end = if !p.protected && p.value == '[' {
            let start =
                i + 1 + usize::from(pattern.get(i + 1).is_some_and(|c| "!^".contains(c.value)));
            (start + 1..pattern.len()).find(|j| pattern[*j].value == ']' && !pattern[*j].protected)
        } else {
            None
        };
        for j in (1..reachable.len()).rev() {
            let c = text[j - 1];
            let accepted = if let Some(end) = class_end {
                let negated = pattern
                    .get(i + 1)
                    .is_some_and(|c| "!^".contains(c.value) && !c.protected);
                let mut k = i + 1 + usize::from(negated);
                let mut found = false;
                while k < end {
                    if k + 2 < end && pattern[k + 1].value == '-' && !pattern[k + 1].protected {
                        found |= pattern[k].value <= c && c <= pattern[k + 2].value;
                        k += 3;
                    } else {
                        found |= pattern[k].value == c;
                        k += 1;
                    }
                }
                found != negated
            } else {
                (!p.protected && p.value == '?') || p.value == c
            };
            reachable[j] = reachable[j - 1] && accepted;
        }
        reachable[0] = false;
        i = class_end.map_or(i + 1, |end| end + 1);
    }
    reachable[text.len()]
}
