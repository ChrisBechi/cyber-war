//! Contract fixtures derived from the GNU manuals, run only against virtual data.
use crate::{
    terminal::{execute, CommandResult},
    terminal_sessions,
    world::WorldState,
};

fn world() -> WorldState {
    WorldState::new("neo", "pc").unwrap()
}
fn file(world: &mut WorldState, name: &str, content: &str) {
    world
        .vfs
        .write(&format!("/home/kali/{name}"), content, "kali")
        .unwrap();
}
fn ok(world: &mut WorldState, command: &str, expected: &str) {
    let result = execute(world, command);
    assert_eq!(
        (
            result.exit_code,
            result.stderr.as_str(),
            result.stdout.as_str()
        ),
        (0, "", expected),
        "{command}"
    );
}
fn session(world: &mut WorldState, id: &str, command: &str) -> CommandResult {
    terminal_sessions::with_session(world, Some(id), |world| Ok(execute(world, command))).unwrap()
}

#[test]
fn navigation_previous_directory_home_and_independent_environment() {
    let mut w = world();
    assert_ne!(execute(&mut w, "cd -").exit_code, 0);
    ok(&mut w, "cd Documents", "");
    ok(&mut w, "pwd -P", "/home/kali/Documents\n");
    ok(&mut w, "printenv PWD", "/home/kali/Documents\n");
    ok(&mut w, "printenv OLDPWD", "/home/kali\n");
    ok(&mut w, "cd -", "/home/kali\n");
    ok(&mut w, "cd ''", "");
    ok(&mut w, "export HOME=/home/kali/Downloads", "");
    ok(&mut w, "cd", "");
    ok(&mut w, "pwd -L", "/home/kali/Downloads\n");
    let old_env = w.terminal.env.clone();
    for cmd in ["cd missing", "cd /root", "cd a b", "pwd --bogus", "cd -Z"] {
        assert_ne!(execute(&mut w, cmd).exit_code, 0, "{cmd}");
        assert_eq!(w.terminal.cwd, "/home/kali/Downloads");
        assert_eq!(w.terminal.env, old_env);
    }
    terminal_sessions::open(&mut w, "a").unwrap();
    terminal_sessions::open(&mut w, "b").unwrap();
    assert_eq!(session(&mut w, "a", "cd Documents").exit_code, 0);
    assert_eq!(session(&mut w, "b", "pwd").stdout, "/home/kali\n");
    assert_ne!(session(&mut w, "b", "cd -").exit_code, 0);
    assert_eq!(session(&mut w, "a", "cd -").stdout, "/home/kali\n");
}

#[test]
fn cat_preserves_bytes_and_numbers_across_file_boundaries() {
    let mut w = world();
    file(&mut w, "a", "A\n\nB");
    file(&mut w, "b", "C\r\nD\tE");
    ok(&mut w, "cat a b", "A\n\nBC\r\nD\tE");
    ok(
        &mut w,
        "cat -n a b",
        "     1\tA\n     2\t\n     3\tBC\r\n     4\tD\tE",
    );
    ok(
        &mut w,
        "cat -nbET a b",
        "     1\tA$\n$\n     2\tBC^M$\n     3\tD^IE",
    );
    ok(
        &mut w,
        "cat --number --number-nonblank a b",
        "     1\tA\n\n     2\tBC\r\n     3\tD\tE",
    );
    file(&mut w, "empty-lines", "\n\n\nA\n\n\n");
    ok(
        &mut w,
        "cat -sEn empty-lines",
        "     1\t$\n     2\tA$\n     3\t$\n",
    );
}

#[test]
fn head_tail_counts_preserve_crlf_and_unterminated_lines() {
    let mut w = world();
    file(&mut w, "sample", "a\r\nb\nc");
    for (cmd, expected) in [
        ("head -n2 sample", "a\r\nb\n"),
        ("head --lines=0 sample", ""),
        ("tail -n1 sample", "c"),
        ("tail -n+2 sample", "b\nc"),
        ("tail -n+0 sample", "a\r\nb\nc"),
        ("tail -n-2 sample", "b\nc"),
        ("head -n-1 sample", "a\r\nb\n"),
        ("head -c-2 sample", "a\r\nb"),
        ("tail --bytes=+4 sample", "b\nc"),
        ("tail -c2 sample", "\nc"),
        ("head -2 sample", "a\r\nb\n"),
        ("head -qn 1 sample", "a\r\n"),
        ("head -n1 -c2 sample", "a\r"),
        ("tail -n 999 sample", "a\r\nb\nc"),
        ("tail -n+999 sample", ""),
        ("head -n-999 sample", ""),
    ] {
        ok(&mut w, cmd, expected);
    }
    file(&mut w, "empty", "");
    ok(&mut w, "head empty", "");
    ok(&mut w, "tail empty", "");
}

