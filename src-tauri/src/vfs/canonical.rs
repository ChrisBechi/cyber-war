//! Canonical names are distinct from syscall lookup: missing-mode may describe
//! nonexistent paths, and acyclic link chains are not limited to 40 links.
use super::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CanonicalMode {
    Existing,
    AllButLast,
    Missing,
}

pub fn path_prefix(base: &str, path: &str) -> bool {
    base == "/"
        || path == base
        || path
            .strip_prefix(base)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Relative spelling between canonical POSIX names, independent of host OS.
pub fn relative_path(path: &str, base: &str) -> String {
    let target: Vec<_> = path.split('/').filter(|s| !s.is_empty()).collect();
    let base: Vec<_> = base.split('/').filter(|s| !s.is_empty()).collect();
    let shared = target.iter().zip(&base).take_while(|(a, b)| a == b).count();
    let result = std::iter::repeat_n("..", base.len() - shared)
        .chain(target[shared..].iter().copied())
        .collect::<Vec<_>>()
        .join("/");
    if result.is_empty() {
        ".".into()
    } else {
        result
    }
}

impl VirtualFileSystem {
    pub fn canonicalize(
        &self,
        path: &str,
        actor: &str,
        mode: CanonicalMode,
        no_links: bool,
    ) -> GameResult<String> {
        if path.is_empty() {
            return Err(error(Errno::NotFound));
        }
        if path.contains('\0') {
            return Err(error(Errno::Invalid));
        }
        if path.len() > 4096 {
            return Err(error(Errno::NameTooLong));
        }
        let mut pending: VecDeque<(String, bool)> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| (s.into(), false))
            .collect();
        let mut current = String::from("/");
        let mut active = BTreeSet::new();
        let mut trailing = path.ends_with('/');
        let mut traversals = 0;
        while let Some((part, end_link)) = pending.pop_front() {
            if end_link {
                active.remove(&part);
                continue;
            }
            traversals += 1;
            if traversals > 10000 {
                return Err(error(Errno::Loop));
            }
            if part == "." {
                continue;
            }
            if part == ".." {
                current = parent(&current).into();
                continue;
            }
            let next = format!("{}/{}", current.trim_end_matches('/'), part);
            if next.len() > 4096 {
                return Err(error(Errno::NameTooLong));
            }
            // Lexical traversal can defer existence checks until the final
            // component. A following '..' (or only trailing dots/slashes)
            // still requires a directory before discarding this component.
            if no_links
                && pending
                    .iter()
                    .filter(|(_, end)| !end)
                    .find(|(s, _)| s != ".")
                    .is_some_and(|(s, _)| s != "..")
            {
                current = next;
                continue;
            }
            // No-links preserves spelling but still tests existence through
            // links unless missing-mode explicitly suppresses filesystem checks.
            let node = if no_links && mode != CanonicalMode::Missing {
                self.stat(&next, actor)
            } else {
                self.lstat(&next, actor)
            };
            match node {
                Ok(node) if node.kind == "symlink" && !no_links => {
                    // Active expansions detect recursive targets, while a later
                    // independent occurrence of the same link remains valid.
                    if !active.insert(next.clone()) {
                        if mode == CanonicalMode::Missing {
                            current = next;
                            continue;
                        }
                        return Err(error(Errno::Loop));
                    }
                    if !pending.iter().any(|(_, end)| !end) && node.content.ends_with('/') {
                        trailing = true;
                    }
                    pending.push_front((next.clone(), true));
                    if node.content.starts_with('/') {
                        current = "/".into();
                    }
                    for p in node.content.split('/').filter(|s| !s.is_empty()).rev() {
                        pending.push_front((p.into(), false));
                    }
                    if pending.iter().map(|(s, _)| s.len() + 1).sum::<usize>() > 65536 {
                        return Err(error(Errno::NameTooLong));
                    }
                }
                result => {
                    let more = pending.iter().any(|(_, end)| !end);
                    let needs_dir = more || trailing;
                    if mode != CanonicalMode::Missing {
                        match result {
                            Ok(n) => {
                                if needs_dir && n.kind != "directory" {
                                    return Err(error(Errno::NotDirectory));
                                }
                            }
                            Err(GameError::Vfs(Errno::NotFound))
                                if mode == CanonicalMode::AllButLast && !more => {}
                            Err(e) => return Err(e),
                        }
                    }
                    current = next;
                }
            }
        }
        Ok(current)
    }
}
