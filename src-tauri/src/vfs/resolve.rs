use super::*;
use std::collections::VecDeque;
#[derive(Clone, Copy)]
pub enum Follow {
    Yes,
    No,
}
pub fn validate_path(path: &str) -> GameResult<()> {
    if path.is_empty() {
        return Err(error(Errno::NotFound));
    }
    if path.contains('\0') {
        return Err(error(Errno::Invalid));
    }
    if path.len() > 4096 || path.split('/').any(|p| p.len() > 255) {
        return Err(error(Errno::NameTooLong));
    }
    Ok(())
}
impl VirtualFileSystem {
    pub fn resolve(&self, path: &str, actor: &str, follow: Follow) -> GameResult<String> {
        self.resolve_missing(path, actor, follow, false)
    }
    pub(crate) fn resolve_missing(
        &self,
        path: &str,
        actor: &str,
        follow: Follow,
        missing: bool,
    ) -> GameResult<String> {
        validate_path(path)?;
        let mut pending: VecDeque<String> = path
            .split('/')
            .filter(|p| !p.is_empty())
            .map(String::from)
            .collect();
        let mut current = String::from("/");
        let mut followed = 0;
        let mut directory_required = path.ends_with('/');
        while let Some(part) = pending.pop_front() {
            let dir = self
                .nodes
                .get(&current)
                .ok_or_else(|| error(Errno::NotFound))?;
            if dir.kind != "directory" {
                return Err(error(Errno::NotDirectory));
            }
            if !self.allowed(dir, actor, 1) {
                return Err(error(Errno::Access));
            }
            if part == "." {
                continue;
            }
            if part == ".." {
                current = parent(&current).to_owned();
                continue;
            }
            let next = format!("{}/{}", current.trim_end_matches('/'), part);
            let Some(node) = self.nodes.get(&next) else {
                if missing && pending.is_empty() && !directory_required {
                    return Ok(next);
                }
                return Err(error(Errno::NotFound));
            };
            if node.kind == "symlink"
                && (!pending.is_empty() || directory_required || matches!(follow, Follow::Yes))
            {
                followed += 1;
                if followed > 40 {
                    return Err(error(Errno::Loop));
                }
                validate_path(&node.content)?;
                if pending.is_empty() && node.content.ends_with('/') {
                    directory_required = true;
                }
                if node.content.starts_with('/') {
                    current = "/".into();
                }
                for component in node.content.split('/').filter(|p| !p.is_empty()).rev() {
                    pending.push_front(component.into());
                }
                if pending.iter().map(|s| s.len() + 1).sum::<usize>() + current.len() > 4096 {
                    return Err(error(Errno::NameTooLong));
                }
            } else {
                current = next;
            }
        }
        let node = self
            .nodes
            .get(&current)
            .ok_or_else(|| error(Errno::NotFound))?;
        if directory_required && node.kind != "directory" {
            return Err(error(Errno::NotDirectory));
        }
        Ok(current)
    }
    pub fn stat(&self, path: &str, actor: &str) -> GameResult<&VfsNode> {
        let resolved = self.resolve(path, actor, Follow::Yes)?;
        self.nodes
            .get(&resolved)
            .ok_or_else(|| error(Errno::NotFound))
    }
    pub fn lstat(&self, path: &str, actor: &str) -> GameResult<&VfsNode> {
        let resolved = self.resolve(path, actor, Follow::No)?;
        self.nodes
            .get(&resolved)
            .ok_or_else(|| error(Errno::NotFound))
    }
    pub fn readlink(&self, path: &str, actor: &str) -> GameResult<String> {
        let node = self.lstat(path, actor)?;
        if node.kind != "symlink" {
            return Err(error(Errno::Invalid));
        }
        Ok(node.content.clone())
    }
    pub fn path_exists(&self, path: &str, actor: &str) -> GameResult<bool> {
        match self.lstat(path, actor) {
            Ok(_) => Ok(true),
            Err(GameError::Vfs(Errno::NotFound)) => Ok(false),
            Err(e) => Err(e),
        }
    }
    /// Extraction has a stricter boundary than ordinary POSIX traversal.
    pub fn reject_symlink_components(&self, path: &str, actor: &str) -> GameResult<()> {
        validate_path(path)?;
        let mut current = String::new();
        for part in path.split('/').filter(|s| !s.is_empty()) {
            if part == "." || part == ".." {
                return Err(error(Errno::Invalid));
            }
            current.push('/');
            current.push_str(part);
            if let Some(node) = self.nodes.get(&current) {
                if node.kind == "symlink" {
                    return Err(domain("archive: symlink traversal is not allowed"));
                }
                self.lstat(&current, actor)?;
            }
        }
        Ok(())
    }
}