#[test]
fn filenames_after_double_dash_never_become_flags() {
    let mut w = world();
    file(&mut w, "-n", "literal\n");
    file(&mut w, "-f", "protected");
    ok(&mut w, "cat -- -n", "literal\n");
    ok(&mut w, "head -n1 -- -n", "literal\n");
    ok(&mut w, "cp -- -n Documents/-n", "");
    ok(&mut w, "cat Documents/-n", "literal\n");
    ok(&mut w, "rm -- -f", "");
    assert!(!w.vfs.nodes.contains_key("/home/kali/-f"));
    assert_ne!(execute(&mut w, "rm -- -f").exit_code, 0);
}

#[test]
fn read_errors_keep_successful_stdout_and_nonzero_status() {
    let mut w = world();
    file(&mut w, "a", "A\n");
    file(&mut w, "b", "B");
    for cmd in [
        "cat a missing b",
        "head -q a missing b",
        "tail -q a missing b",
    ] {
        let result = execute(&mut w, cmd);
        assert_eq!(result.stdout, "A\nB", "{cmd}");
        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("missing"));
    }
    let result = execute(&mut w, "cat a missing b > Documents/partial.txt");
    assert_eq!(result.exit_code, 1);
    assert!(result.stdout.is_empty());
    assert!(result.stderr.contains("missing"));
    assert_eq!(
        w.vfs
            .read("/home/kali/Documents/partial.txt", "kali")
            .unwrap(),
        "A\nB"
    );
}

#[test]
fn file_headers_use_supplied_names_and_last_verbosity_flag() {
    let mut w = world();
    file(&mut w, "a", "A\n");
    file(&mut w, "b", "B");
    ok(&mut w, "head a b", "==> a <==\nA\n\n==> b <==\nB");
    ok(&mut w, "tail -q a b", "A\nB");
    ok(&mut w, "head -qv a", "==> a <==\nA\n");
    ok(&mut w, "tail -vq a b", "A\nB");
}

#[test]
fn unsupported_flags_and_stdin_are_explicit_and_nonmutating() {
    let mut w = world();
    file(&mut w, "a", "original");
    let before = serde_json::to_string(&w.vfs).unwrap();
    for cmd in [
        "cat --invented a",
        "head -f a",
        "tail --follow=invalid a",
        "tail -n",
        "head -n bad a",
        "cp --backup a Documents/a",
        "mv -r a Documents/a",
        "rm --invented a",
        "rm -I a",
    ] {
        let result = execute(&mut w, cmd);
        assert_ne!(result.exit_code, 0, "{cmd}");
        assert!(!result.stderr.is_empty(), "{cmd}");
        assert_eq!(serde_json::to_string(&w.vfs).unwrap(), before, "{cmd}");
    }
    // These former subset rejections are accepted by the pinned GNU reference.
    ok(&mut w, "tail -", "");
    ok(&mut w, "head -c2K a", "original");
    ok(
        &mut w,
        "head -n9999999999999999999999999999999 a",
        "original",
    );
}

#[test]
fn utf8_byte_ranges_preserve_exact_bytes_without_semantic_replacement() {
    let mut w = world();
    file(&mut w, "utf8", "áZ");
    ok(&mut w, "head -c2 utf8", "á");
    ok(&mut w, "tail -c1 utf8", "Z");
    let result = execute(&mut w, "head -c1 utf8");
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout_bytes, vec![0xc3]);
    assert!(result.stderr.is_empty());
}

#[test]
fn copy_move_overwrite_and_no_clobber_obey_permissions() {
    let mut w = world();
    file(&mut w, "a", "new");
    file(&mut w, "b", "old");
    ok(&mut w, "cp -nv a b", "");
    ok(&mut w, "cat b", "old");
    ok(&mut w, "cp -v a b", "'a' -> 'b'\n");
    ok(&mut w, "cat b", "new");
    ok(&mut w, "mv -n a b", "");
    assert!(w.vfs.nodes.contains_key("/home/kali/a"));
    ok(&mut w, "mv -v a b", "renamed 'a' -> 'b'\n");
    assert!(!w.vfs.nodes.contains_key("/home/kali/a"));
    file(&mut w, "a", "keep");
    w.vfs.chmod("/home/kali/b", "kali", 0o400).unwrap();
    assert_ne!(execute(&mut w, "cp a b").exit_code, 0);
    ok(&mut w, "cat b", "new");
    ok(&mut w, "sudo cp a b", "");
    ok(&mut w, "cat b", "keep");
    assert_eq!(w.terminal.user, "kali");
    ok(&mut w, "cp -v a Documents", "'a' -> 'Documents/a'\n");
}

