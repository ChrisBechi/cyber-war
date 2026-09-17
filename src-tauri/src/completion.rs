use crate::{
    error::GameResult,
    vfs::{domain, normalize},
    world::WorldState,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    pub line: String,
    /// Unicode scalar offset, matching the frontend IPC contract.
    pub cursor: usize,
    pub candidates: Vec<String>,
}

struct Word {
    start: usize,
    end: usize,
    value: String,
}

/// Tolerates incomplete quotes while the player is still editing a command.
fn words(chars: &[char]) -> Vec<Word> {
    let mut result = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index].is_whitespace() {
            index += 1;
            continue;
        }
        let start = index;
        let mut quote = None;
        let mut value = String::new();
        while index < chars.len() {
            let c = chars[index];
            if let Some(q) = quote {
                if c == q {
                    quote = None;
                } else {
                    value.push(c);
                }
            } else if c == '\'' || c == '"' {
                quote = Some(c);
            } else if c.is_whitespace() {
                break;
            } else if c == '>' {
                if index > start {
                    break;
                }
                value.push(c);
                index += 1;
                if chars.get(index) == Some(&'>') {
                    value.push('>');
                    index += 1;
                }
                break;
            } else {
                value.push(c);
            }
            index += 1;
        }
        result.push(Word {
            start,
            end: index,
            value,
        });
    }
    result
}

