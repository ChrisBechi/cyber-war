//! Offline compatibility cases. Test data is never part of the production binary.
use crate::{terminal::execute, world::WorldState};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    reference: String,
    command: String,
    #[serde(default)]
    initial_filesystem: BTreeMap<String, String>,
    #[serde(default)]
    environment: BTreeMap<String, String>,
    #[serde(default)]
    stdin: Option<String>,
    #[serde(default = "columns")]
    terminal_width: usize,
    #[serde(default)]
    is_tty: bool,
    expected_stdout: String,
    expected_stderr: String,
    expected_exit_code: i32,
    #[serde(default)]
    expected_filesystem: BTreeMap<String, String>,
    #[serde(default)]
    absent_paths: Vec<String>,
}
fn columns() -> usize {
    80
}

#[test]
fn cli_compat_inventory_covers_runtime_registrations() {
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("../../docs/cli/inventory.json")).unwrap();
    let expected: std::collections::BTreeSet<String> = inventory["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["command"].as_str().unwrap().into())
        .collect();
    let mut found: std::collections::BTreeSet<String> = crate::terminal::COMMANDS
        .iter()
        .map(|n| n.to_string())
        .collect();
    found.extend(crate::shell::SHELL_ONLY.iter().map(|name| name.to_string()));
    for e in &crate::software::CATALOG.entries {
        found.insert(e.id.clone());
        found.insert(e.name.clone());
        found.extend(e.commands.clone());
    }
    for repo in crate::packages::repository::REPOSITORIES.values() {
        for (_, entry) in &repo.entries {
            for file in &entry.package.files {
                if file.binding.is_some() {
                    found.insert(file.path.rsplit('/').next().unwrap().to_string());
                }
            }
        }
    }
    assert_eq!(
        found, expected,
        "Runtime discovery differs from the reviewed inventory"
    );
}

#[test]
fn cli_compat_golden_cases() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("../../tests/cli/compat/shell.json")).unwrap();
    let mut failures = Vec::new();
    for case in &cases {
        assert!(
            !case.reference.is_empty(),
            "reference required: {}",
            case.id
        );
        let mut world = WorldState::new("kali", "lifeos").unwrap();
        world.terminal.env = case.environment.clone();
        world.terminal.stdin = case.stdin.clone();
        world.terminal.presentation.columns = case.terminal_width;
        world.terminal.io.stdout_tty = case.is_tty;
        world.terminal.io.stdin_tty = case.is_tty && case.stdin.is_none();
        world.terminal.io.stderr_tty = case.is_tty;
        for (path, content) in &case.initial_filesystem {
            world.vfs.write(path, content, "kali").unwrap();
        }
        let out = execute(&mut world, &case.command);
        if (out.stdout.as_str(), out.stderr.as_str(), out.exit_code)
            != (
                case.expected_stdout.as_str(),
                case.expected_stderr.as_str(),
                case.expected_exit_code,
            )
        {
            failures.push(format!(
                "{}: got {:?}, expected {:?}",
                case.id,
                (&out.stdout, &out.stderr, out.exit_code),
                (
                    &case.expected_stdout,
                    &case.expected_stderr,
                    case.expected_exit_code
                )
            ));
        }
        for (path, content) in &case.expected_filesystem {
            if world.vfs.read(path, "root").ok().as_ref() != Some(content) {
                failures.push(format!("{}: filesystem mismatch {path}", case.id));
            }
        }
        for path in &case.absent_paths {
            if world.vfs.nodes.contains_key(path) {
                failures.push(format!("{}: unexpected file {path}", case.id));
            }
        }
        assert_eq!(
            world.terminal.last_status, out.exit_code,
            "{}: $?'s backing status",
            case.id
        );
        assert_eq!(
            world.terminal.cwd, "/home/kali",
            "{}: caller directory",
            case.id
        );
    }
    assert!(
        failures.is_empty(),
        "{} / {} cases failed:\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
    println!(
        "{} exact compatibility cases passed; no output normalization",
        cases.len()
    );
}

