//! Incremental GNU cat transform. State survives chunks and operand boundaries.
use super::foundation_options;
use crate::{error::GameResult, terminal_io::Output, world::WorldState};

pub(crate) fn help() -> String {
    super::foundation_messages::message("cat", "cat", "help")
}

pub(crate) struct Cat {
    pub files: std::collections::VecDeque<String>,
    pub current: Option<String>,
    pub handle: Option<u64>,
    pub transform: Transform,
}
impl Cat {
    pub fn new(invocation: &str, args: &[String], posix: bool) -> Result<Self, Box<Output>> {
        let opts = foundation_options::parse("cat", invocation, args, posix)?;
        if let Some(kind) = opts.special {
            return Err(Box::new(Output::success(
                super::foundation_messages::message("cat", invocation, kind),
            )));
        }
        Ok(Self {
            files: if opts.files.is_empty() {
                ["-".into()].into()
            } else {
                opts.files.into()
            },
            current: None,
            handle: None,
            transform: Transform::new(&opts.flags),
        })
    }
}
pub(crate) fn execute(
    world: &mut WorldState,
    invocation: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    let user = std::mem::replace(&mut world.terminal.user, actor.into());
    let result = crate::shell_pipeline::run_stages(
        world,
        &[crate::shell_pipeline::Stage {
            arguments: std::iter::once(invocation.to_owned())
                .chain(args.iter().cloned())
                .collect(),
            ..Default::default()
        }],
    );
    world.terminal.user = user;
    result
}
#[derive(Default)]
pub(crate) struct Transform {
    number: u64,
    line_start: bool,
    blank_before: bool,
    pending_cr: bool,
    number_all: bool,
    number_nonblank: bool,
    squeeze: bool,
    visible: bool,
    tabs: bool,
    ends: bool,
}
impl Transform {
    pub fn new(flags: &[char]) -> Self {
        let has = |choices: &str| flags.iter().any(|c| choices.contains(*c));
        Self {
            number: 1,
            line_start: true,
            number_all: has("n"),
            number_nonblank: has("b"),
            squeeze: has("s"),
            visible: has("vAet"),
            tabs: has("TAt"),
            ends: has("EAe"),
            ..Default::default()
        }
    }
    fn visible(byte: u8, out: &mut Vec<u8>) {
        let mut b = byte;
        if b >= 128 {
            out.extend_from_slice(b"M-");
            b -= 128;
        }
        match b {
            0..=31 => out.extend_from_slice(&[b'^', b + 64]),
            127 => out.extend_from_slice(b"^?"),
            _ => out.push(b),
        }
    }
    pub fn transform(&mut self, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len());
        for &b in data {
            if self.pending_cr {
                out.extend_from_slice(if b == b'\n' { b"^M" } else { b"\r" });
                self.pending_cr = false;
            }
            let blank = self.line_start && b == b'\n';
            if blank && self.blank_before && self.squeeze {
                continue;
            }
            if self.line_start {
                if (self.number_nonblank && !blank) || (self.number_all && !self.number_nonblank) {
                    out.extend_from_slice(format!("{:>6}\t", self.number).as_bytes());
                    self.number += 1;
                }
                self.blank_before = blank;
            }
            match b {
                b'\n' => {
                    if self.ends {
                        out.push(b'$');
                    }
                    out.push(b'\n');
                }
                b'\t' => {
                    if self.tabs {
                        out.extend_from_slice(b"^I");
                    } else {
                        out.push(b);
                    }
                }
                b'\r' if self.ends && !self.visible => self.pending_cr = true,
                _ if self.visible => Self::visible(b, &mut out),
                _ => out.push(b),
            }
            self.line_start = b == b'\n';
        }
        out
    }
    pub fn finish(&mut self) -> Vec<u8> {
        if std::mem::take(&mut self.pending_cr) {
            vec![b'\r']
        } else {
            Vec::new()
        }
    }
}