fn quoted(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_alphanumeric() || "_./-~".contains(c))
    {
        return value.into();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn common_prefix(candidates: &[(String, bool)]) -> String {
    let Some((first, _)) = candidates.first() else {
        return String::new();
    };
    let mut common: Vec<_> = first.chars().collect();
    for (value, _) in candidates.iter().skip(1) {
        let count = common
            .iter()
            .zip(value.chars())
            .take_while(|(a, b)| **a == *b)
            .count();
        common.truncate(count);
    }
    common.into_iter().collect()
}

pub fn complete(world: &WorldState, line: &str, cursor: usize) -> GameResult<Completion> {
    if line.len() > 8192 || line.chars().any(char::is_control) {
        return Err(domain("invalid completion input"));
    }
    let chars: Vec<_> = line.chars().collect();
    if cursor > chars.len() {
        return Err(domain("cursor outside command"));
    }
    let tokens = words(&chars);
    let active = tokens
        .iter()
        .position(|w| w.start <= cursor && cursor <= w.end);
    let index = active.unwrap_or_else(|| tokens.iter().take_while(|w| w.end < cursor).count());
    let start = active.map(|i| tokens[i].start).unwrap_or(cursor);
    let end = active.map(|i| tokens[i].end).unwrap_or(cursor);
    let prefix = words(&chars[start..cursor])
        .first()
        .map(|w| w.value.clone())
        .unwrap_or_default();
    let command_position =
        index == 0 || (index == 1 && tokens.first().is_some_and(|w| w.value == "sudo"));
    let command = tokens.first().map(|w| w.value.as_str()).unwrap_or("");
    let mut candidates: Vec<(String, bool)> = if command_position {
        crate::command_registry::NAMES
            .iter()
            .map(String::as_str)
            .filter(|c| c.starts_with(&prefix))
            .filter(|c| crate::command_registry::available(world, c))
            .map(|c| (c.into(), false))
            .collect()
    } else if tokens
        .iter()
        .any(|w| w.value == "apt" || w.value == "apt-get")
        && index > 1
    {
        let removing = tokens
            .iter()
            .any(|w| ["remove", "purge", "reinstall"].contains(&w.value.as_str()));
        let names = if removing {
            world.packages.installed.keys().cloned().collect::<Vec<_>>()
        } else {
            crate::packages::resolver::candidates(&world.packages)
                .iter()
                .map(|e| e.package.name.clone())
                .collect()
        };
        names
            .into_iter()
            .filter(|n| n.starts_with(&prefix))
            .map(|n| (n, false))
            .collect()
    } else {
        if !prefix.is_empty() {
            normalize(&prefix, &world.terminal.cwd)?;
        }
        let (directory, name) = prefix
            .rsplit_once('/')
            .map(|(p, n)| (format!("{p}/"), n))
            .unwrap_or((String::new(), prefix.as_str()));
        let expanded = if directory.starts_with("~/")
            && !matches!(chars.get(start), Some('\'' | '"' | '\\'))
        {
            let env = crate::terminal::virtual_env(world, &world.terminal.user);
            format!(
                "{}{}",
                env.get("HOME")
                    .map(String::as_str)
                    .unwrap_or(crate::vfs::HOME),
                &directory[1..]
            )
        } else {
            directory.clone()
        };
        let path = normalize(
            if expanded.is_empty() { "." } else { &expanded },
            &world.terminal.cwd,
        )?;
        // Permission failures reveal no names. Completion never touches the host filesystem.
        world
            .fs()?
            .list(&path, &world.terminal.user)
            .unwrap_or_default()
            .into_iter()
            .filter(|n| {
                n.name.starts_with(name)
                    && (!n.name.starts_with('.') || name.starts_with('.'))
                    && (command != "cd" || n.kind == "directory")
            })
            .map(|n| {
                let dir = n.kind == "directory";
                (
                    format!(
                        "{directory}{}{suffix}",
                        n.name,
                        suffix = if dir { "/" } else { "" }
                    ),
                    dir,
                )
            })
            .collect()
    };
    if command_position {
        candidates.extend(
            crate::packages::executables::names(world, &world.terminal.user)
                .into_iter()
                .filter(|n| n.starts_with(&prefix))
                .map(|n| (n, false)),
        );
    }
    candidates.sort();
    candidates.dedup();
    let mut result = Completion {
        line: line.into(),
        cursor,
        candidates: candidates.iter().map(|(s, _)| s.clone()).collect(),
    };
    let value = if candidates.len() == 1 {
        candidates[0].0.clone()
    } else {
        common_prefix(&candidates)
    };
    if candidates.is_empty()
        || (candidates.len() > 1 && value.chars().count() <= prefix.chars().count())
    {
        return Ok(result);
    }
    let mut replacement = quoted(&value);
    if candidates.len() == 1
        && !candidates[0].1
        && !chars.get(end).is_some_and(|c| c.is_whitespace())
    {
        replacement.push(' ');
    }
    result.cursor = start + replacement.chars().count();
    result.line = chars[..start].iter().collect::<String>()
        + &replacement
        + &chars[end..].iter().collect::<String>();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> WorldState {
        WorldState::new("neo", "pc").expect("world")
    }
    fn tab(w: &WorldState, line: &str) -> Completion {
        complete(w, line, line.chars().count()).expect("completion")
    }
    #[test]
    fn commands_paths_quotes_and_cursor_in_middle() {
        let mut w = world();
        assert_eq!(tab(&w, "mkd").line, "mkdir ");
        // The default catalog adds commands beginning with `cho`.
        assert_eq!(tab(&w, "sudo chow").line, "sudo chown ");
        assert!(tab(&w, "sudo cho").candidates.len() > 1);
        assert_eq!(tab(&w, "cd Doc").line, "cd Documents/");
        assert_eq!(tab(&w, "cat ~/Doc").line, "cat ~/Documents/");
        assert_eq!(tab(&w, "cat /etc/os-r").line, "cat /etc/os-release ");
        w.vfs
            .mkdir("/home/kali/Meus arquivos", "kali")
            .expect("dir");
        w.vfs
            .write("/home/kali/Meus arquivos/ação.txt", "ok", "kali")
            .expect("file");
        assert_eq!(tab(&w, "cd 'Meus ar").line, "cd 'Meus arquivos/'");
        let c = tab(&w, "cat 'Meus arquivos/aç");
        assert_eq!(c.line, "cat 'Meus arquivos/ação.txt' ");
        assert_eq!(crate::terminal::execute(&mut w, &c.line).stdout, "ok");
        let mid = complete(&w, "cat Doc notes.txt", 7).expect("middle");
        assert_eq!(mid.line, "cat Documents/ notes.txt");
        assert_eq!(mid.cursor, 14);
        assert_eq!(
            complete(&w, "cd Documents/", 6).expect("inside token").line,
            "cd Documents/"
        );
        assert_eq!(tab(&w, "echo hello>Doc").line, "echo hello>Documents/");
    }
    #[test]
    fn ambiguity_permissions_missing_paths_and_remote_session() {
        let mut w = world();
        let ambiguous = tab(&w, "cd D");
        assert_eq!(ambiguous.line, "cd D");
        assert_eq!(
            ambiguous.candidates,
            vec!["Desktop/", "Documents/", "Downloads/"]
        );
        w.vfs.mkdir("/home/kali/project-one", "kali").expect("dir");
        w.vfs.mkdir("/home/kali/project-two", "kali").expect("dir");
        assert_eq!(tab(&w, "cd project-").candidates.len(), 2);
        w.vfs.seed("/root/secret.txt", "file", "private", "root");
        for line in ["cat /root/s", "cat /missing/f", "cd unknown"] {
            assert!(tab(&w, line).candidates.is_empty());
        }
        assert!(complete(&w, "cat C:\\Windows\\", 15)
            .unwrap()
            .candidates
            .is_empty());
        assert!(complete(&w, "cd", 3).is_err());
        assert_eq!(
            crate::terminal::execute(&mut w, "ssh vex@vex.local lab-only").exit_code,
            0
        );
        w.fs_mut()
            .expect("remote")
            .write("/home/kali/remote-only.txt", "remote", "vex")
            .expect("write");
        assert_eq!(tab(&w, "cat remote-").line, "cat remote-only.txt ");
        assert!(!w.vfs.nodes.contains_key("/home/kali/remote-only.txt"));
    }
}