#[test]
fn directory_copy_requires_recursion_and_merges_existing_directories() {
    let mut w = world();
    w.vfs.mkdir("/home/kali/source", "kali").unwrap();
    file(&mut w, "source/a", "A");
    assert_ne!(execute(&mut w, "cp source copy").exit_code, 0);
    ok(&mut w, "cp -R source copy", "");
    ok(&mut w, "cat copy/a", "A");
    let before = serde_json::to_string(&w.vfs).unwrap();
    assert_ne!(execute(&mut w, "cp -r source source/nested").exit_code, 0);
    assert_eq!(serde_json::to_string(&w.vfs).unwrap(), before);
    ok(&mut w, "cp -r source Documents", "");
    file(&mut w, "source/b", "B");
    ok(&mut w, "cp -r source Documents", "");
    assert_eq!(
        w.vfs.read("/home/kali/Documents/source/b", "kali").unwrap(),
        "B"
    );
    assert_eq!(
        w.vfs.read("/home/kali/Documents/source/a", "kali").unwrap(),
        "A"
    );
}

#[test]
fn rm_force_does_not_hide_permissions_and_successful_operands_survive_failure() {
    let mut w = world();
    file(&mut w, "keep", "keep");
    assert_ne!(execute(&mut w, "rm keep missing").exit_code, 0);
    assert!(!w.vfs.nodes.contains_key("/home/kali/keep"));
    file(&mut w, "keep", "keep");
    assert_ne!(execute(&mut w, "rm -f /root/missing").exit_code, 0);
    assert_ne!(execute(&mut w, "rm -f /root/missing/child").exit_code, 0);
    assert_ne!(execute(&mut w, "rm -f keep/child").exit_code, 0);
    ok(&mut w, "rm -f non/existing/file", "");
    ok(&mut w, "rm -v keep", "removed 'keep'\n");
    w.vfs.mkdir("/home/kali/dir", "kali").unwrap();
    file(&mut w, "dir/a", "a");
    ok(
        &mut w,
        "rm -rv dir",
        "removed 'dir/a'\nremoved directory 'dir'\n",
    );
}

#[test]
fn redirection_opens_before_reading_and_append_requires_write_not_read_permission() {
    let mut w = world();
    file(&mut w, "same", "old");
    ok(&mut w, "cat same > same", "");
    ok(&mut w, "cat same", "");
    file(&mut w, "append", "old");
    w.vfs.chmod("/home/kali/append", "kali", 0o200).unwrap();
    ok(&mut w, "echo new >> append", "");
    assert_eq!(w.vfs.read("/home/kali/append", "root").unwrap(), "oldnew\n");
    let result = execute(&mut w, "sudo false > Documents/false.txt");
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.is_empty());
    assert!(w.vfs.nodes.contains_key("/home/kali/Documents/false.txt"));
    assert_ne!(
        execute(&mut w, "sudo echo denied > /etc/forbidden").exit_code,
        0
    );
    assert!(!w.vfs.nodes.contains_key("/etc/forbidden"));
}

#[test]
fn ssh_reads_remote_files_and_redirect_target_stays_in_the_origin_host() {
    let mut w = world();
    file(&mut w, "a", "local");
    ok(&mut w, "cd Downloads", "");
    assert!(w.terminal.env.contains_key("OLDPWD"));
    ok(&mut w, "cd /home/kali", "");
    let login = execute(&mut w, "ssh vex@vex.local lab-only > Documents/login.txt");
    assert_eq!(login.exit_code, 0, "{}", login.stderr);
    ok(&mut w, "printenv PWD", "/home/kali\n");
    assert!(!w.terminal.env.contains_key("OLDPWD"));
    assert!(w.vfs.nodes.contains_key("/home/kali/Documents/login.txt"));
    let remote = w.terminal.host.clone().unwrap();
    assert!(!w.network.hosts[&remote]
        .files
        .nodes
        .contains_key("/home/kali/Documents/login.txt"));
    w.network
        .hosts
        .get_mut(&remote)
        .unwrap()
        .files
        .write("/home/kali/a", "remote\nlast", "vex")
        .unwrap();
    ok(&mut w, "cat a", "remote\nlast");
    ok(&mut w, "tail -n1 a", "last");
    ok(&mut w, "cp /etc/web.conf /home/kali/web-backup.conf", "");
    assert!(!w.vfs.nodes.contains_key("/home/kali/web-backup.conf"));
    assert_eq!(w.vfs.read("/home/kali/a", "kali").unwrap(), "local");
}