#[test]
fn cli_compat_tty_follows_each_descriptor_and_is_restored() {
    let mut world = WorldState::new("kali", "lifeos").unwrap();
    for width in [40, 80, 120, 200] {
        world.terminal.presentation.columns = width;
        assert!(execute(&mut world, "fastfetch").stdout.contains('\x1b'));
        let redirected = execute(&mut world, "fastfetch > fetch.txt");
        assert_eq!(redirected.exit_code, 0);
        assert!(!world
            .vfs
            .read("/home/kali/fetch.txt", "kali")
            .unwrap()
            .contains('\x1b'));
        let piped = execute(&mut world, "fastfetch | grep '^OS:'");
        assert_eq!(piped.exit_code, 0);
        assert!(!piped.stdout.contains('\x1b'));
        assert!(execute(&mut world, "fastfetch >&2").stderr.contains('\x1b'));
        assert!(world.terminal.io.stdout_tty);
    }
}

#[test]
fn cli_compat_opening_redirect_is_not_rolled_back_with_command_failure() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs.write("/home/kali/out", "old", "kali").unwrap();
    let out = execute(&mut w, "missing-command > out");
    assert_eq!(out.exit_code, 127);
    assert_eq!(w.vfs.read("/home/kali/out", "kali").unwrap(), "");
    let out = execute(&mut w, "echo nope > first > /root/denied");
    assert_ne!(out.exit_code, 0);
    assert_eq!(w.vfs.read("/home/kali/first", "kali").unwrap(), "");
    assert!(!w.vfs.nodes.contains_key("/root/denied"));
}

#[test]
fn cli_compat_virtual_boundary_and_pipeline_context() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    for text in [
        "cat < 'C:\\Windows\\win.ini'",
        "cat < /dev/tcp/example.com/80",
        "echo x > /dev/tcp/example.com/80",
        "curl https://example.com/",
    ] {
        assert_ne!(execute(&mut w, text).exit_code, 0, "{text}");
    }
    assert_eq!(execute(&mut w, "cd Documents | cat").exit_code, 0);
    assert_eq!(w.terminal.cwd, "/home/kali");
    execute(&mut w, "export SECRET=value | cat");
    assert!(!w.terminal.env.contains_key("SECRET"));
    execute(&mut w, "echo persisted > saved | cat");
    assert_eq!(
        w.vfs.read("/home/kali/saved", "kali").unwrap(),
        "persisted\n"
    );
}

#[test]
fn cli_compat_package_resolution_obeys_path_and_execute_permission() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    execute(&mut w, "export PATH=/missing");
    assert_eq!(execute(&mut w, "ls").exit_code, 127);
    assert_eq!(execute(&mut w, "dpkg -l").exit_code, 127);
    assert_eq!(execute(&mut w, "/usr/bin/ls -d .").exit_code, 0);
    execute(&mut w, "export PATH=/usr/bin");
    w.vfs.chmod("/usr/bin/ls", "root", 0o644).unwrap();
    assert_eq!(execute(&mut w, "ls").exit_code, 126);
    assert_eq!(execute(&mut w, "/usr/bin/ls").exit_code, 126);
    w.vfs.chmod("/usr/bin/ls", "root", 0o755).unwrap();
    execute(&mut w, "cd /usr");
    execute(&mut w, "export PATH=bin");
    assert_eq!(execute(&mut w, "ls -d .").exit_code, 0);
    w.vfs
        .write("/home/kali/program", "#!/bin/bash\necho script\n", "kali")
        .unwrap();
    assert_eq!(execute(&mut w, "/home/kali/program").exit_code, 126);
    w.vfs.chmod("/home/kali/program", "kali", 0o755).unwrap();
    assert_eq!(execute(&mut w, "/home/kali/program").stdout, "script\n");
}

