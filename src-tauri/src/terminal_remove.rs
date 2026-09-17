//! Permanent removal in the virtual filesystem, distinct from desktop trash.
use crate::{
    error::GameResult,
    terminal_io::{error_reason, options, Options, Output},
    vfs::{domain, normalize, VirtualFileSystem, HOME},
    world::WorldState,
};

struct Remove<'a> {
    actor: &'a str,
    opts: Options,
    output: Output,
}
impl Remove<'_> {
    fn error(&mut self, label: &str, error: impl std::fmt::Display) {
        let message = format!("rm: cannot remove '{label}': {}\n", error_reason(error));
        self.output.stderr.push_str(&message);
        self.output.ordered.push((2, message));
        self.output.status = 1;
    }
    fn entry(&mut self, fs: &mut VirtualFileSystem, path: &str, label: &str, depth: usize) {
        if path == "/" || VirtualFileSystem::is_trash_container(path) {
            self.error(label, "virtual root and trash infrastructure are protected");
            return;
        }
        match fs.path_exists(path, self.actor) {
            Ok(false) => {
                if !self.opts.has('f') {
                    self.error(label, "No such file or directory");
                }
                return;
            }
            Err(error) => {
                self.error(label, error);
                return;
            }
            Ok(true) => {}
        }
        let node = match fs.lstat(path, self.actor) {
            Ok(node) => node,
            Err(error) => {
                self.error(label, error);
                return;
            }
        };
        let canonical = node.id.clone();
        let path = canonical.as_str();
        let directory = node.kind == "directory";
        if label.ends_with('/') && !directory {
            self.error(label, "Not a directory");
            return;
        }
        let recursive = self.opts.has('r') || self.opts.has('R');
        if directory && !recursive && !self.opts.has('d') {
            self.error(label, "Is a directory");
            return;
        }
        if directory && recursive {
            if depth >= 256 {
                self.error(label, "virtual traversal depth limit reached");
                return;
            }
            let children = match fs.list(path, self.actor) {
                Ok(children) => children,
                Err(error) => {
                    self.error(label, error);
                    return;
                }
            };
            for child in children {
                self.entry(
                    fs,
                    &child.id,
                    &format!("{}/{}", label.trim_end_matches('/'), child.name),
                    depth + 1,
                );
            }
            // Failed children stay in place. Their diagnostics already explain
            // why the parent cannot be removed, without duplicate parent errors.
            if fs.nodes.keys().any(|p| p.starts_with(&format!("{path}/"))) {
                return;
            }
        }
        match fs.remove(path, self.actor, false) {
            Ok(()) => {
                if self.opts.has('v') {
                    let message = format!(
                        "removed {}'{label}'\n",
                        if directory { "directory " } else { "" }
                    );
                    self.output.stdout.push_str(&message);
                    self.output.ordered.push((1, message));
                }
            }
            Err(error) => {
                let message = error.to_string();
                self.error(
                    label,
                    if message == "directory not empty; use -r" {
                        "Directory not empty"
                    } else {
                        &message
                    },
                );
            }
        }
    }
}

pub(crate) fn execute(world: &mut WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        "rm",
        args,
        "rRfvd",
        "",
        &[
            ("recursive", 'r'),
            ("force", 'f'),
            ("verbose", 'v'),
            ("dir", 'd'),
            ("version", 'V'),
        ],
    )?;
    if opts.help {
        return Ok(Output::success(
            crate::terminal_io::manual("rm").unwrap_or_default(),
        ));
    }
    if opts.has('V') {
        return Ok(Output::success("rm (GNU coreutils) 9.7\n".into()));
    }
    if opts.files.is_empty() && !opts.has('f') {
        return Err(domain(
            "rm: missing operand\nTry 'rm --help' for more information.",
        ));
    }
    let mut removal = Remove {
        actor,
        opts,
        output: Output::default(),
    };
    for label in removal.opts.files.clone() {
        if matches!(
            label.trim_end_matches('/').rsplit('/').next(),
            Some("." | "..")
        ) {
            removal.error(&label, "refusing to remove '.' or '..' directory");
            continue;
        }
        match normalize(&label, &world.terminal.cwd) {
            Ok(path) => removal.entry(world.fs_mut()?, &path, &label, 0),
            Err(error) => removal.error(&label, error),
        }
    }
    if !world.fs()?.nodes.contains_key(&world.terminal.cwd) {
        world.terminal.cwd = HOME.into();
        world.terminal.env.insert("PWD".into(), HOME.into());
    }
    Ok(removal.output)
}