pub(crate) fn diagnostic(file: &str, reason: impl std::fmt::Display) -> Vec<u8> {
    let reason = crate::terminal_io::error_reason(reason);
    // The pinned Linux/musl reference exposes this strerror spelling.
    let reason = if reason == "Too many levels of symbolic links" {
        "Symbolic link loop"
    } else {
        &reason
    };
    format!(
        "cat: {}: {}\n",
        foundation_options::shell_quote(file),
        reason
    )
    .into_bytes()
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use crate::world::WorldState;
    #[test]
    fn cat_signal_commits_only_delivered_output_and_closes_descriptors() {
        use crate::shell::{
            control,
            signals::{Termination, VirtualSignal},
        };
        use std::time::Duration;
        let (tx, rx) = std::sync::mpsc::channel();
        let channel = tauri::ipc::Channel::new(move |body| {
            if let crate::cli_contract::Event::Stdout(text) = body.deserialize().unwrap() {
                tx.send(text).unwrap();
            }
            Ok(())
        });
        let registration = control::register_output("cat-blocked-delivery", Some(channel));
        let (finished, result) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            control::run(&registration, || {
                let mut w = WorldState::new("kali", "lifeos").unwrap();
                w.vfs
                    .write("/home/kali/large", &"x".repeat(128 * 1024), "kali")
                    .unwrap();
                let output = crate::terminal::execute(&mut w, "cat large");
                assert!(!w.processes.iter().any(|p| p.name == "cat"));
                assert!(control::process_ids("cat-blocked-delivery").is_empty());
                assert_eq!(w.vfs.open_handle_count(), 0);
                finished.send(output).unwrap();
            })
        });
        let mut delivered = String::new();
        for _ in 0..16 {
            delivered.push_str(&rx.recv_timeout(Duration::from_secs(5)).unwrap());
        }
        let pid = control::process_ids("cat-blocked-delivery")[0];
        assert!(control::signal(
            "cat-blocked-delivery",
            Some(pid),
            VirtualSignal::Term
        ));
        let output = result.recv_timeout(Duration::from_secs(5)).unwrap();
        worker.join().unwrap();
        assert_eq!(output.stdout_bytes, delivered.as_bytes());
        assert_eq!(delivered.len(), 64 * 1024);
        assert_eq!(output.exit_code, 143);
        assert_eq!(
            output.termination,
            Termination::Signal {
                signal: VirtualSignal::Term
            }
        );
    }
    #[test]
    fn cat_crlf_transform_is_chunk_independent() {
        let mut full = Transform::new(&['E']);
        let mut whole = full.transform(b"x\r\ny\r");
        whole.extend(full.finish());
        let mut cat = Transform::new(&['E']);
        let mut split = cat.transform(b"x\r");
        split.extend(cat.transform(b"\ny\r"));
        split.extend(cat.finish());
        assert_eq!(split, whole);
    }
    #[test]
    fn cat_transformed_tty_emits_before_eof() {
        use crate::shell::control;
        use std::time::Duration;
        let (tx, rx) = std::sync::mpsc::channel();
        let channel = tauri::ipc::Channel::new(move |body| {
            if matches!(
                body.deserialize::<crate::cli_contract::Event>().unwrap(),
                crate::cli_contract::Event::Stdout(_)
            ) {
                tx.send(()).unwrap();
            }
            Ok(())
        });
        let registration = control::register_output("cat-before-eof", Some(channel));
        let worker = std::thread::spawn(move || {
            control::run(&registration, || {
                let mut w = WorldState::new("kali", "lifeos").unwrap();
                crate::terminal::execute(&mut w, "cat -n")
            })
        });
        assert!(control::input("cat-before-eof", Some("hello\n".into())));
        let emitted = rx.recv_timeout(Duration::from_secs(2)).is_ok();
        control::cancel("cat-before-eof");
        let result = worker.join().unwrap();
        assert!(emitted, "transformed cat waited for EOF: {}", result.stderr);
    }
    #[test]
    fn cat_transform_chunk_properties_cover_all_bytes_and_options() {
        let mut state = 0xace1u32;
        let mut data = (0..=255).collect::<Vec<u8>>();
        for _ in 0..2000 {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            data.push(state as u8);
        }
        data.extend_from_slice(b"\r\n\n\nA\n\nB\r\t\r\n");
        for bits in 0..64 {
            let flags: Vec<char> = ['v', 'E', 'T', 's', 'n', 'b']
                .into_iter()
                .enumerate()
                .filter(|(i, _)| bits & (1 << i) != 0)
                .map(|(_, f)| f)
                .collect();
            let mut full = Transform::new(&flags);
            let mut expected = full.transform(&data);
            expected.extend(full.finish());
            for size in [1, 2, 3, 7, 31, 255, 4096] {
                let mut transform = Transform::new(&flags);
                let mut actual = Vec::new();
                for chunk in data.chunks(size) {
                    actual.extend(transform.transform(chunk));
                }
                actual.extend(transform.finish());
                assert_eq!(actual, expected, "{flags:?}, chunk {size}");
            }
        }
    }
    #[test]
    fn cat_streams_files_and_preserves_shell_package_context() {
        let mut w = WorldState::new("kali", "lifeos").unwrap();
        w.vfs.write("/home/kali/a", "A\r", "kali").unwrap();
        w.vfs.write("/home/kali/b", "\n\nB", "kali").unwrap();
        let result = crate::terminal::execute(&mut w, "cat -nE a b | cat | cat > out");
        assert_eq!(result.exit_code, 0, "{}", result.stderr);
        assert_eq!(
            w.vfs.readable("/home/kali/out", "kali").unwrap().content,
            "     1\tA^M$\n     2\t$\n     3\tB"
        );
        assert!(!w.processes.iter().any(|p| p.name == "cat"));
        w.packages.installed.remove("coreutils");
        assert_eq!(crate::terminal::execute(&mut w, "cat a").exit_code, 126);
    }
}