#[test]
fn cli_compat_ls_recursion_color_and_symlinks() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    for path in ["/home/kali/list", "/home/kali/list/nested"] {
        w.vfs.mkdir(path, "kali").unwrap();
    }
    w.vfs.write("/home/kali/list/a", "a", "kali").unwrap();
    w.vfs.write("/home/kali/list/.hidden", "h", "kali").unwrap();
    w.vfs
        .write("/home/kali/list/nested/b", "b", "kali")
        .unwrap();
    assert_eq!(
        execute(&mut w, "ls -1R list").stdout,
        "list:\na\nnested\n\nlist/nested:\nb\n"
    );
    assert_eq!(
        execute(&mut w, "ls -1aA list").stdout,
        ".\n..\n.hidden\na\nnested\n"
    );
    for width in [40, 80, 120, 200] {
        w.terminal.presentation.columns = width;
        assert!(execute(&mut w, "ls --color=auto -d list")
            .stdout
            .contains('\x1b'));
        assert!(!execute(&mut w, "ls --color=auto -d list | cat")
            .stdout
            .contains('\x1b'));
        assert!(execute(&mut w, "ls --color=always -d list | cat")
            .stdout
            .contains('\x1b'));
        assert!(!execute(&mut w, "ls --color=never -d list")
            .stdout
            .contains('\x1b'));
    }
    w.vfs
        .symlink("/home/kali/list/link", "missing", "kali")
        .unwrap();
    let link = execute(&mut w, "ls -l list/link");
    assert!(link.stdout.starts_with('l'));
    assert!(link.stdout.ends_with("list/link -> missing\n"));
    assert_eq!(execute(&mut w, "ls -dF list/link").stdout, "list/link@\n");
    assert_eq!(execute(&mut w, "ls --color=invalid").exit_code, 2);
    let invalid = execute(&mut w, "ls --batata");
    assert_eq!(
        invalid.stderr,
        "ls: unrecognized option '--batata'\nTry 'ls --help' for more information.\n"
    );
}

#[test]
fn cli_compat_recursive_grep_uses_vfs_and_keeps_exit_status() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs.mkdir("/home/kali/search", "kali").unwrap();
    w.vfs.mkdir("/home/kali/search/sub", "kali").unwrap();
    w.vfs
        .write("/home/kali/search/a", "Alpha\nno\n", "kali")
        .unwrap();
    w.vfs
        .write("/home/kali/search/sub/b", "alpha\n", "kali")
        .unwrap();
    let r = execute(&mut w, "grep -rin alpha search");
    assert_eq!(
        (r.stdout.as_str(), r.stderr.as_str(), r.exit_code),
        ("search/a:1:Alpha\nsearch/sub/b:1:alpha\n", "", 0)
    );
    assert_eq!(execute(&mut w, "grep -rh alpha search").stdout, "alpha\n");
    assert_eq!(execute(&mut w, "grep -r absent search").exit_code, 1);
    w.vfs.chmod("/home/kali/search/sub", "kali", 0).unwrap();
    let r = execute(&mut w, "grep -ri alpha search");
    assert_eq!(r.exit_code, 2);
    assert_eq!(r.stdout, "search/a:Alpha\n");
    assert!(r.stderr.contains("Permission denied"));
    assert_eq!(execute(&mut w, "grep -rqi alpha search").exit_code, 0);
}

#[test]
fn cli_compat_parsers_are_bounded_and_do_not_panic() {
    let environment = BTreeMap::from([("VALUE".into(), "a b 2>&1".into())]);
    let alphabet = [
        'a', '2', ' ', '\'', '"', '\\', '$', '<', '>', '|', '&', '\0',
    ];
    for a in alphabet {
        for b in alphabet {
            for c in alphabet {
                let input = format!("echo {a}{b}{c}");
                let _ = crate::shell::words(&input, &environment, "bash", &[], 0);
            }
        }
    }
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    for command in [
        "cat 99> out",
        "cat < < out",
        "echo ok >out |",
        "| cat",
        "cat 2>&bad",
        "cat <&0",
    ] {
        assert_eq!(execute(&mut w, command).exit_code, 2, "{command}");
        assert!(!w.vfs.nodes.contains_key("/home/kali/out"));
    }
}