#[test]
fn sudo_cd_cannot_leave_the_session_in_an_inaccessible_directory() {
    let mut w = world();
    assert_ne!(execute(&mut w, "sudo cd /root").exit_code, 0);
    assert_eq!(w.terminal.cwd, "/home/kali");
    assert_eq!(w.terminal.user, "kali");
    w.validate().unwrap();
}

#[test]
fn excessive_text_output_is_bounded_and_does_not_change_files() {
    let mut w = world();
    file(&mut w, "large", &"x".repeat(1024 * 1024));
    let result = execute(&mut w, "cat large large large large large");
    assert_ne!(result.exit_code, 0);
    assert!(result.stderr.contains("4 MiB"));
    assert_eq!(result.stderr_bytes, result.stderr.as_bytes());
    assert_eq!(result.stdout, "x".repeat(4 * 1024 * 1024 - 4096));
    assert_eq!(w.vfs.nodes["/home/kali/large"].content.len(), 1024 * 1024);
}

#[test]
fn audited_manuals_describe_the_implemented_subset() {
    let mut w = world();
    for command in ["pwd", "cd", "cp", "mv", "rm"] {
        let help = execute(&mut w, &format!("{command} --help"));
        assert_eq!(help.exit_code, 0, "{command}");
        assert!(help.stdout.contains("virtual subset"));
        ok(&mut w, &format!("man {command}"), &help.stdout);
    }
    let head = execute(&mut w, "head --help");
    assert_eq!(head.exit_code, 0);
    assert!(head.stdout.contains("--zero-terminated"));
    assert!(head.stdout.contains("NUM may have a multiplier suffix:"));
    ok(&mut w, "man head", &head.stdout);
    let tail = execute(&mut w, "tail --help");
    assert_eq!(tail.exit_code, 0);
    assert_eq!(
        tail.stdout,
        include_str!("coreutils/messages/tail-help.txt").replace("{invocation}", "tail")
    );
    let manual = execute(&mut w, "man tail");
    assert_eq!(manual.exit_code, 0);
    assert!(manual
        .stdout
        .contains("virtual inode and pathname subscriptions"));
    assert!(manual.stdout.contains("SIGPIPE"));
    let help = execute(&mut w, "cat --help");
    assert_eq!(help.exit_code, 0);
    assert!(help
        .stdout
        .contains("Concatenate FILE(s) to standard output."));
    let manual = execute(&mut w, "man cat");
    assert_eq!(manual.exit_code, 0);
    assert!(manual.stdout.contains("canonical virtual TTY"));
    assert!(manual.stdout.contains("SIGPIPE"));
    assert!(execute(&mut w, "man ip")
        .stdout
        .contains("still require a vertical audit"));
}

#[test]
fn partial_read_redirect_is_saved_by_the_real_service_boundary() {
    let mut game =
        crate::service::GameService::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
    game.new_game(1, "neo", "pc", false).unwrap();
    game.mutate(
        |_, w, _| {
            w.vfs
                .write("/home/kali/Documents/personal.txt", "personal", "kali")
        },
        false,
    )
    .unwrap();
    let result = game
        .mutate(
            |_, w, _| {
                Ok(execute(
                    w,
                    "cat Documents/personal.txt missing > Documents/collected.txt",
                ))
            },
            false,
        )
        .unwrap();
    assert_eq!(result.exit_code, 1);
    assert!(result.stdout.is_empty());
    assert!(result.stderr.contains("missing"));
    game.end_session().unwrap();
    game.load(1, false).unwrap();
    assert_eq!(
        game.world()
            .unwrap()
            .vfs
            .read("/home/kali/Documents/collected.txt", "kali")
            .unwrap(),
        "personal"
    );
    assert_eq!(
        game.world()
            .unwrap()
            .vfs
            .read("/home/kali/Documents/personal.txt", "kali")
            .unwrap(),
        "personal"
    );
}
