use super::*;
use crate::terminal::{execute as run, execute_interactive};
fn world() -> WorldState {
    WorldState::new("kali", "pc").unwrap()
}
fn check(world: &mut WorldState, source: &str, stdout: &str, status: i32) {
    let result = run(world, source);
    assert_eq!(
        (
            result.stdout.as_str(),
            result.stderr.as_str(),
            result.exit_code
        ),
        (stdout, "", status),
        "{source}"
    );
}
#[test]
fn ast_precedence_spans_empty_words_and_incomplete_input() {
    use syntax::*;
    for source in ["echo '", "echo x|", "echo x &&", "echo $(pwd", "echo \\"] {
        assert!(
            matches!(parse(source), ParseResult::Incomplete(_)),
            "{source}"
        );
    }
    for source in ["|echo", "echo;;pwd", "echo 99>x", "echo ${x:-y}", "echo\0"] {
        assert!(matches!(parse(source), ParseResult::Error(_)), "{source}");
    }
    let ParseResult::Complete(ast) = parse("echo a|cat&&false||echo 'x y';echo z") else {
        panic!("AST")
    };
    assert_eq!(ast.items.len(), 2);
    assert_eq!(ast.items[0].pipelines.len(), 3);
    assert_eq!(ast.items[0].pipelines[0].1.commands.len(), 2);
    assert_eq!(
        ast.items[0].pipelines[0].1.commands[0].span,
        Span { start: 0, end: 6 }
    );
    let mut w = world();
    check(
        &mut w,
        "false&&echo no||echo yes;true||echo no&&echo next",
        "yes\nnext\n",
        0,
    );
    check(&mut w, "false;echo $?;true|false;echo $?", "1\n1\n", 0);
    check(&mut w, "echo 'a\nb'\necho c\\\nd", "a\nb\ncd\n", 0);
    assert!(execute_interactive(&mut w, "echo 'hello").shell_incomplete);
    let result = execute_interactive(&mut w, "world'");
    assert!(!result.shell_incomplete);
    assert_eq!(result.stdout, "hello\nworld\n");
    assert!(execute_interactive(&mut w, "echo '").shell_incomplete);
    assert_eq!(execute_interactive(&mut w, "\x03").exit_code, 130);
    assert!(w.terminal.shell.continuation.is_empty());
}
#[test]
fn expansion_environment_assignments_ifs_and_substitution() {
    let mut w = world();
    assert_eq!(
        words("echo \"\"\"$@\"", &Default::default(), "bash", &[], 0).unwrap(),
        ["echo", ""]
    );
    assert_eq!(
        words("echo \"$@\"\"\"", &Default::default(), "bash", &[], 0).unwrap(),
        ["echo", ""]
    );
    assert_eq!(
        words(
            "echo pre\"$@\"post",
            &Default::default(),
            "bash",
            &["a".into(), "b".into()],
            0
        )
        .unwrap(),
        ["echo", "prea", "bpost"]
    );
    check(
        &mut w,
        "A='one two';B=$A C=$B;echo \"$B:$C\"",
        "one two:one two\n",
        0,
    );
    check(
        &mut w,
        "A=temporary bash -c 'echo $A';echo $A",
        "temporary\none two\n",
        0,
    );
    check(
        &mut w,
        "export X=shared;unset X;echo \"[$X]\";bash -c 'echo \"[$X]\"'",
        "[]\n[]\n",
        0,
    );
    check(&mut w, "unset HOME;echo \"[$HOME]\"", "[]\n", 0);
    check(
        &mut w,
        "HOME=/home/kali;echo ~ '~/x'",
        "/home/kali ~/x\n",
        0,
    );
    check(
        &mut w,
        "V='a::b:';IFS=:;bash -c 'echo \"$#\"' _ $V",
        "3\n",
        0,
    );
    check(
        &mut w,
        "unset IFS;echo \"$(echo $(pwd))\"",
        "/home/kali\n",
        0,
    );
    check(&mut w, "VALUE=$(echo a;echo);echo \"<$VALUE>\"", "<a>\n", 0);
    check(&mut w, "VALUE=$(false);echo $?", "1\n", 0);
    check(
        &mut w,
        "HERE=$(cd Documents;export INNER=x;pwd);echo \"$HERE:$INNER\";pwd",
        "/home/kali/Documents:\n/home/kali\n",
        0,
    );
    let result = run(&mut w, "echo \"$(echo warning >&2;echo text)\"");
    assert_eq!(
        (result.stdout.as_str(), result.stderr.as_str()),
        ("text\n", "warning\n")
    );
    check(&mut w, "bash -c 'echo \"$#\"' _ '' \"\" a' b'c", "3\n", 0);
    check(
        &mut w,
        r#"bash -c 'bash -c '\''echo "$#"'\'' _ "$@"'"#,
        "0\n",
        0,
    );
}
#[test]
fn virtual_globs_are_sorted_quote_aware_and_permission_checked() {
    let mut w = world();
    run(
        &mut w,
        "mkdir glob;cd glob;touch b.log a.log a.txt .hidden;mkdir nest;touch nest/c.log",
    );
    check(&mut w, "echo *.log", "a.log b.log\n", 0);
    check(
        &mut w,
        "echo ?.log [ab].log [a-z].txt [!a].log",
        "a.log b.log a.log b.log a.txt b.log\n",
        0,
    );
    check(
        &mut w,
        "echo '*.log' \\*.log missing* .h*",
        "*.log *.log missing* .hidden\n",
        0,
    );
    check(
        &mut w,
        "echo nest/*.log ../glob/*.txt",
        "nest/c.log ../glob/a.txt\n",
        0,
    );
    check(&mut w, "X='*.log';echo $X \"$X\"", "a.log b.log *.log\n", 0);
    let result = run(&mut w, "echo * > *.log");
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("ambiguous redirect"));
}
#[test]
fn streaming_three_stages_early_close_eof_and_large_input() {
    let mut w = world();
    check(&mut w, "yes|head -n 3", "y\ny\ny\n", 0);
    check(&mut w, "yes alpha|head -n 100000|wc -l", "100000\n", 0);
    assert!(crate::shell_pipeline::streams::last_peak() <= 65536);
    check(&mut w, "yes|head -n 0", "", 0);
    check(&mut w, "yes|head -n 5 >five;cat five", "y\ny\ny\ny\ny\n", 0);
    check(&mut w, "echo alpha|cat|cat", "alpha\n", 0);
    check(&mut w, "echo alpha|cat </dev/null", "", 0);
    check(&mut w, "false|true", "", 0);
    check(&mut w, "true|false", "", 1);
    assert!(!w.processes.iter().any(|p| p.pid >= 10000));
    w.terminal.stdin = Some("x\n".repeat(100000));
    check(&mut w, "cat|cat|wc -l", "100000\n", 0);
    assert!(w.terminal.io.stdin_tty);
}
#[test]
fn stream_control_has_bounded_input_eof_cancel_and_cleanup() {
    let mut w = world();
    let registration = control::register("foundation-input");
    assert!(!control::input("foundation-input", Some("x".repeat(65537))));
    assert!(control::input("foundation-input", Some("typed\n".into())));
    assert!(control::input("foundation-input", None));
    let result = control::run(&registration, || run(&mut w, "cat|cat"));
    assert_eq!((result.stdout.as_str(), result.exit_code), ("typed\n", 0));
    drop(registration);
    assert!(!control::input("foundation-input", None));
    let registration = control::register("foundation-cancel");
    control::cancel("foundation-cancel");
    let result = control::run(&registration, || run(&mut w, "yes|cat"));
    assert_eq!(result.exit_code, 130);
    drop(registration);
    let registration = control::register("foundation-running");
    let worker =
        std::thread::spawn(move || control::run(&registration, || run(&mut world(), "cat|cat")));
    assert!(control::input(
        "foundation-running",
        Some("before\n".into())
    ));
    control::cancel("foundation-running");
    assert_eq!(worker.join().unwrap().exit_code, 130);
}
#[test]
fn command_resolution_uses_virtual_path_and_modes() {
    let mut w = world();
    w.vfs
        .write("/home/kali/tool", "#!/bin/bash\necho \"tool:$1\"\n", "kali")
        .unwrap();
    w.vfs.chmod("/home/kali/tool", "kali", 0o755).unwrap();
    check(
        &mut w,
        "PATH=/home/kali:$PATH;tool argument",
        "tool:argument\n",
        0,
    );
    check(&mut w, "./tool relative", "tool:relative\n", 0);
    w.vfs.chmod("/home/kali/tool", "kali", 0o644).unwrap();
    assert_eq!(run(&mut w, "tool").exit_code, 126);
    assert_eq!(run(&mut w, "missing-program").exit_code, 127);
    check(
        &mut w,
        "PATH=;echo builtin;cd Documents;pwd",
        "builtin\n/home/kali/Documents\n",
        0,
    );
    assert_eq!(run(&mut w, "cat").exit_code, 127);
}
#[test]
fn runtime_is_not_serialized_and_host_syntax_has_no_effects() {
    let mut w = world();
    run(&mut w, "A=secret;export A");
    let data = serde_json::to_value(&w).unwrap();
    assert!(data["terminal"].get("shell").is_none());
    let loaded: WorldState = serde_json::from_value(data).unwrap();
    assert!(loaded.terminal.env.is_empty());
    assert!(loaded.terminal.shell.continuation.is_empty());
    for source in [
        "echo x > C:\\Windows\\x",
        "cmd.exe /c dir",
        "powershell Get-Process",
        "echo $(cmd.exe /c dir)",
        "echo x > /../../etc/protected",
    ] {
        let result = run(&mut w, source);
        assert!(
            result.exit_code != 0 || !result.stderr.is_empty(),
            "{source}"
        );
    }
    let before = w.vfs.nodes.clone();
    assert_eq!(
        run(&mut w, "echo x > never;echo 'unterminated").exit_code,
        2
    );
    assert_eq!(w.vfs.nodes, before);
}
#[test]
fn old_baseline_save_gains_yes_binding_once_without_reinstalling_deleted_files() {
    let mut w = world();
    w.vfs.remove("/usr/bin/yes", "root", false).unwrap();
    w.packages.ownership.remove("/usr/bin/yes");
    w.packages
        .installed
        .get_mut("coreutils")
        .unwrap()
        .definition
        .files
        .retain(|file| file.path != "/usr/bin/yes");
    crate::packages::state::initialize(&mut w).unwrap();
    check(&mut w, "yes|head -n1", "y\n", 0);
    let snapshot = serde_json::to_string(&w).unwrap();
    crate::packages::state::initialize(&mut w).unwrap();
    assert_eq!(serde_json::to_string(&w).unwrap(), snapshot);
    w.vfs.remove("/usr/bin/yes", "root", false).unwrap();
    crate::packages::state::initialize(&mut w).unwrap();
    assert!(!w.vfs.nodes.contains_key("/usr/bin/yes"));
}
#[test]
fn parser_fuzz_limits_do_not_execute_or_panic() {
    let alphabet = [
        'a', ' ', '\'', '"', '\\', '$', '{', '}', '(', ')', '<', '>', '|', '&', ';', '\n', 'é',
    ];
    let mut seed = 19u64;
    for _ in 0..10000 {
        let mut text = String::new();
        for _ in 0..64 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            text.push(alphabet[(seed >> 32) as usize % alphabet.len()]);
        }
        let _ = syntax::parse(&text);
    }
    assert!(matches!(
        syntax::parse(&"echo x;".repeat(10000)),
        syntax::ParseResult::Error(_)
    ));
    assert!(matches!(
        syntax::parse(&format!("echo {}x{}", "$(echo ".repeat(30), ")".repeat(30))),
        syntax::ParseResult::Error(_)
    ));
}

#[test]
fn expansion_limits_are_checked_before_accumulating_unbounded_fields() {
    let mut w = world();
    w.terminal.env.insert("LARGE".into(), "x".repeat(65536));
    let result = run(&mut w, &format!("echo {}", "$LARGE".repeat(32)));
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("expanded argument limit"));
    let result = run(&mut w, &format!("echo {}*", "a".repeat(4096)));
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("glob pattern limit"));
    check(&mut w, "echo a\u{2000}b", "a\u{2000}b\n", 0);
}