#[test]
fn cli_compat_copy_merges_and_keeps_successful_children_on_permission_failure() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    for dir in ["src", "src/sub", "Documents/src", "Documents/src/sub"] {
        w.vfs.mkdir(&format!("/home/kali/{dir}"), "kali").unwrap();
    }
    for (path, text) in [
        ("src/a", "new"),
        ("src/sub/readable", "read"),
        ("src/sub/secret", "secret"),
        ("Documents/src/a", "old"),
        ("Documents/src/keep", "keep"),
    ] {
        w.vfs
            .write(&format!("/home/kali/{path}"), text, "kali")
            .unwrap();
    }
    w.vfs.chmod("/home/kali/src/sub/secret", "kali", 0).unwrap();
    let out = execute(&mut w, "cp -r src Documents");
    assert_eq!(out.exit_code, 1);
    assert!(out.stderr.contains("Permission denied"));
    for (path, text) in [
        ("Documents/src/a", "new"),
        ("Documents/src/sub/readable", "read"),
        ("Documents/src/keep", "keep"),
    ] {
        assert_eq!(
            w.vfs.read(&format!("/home/kali/{path}"), "kali").unwrap(),
            text
        );
    }
    assert!(!w
        .vfs
        .nodes
        .contains_key("/home/kali/Documents/src/sub/secret"));
    let out = execute(&mut w, "cp -r src/. Downloads");
    assert_eq!(out.exit_code, 1);
    assert_eq!(w.vfs.read("/home/kali/Downloads/a", "kali").unwrap(), "new");
    assert!(!w.vfs.nodes.contains_key("/home/kali/Downloads/src"));
}

#[test]
fn cli_compat_move_is_a_rename_with_parent_permissions_and_preserved_metadata() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs.mkdir("/home/kali/locked", "kali").unwrap();
    w.vfs
        .write("/home/kali/locked/secret", "secret", "kali")
        .unwrap();
    w.vfs.chmod("/home/kali/locked/secret", "kali", 0).unwrap();
    w.vfs.chmod("/home/kali/locked", "kali", 0).unwrap();
    let original = w.vfs.nodes["/home/kali/locked/secret"].clone();
    assert_eq!(execute(&mut w, "mv locked renamed").exit_code, 0);
    let moved = &w.vfs.nodes["/home/kali/renamed/secret"];
    assert_eq!(
        (moved.mode, moved.created_at, moved.modified_at),
        (original.mode, original.created_at, original.modified_at)
    );
    assert_eq!(moved.content, "secret");
    assert!(!w.vfs.nodes.contains_key("/home/kali/locked"));
    w.vfs.mkdir("/home/kali/empty", "kali").unwrap();
    assert_eq!(execute(&mut w, "mv -T renamed empty").exit_code, 0);
    assert!(w.vfs.nodes.contains_key("/home/kali/empty/secret"));
    w.vfs.mkdir("/home/kali/other", "kali").unwrap();
    w.vfs
        .write("/home/kali/other/keep", "keep", "kali")
        .unwrap();
    let before = w.vfs.clone();
    let out = execute(&mut w, "mv -T empty other");
    assert_eq!(out.exit_code, 1);
    assert!(out.stderr.contains("Directory not empty"));
    assert_eq!(w.vfs, before);
}

#[test]
fn cli_compat_copy_modes_force_update_and_links_use_virtual_metadata() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs.write("/home/kali/source", "new", "kali").unwrap();
    w.vfs.chmod("/home/kali/source", "kali", 0o777).unwrap();
    assert_eq!(execute(&mut w, "cp source copied").exit_code, 0);
    let copied = &w.vfs.nodes["/home/kali/copied"];
    assert_eq!(copied.mode, 0o755);
    assert!(copied.modified_at > w.vfs.nodes["/home/kali/source"].modified_at);
    w.vfs.write("/home/kali/copied", "newer", "kali").unwrap();
    assert_eq!(execute(&mut w, "cp -u source copied").exit_code, 0);
    assert_eq!(w.vfs.read("/home/kali/copied", "kali").unwrap(), "newer");
    w.vfs.chmod("/home/kali/copied", "kali", 0).unwrap();
    assert_eq!(execute(&mut w, "cp source copied").exit_code, 1);
    assert_eq!(execute(&mut w, "cp -f source copied").exit_code, 0);
    assert_eq!(w.vfs.read("/home/kali/copied", "kali").unwrap(), "new");
    w.vfs.symlink("/home/kali/link", "source", "kali").unwrap();
    assert_eq!(execute(&mut w, "cp link followed").exit_code, 0);
    assert_eq!(w.vfs.read("/home/kali/followed", "kali").unwrap(), "new");
    assert_eq!(execute(&mut w, "cp -P link preserved").exit_code, 0);
    assert_eq!(w.vfs.nodes["/home/kali/preserved"].kind, "symlink");
    assert_eq!(w.vfs.nodes["/home/kali/preserved"].content, "source");
    w.vfs.capacity_bytes = w.vfs.used_bytes();
    assert_eq!(execute(&mut w, "cp source full").exit_code, 1);
    assert!(!w.vfs.nodes.contains_key("/home/kali/full"));
}

#[test]
fn cli_compat_partial_transfer_is_persisted_through_service_and_reload() {
    let mut game =
        crate::service::GameService::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
    game.new_game(1, "neo", "pc", false).unwrap();
    let result = game
        .mutate(
            |_, w, _| {
                w.vfs.write("/home/kali/a", "A", "kali")?;
                w.vfs.write("/home/kali/b", "B", "kali")?;
                Ok(execute(w, "mv -v a missing b Documents 2>&1"))
            },
            false,
        )
        .unwrap();
    assert_eq!(result.exit_code, 1);
    assert_eq!(result.stdout, "renamed 'a' -> 'Documents/a'\nmv: cannot stat 'missing': No such file or directory\nrenamed 'b' -> 'Documents/b'\n");
    assert!(result.stderr.is_empty());
    game.end_session().unwrap();
    game.load(1, false).unwrap();
    let w = game.world().unwrap();
    assert_eq!(w.vfs.read("/home/kali/Documents/a", "kali").unwrap(), "A");
    assert_eq!(w.vfs.read("/home/kali/Documents/b", "kali").unwrap(), "B");
    assert!(!w.vfs.nodes.contains_key("/home/kali/a"));
    assert!(!w.vfs.nodes.contains_key("/home/kali/b"));
}

#[test]
fn cli_compat_rm_checks_each_parent_and_keeps_partial_results() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    for dir in ["remove", "remove/locked", "empty"] {
        w.vfs.mkdir(&format!("/home/kali/{dir}"), "kali").unwrap();
    }
    for (path, text) in [
        ("remove/a", "A"),
        ("remove/locked/keep", "keep"),
        ("after", "after"),
    ] {
        w.vfs
            .write(&format!("/home/kali/{path}"), text, "kali")
            .unwrap();
    }
    w.vfs
        .chmod("/home/kali/remove/locked", "kali", 0o555)
        .unwrap();
    let r = execute(&mut w, "rm -rfv remove missing after");
    assert_eq!(r.exit_code, 1);
    assert_eq!(r.stdout, "removed 'remove/a'\nremoved 'after'\n");
    assert_eq!(
        r.stderr,
        "rm: cannot remove 'remove/locked/keep': Permission denied\n"
    );
    assert!(w.vfs.nodes.contains_key("/home/kali/remove/locked/keep"));
    assert!(!w.vfs.nodes.contains_key("/home/kali/after"));
    assert_eq!(
        execute(&mut w, "rm -dv empty").stdout,
        "removed directory 'empty'\n"
    );
    assert_eq!(execute(&mut w, "rm -d remove").exit_code, 1);
    assert!(w.vfs.nodes.contains_key("/home/kali/remove"));
    w.vfs.symlink("/home/kali/link", "remove", "kali").unwrap();
    assert_eq!(execute(&mut w, "rm -r link").exit_code, 0);
    assert!(!w.vfs.nodes.contains_key("/home/kali/link"));
    assert!(w.vfs.nodes.contains_key("/home/kali/remove/locked/keep"));
    for operand in [
        "remove/.",
        "remove/..",
        "/",
        "/home/kali/.local/share/Trash",
    ] {
        let before = w.vfs.clone();
        assert_eq!(execute(&mut w, &format!("rm -rf {operand}")).exit_code, 1);
        assert_eq!(w.vfs, before);
    }
}

#[test]
fn cli_compat_text_errors_limits_and_output_paths_do_not_hide_data_loss() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs.write("/home/kali/a", "b\na\n", "kali").unwrap();
    w.vfs.write("/home/kali/out", "keep", "kali").unwrap();
    let r = execute(&mut w, "sort a missing -o out");
    assert_eq!(r.exit_code, 2);
    assert!(r.stdout.is_empty());
    assert_eq!(w.vfs.read("/home/kali/out", "kali").unwrap(), "keep");
    assert_eq!(execute(&mut w, "sort -o --check=quiet a").exit_code, 0);
    assert_eq!(
        w.vfs.read("/home/kali/--check=quiet", "kali").unwrap(),
        "a\nb\n"
    );
    for command in [
        "wc -t only a",
        "uniq -w18446744073709551616 a out",
        "sort -V a",
        "sort --check=invalid a",
        "wc --total=wrong a",
    ] {
        let before = w.vfs.clone();
        assert_ne!(execute(&mut w, command).exit_code, 0, "{command}");
        assert_eq!(w.vfs, before, "{command}");
    }
    w.vfs
        .write("/home/kali/many", &"a\n".repeat(65_537), "kali")
        .unwrap();
    let r = execute(&mut w, "sort many -o out");
    assert_eq!(r.exit_code, 2);
    assert!(r.stderr.contains("65536"));
    assert_eq!(w.vfs.read("/home/kali/out", "kali").unwrap(), "keep");
    for name in ["wc", "sort", "uniq"] {
        assert_eq!(
            execute(&mut w, &format!("{name} --version")).stdout,
            format!("{name} (GNU coreutils) 9.7\n")
        );
        let help = execute(&mut w, &format!("{name} --help"));
        assert!(help.stdout.contains("virtual subset"));
        assert_eq!(execute(&mut w, &format!("man {name}")).stdout, help.stdout);
    }
    w.terminal.env.insert("LC_CTYPE".into(), "C".into());
    w.vfs.write("/home/kali/utf8", "á\n", "kali").unwrap();
    assert_eq!(execute(&mut w, "wc -m utf8").stdout, "3 utf8\n");
    w.terminal.env.insert("LC_ALL".into(), "C.UTF-8".into());
    assert_eq!(execute(&mut w, "wc -m utf8").stdout, "2 utf8\n");
}

#[test]
fn cli_compat_runtime_sources_have_no_host_execution_or_network_fallback() {
    for source in [
        include_str!("terminal.rs"),
        include_str!("shell.rs"),
        include_str!("shell_pipeline.rs"),
        include_str!("terminal_io.rs"),
        include_str!("terminal_query.rs"),
        include_str!("terminal_transfer.rs"),
        include_str!("terminal_remove.rs"),
        include_str!("terminal_text.rs"),
        include_str!("packages/executables.rs"),
    ] {
        for forbidden in [
            "Command::new(",
            "std::process::",
            "TcpStream::",
            "UdpSocket::",
            "reqwest::",
            "std::fs::",
            "tokio::process::",
        ] {
            assert!(
                !source.contains(forbidden),
                "CLI boundary contains {forbidden}"
            );
        }
    }
}
