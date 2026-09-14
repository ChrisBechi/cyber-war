use crate::{terminal::execute, world::WorldState};
fn world() -> WorldState {
    WorldState::new("neo", "pc").unwrap()
}
fn script(w: &mut WorldState, name: &str, content: &str) {
    w.vfs
        .write(&format!("/home/kali/{name}"), content, "kali")
        .unwrap();
}
#[test]
fn quote_expansion_preserves_arguments_without_reparsing_operators() {
    let mut w = world();
    assert_eq!(execute(&mut w, "VALUE='a b > stolen'").exit_code, 0);
    assert_eq!(
        execute(&mut w, "echo '$VALUE' \"$VALUE\" \"${VALUE}\" ").stdout,
        "$VALUE a b > stolen a b > stolen\n"
    );
    assert!(!w.vfs.nodes.contains_key("/home/kali/stolen"));
    assert_eq!(
        execute(&mut w, "echo \"escaped \\\"quote\\\"\" a\\ b").stdout,
        "escaped \"quote\" a b\n"
    );
    assert_eq!(
        execute(&mut w, "echo literal#hash # comment").stdout,
        "literal#hash\n"
    );
    assert_eq!(execute(&mut w, "false").exit_code, 1);
    assert_eq!(execute(&mut w, "echo $?").stdout, "1\n");
    for text in [
        "echo $(whoami)",
        "echo \"`whoami`\"",
        "echo ${VALUE:-fallback}",
    ] {
        assert_ne!(execute(&mut w, text).exit_code, 0);
    }
}

#[test]
fn assignment_values_do_not_split_and_control_bytes_never_become_operators() {
    let mut w = world();
    execute(&mut w, "VALUE='a b > stolen'");
    assert_eq!(execute(&mut w, "COPY=$VALUE").exit_code, 0);
    assert_eq!(execute(&mut w, "export SHARED=$VALUE").exit_code, 0);
    assert_eq!(w.terminal.env["COPY"], "a b > stolen");
    assert_eq!(w.terminal.env["SHARED"], "a b > stolen");
    w.terminal.env.insert("CONTROL".into(), "\0>".into());
    assert_ne!(execute(&mut w, "echo text $CONTROL stolen").exit_code, 0);
    assert_ne!(execute(&mut w, "echo text \\\0> stolen").exit_code, 0);
    assert!(!w.vfs.nodes.contains_key("/home/kali/stolen"));
    script(&mut w, "editor.sh", "nano note.txt\n");
    assert_ne!(execute(&mut w, "source editor.sh").exit_code, 0);
    assert!(w.terminal.nano.is_none());
    assert!(w.terminal.foreground.is_none());
}
#[test]
fn child_shells_isolate_context_and_only_inherit_exports() {
    let mut w = world();
    execute(&mut w, "PRIVATE=hidden");
    execute(&mut w, "export PUBLIC=visible");
    script(&mut w, "child.sh", "echo \"$PRIVATE:$PUBLIC\"\ncd Documents\nLOCAL=value\necho persisted > child.txt\nexit 7\necho unreachable\n");
    let out = execute(&mut w, "bash child.sh");
    assert_eq!(out.exit_code, 7);
    assert_eq!(out.stdout, ":visible\n");
    assert!(out.stderr.is_empty());
    assert_eq!(w.terminal.cwd, "/home/kali");
    assert!(!w.terminal.env.contains_key("LOCAL"));
    assert_eq!(
        w.vfs
            .read("/home/kali/Documents/child.txt", "kali")
            .unwrap(),
        "persisted\n"
    );
}
#[test]
fn source_shares_state_and_return_does_not_exit_the_parent() {
    let mut w = world();
    script(
        &mut w,
        "library.sh",
        "export LIB=loaded\necho \"$0:$1\"\nreturn 9\necho unreachable\n",
    );
    script(
        &mut w,
        "main.sh",
        "source library.sh nested\necho \"$?:$LIB:$1\"\n",
    );
    let out = execute(&mut w, "bash main.sh outer");
    assert_eq!(out.exit_code, 0);
    assert_eq!(out.stdout, "main.sh:nested\n9:loaded:outer\n");
    assert!(!w.terminal.env.contains_key("LIB"));
    assert_eq!(execute(&mut w, ". library.sh interactive").exit_code, 9);
    assert_eq!(w.terminal.env["LIB"], "loaded");
    script(&mut w, "exit.sh", "echo before-exit\nexit 4\n");
    script(&mut w, "outer.sh", "source exit.sh\necho unreachable\n");
    let out = execute(&mut w, "bash outer.sh");
    assert_eq!(out.exit_code, 4);
    assert_eq!(out.stdout, "before-exit\n");
}
#[test]
fn positional_at_keeps_empty_and_space_arguments_and_c_uses_supplied_zero() {
    let mut w = world();
    script(&mut w, "args.sh", "echo \"$0:$#:${10}:$9\"\necho \"$@\"\n");
    let out = execute(&mut w, "bash args.sh 'a b' '' c d e f g h i j");
    assert_eq!(out.stdout, "args.sh:10:j:i\na b  c d e f g h i j\n");
    assert_eq!(
        execute(&mut w, "bash -c 'echo \"$0:$1\"' name 'two words'").stdout,
        "name:two words\n"
    );
    let env = Default::default();
    assert_eq!(
        crate::shell::words("echo \"$@\"", &env, "bash", &[], 0).unwrap(),
        ["echo"]
    );
    assert_eq!(
        crate::shell::words("echo \"$@\"", &env, "bash", &["a b".into(), "".into()], 0).unwrap(),
        ["echo", "a b", ""]
    );
}
#[test]
fn failures_keep_stdout_stderr_and_continue_without_implicit_errexit() {
    let mut w = world();
    script(
        &mut w,
        "errors.sh",
        "echo before\ncat missing\necho \"status=$?\"\nfalse\n",
    );
    let out = execute(&mut w, "bash errors.sh > result.txt");
    assert_eq!(out.exit_code, 1);
    assert!(out.stdout.is_empty());
    assert!(out.stderr.contains("missing"));
    assert_eq!(
        w.vfs.read("/home/kali/result.txt", "kali").unwrap(),
        "before\nstatus=1\n"
    );
    script(&mut w, "recursion.sh", "bash recursion.sh\n");
    let out = execute(&mut w, "bash recursion.sh");
    assert_ne!(out.exit_code, 0);
    assert!(out.stderr.contains("nesting"));
    assert_eq!(w.terminal.shell_depth, 0);
    script(&mut w, "unsupported.sh", "#!/usr/bin/python\necho no\n");
    assert_ne!(execute(&mut w, "bash unsupported.sh").exit_code, 0);
    assert!(w.terminal.nano.is_none());
}
#[test]
fn service_flags_return_statuses_and_processes_follow_real_virtual_events() {
    let mut w = world();
    assert_eq!(execute(&mut w, "systemctl is-active ssh").exit_code, 3);
    assert_eq!(execute(&mut w, "systemctl is-enabled ssh").exit_code, 1);
    assert_eq!(
        execute(&mut w, "sudo systemctl enable --now ssh").exit_code,
        0
    );
    let pid = w
        .processes
        .iter()
        .find(|p| p.name == "ssh" && p.running)
        .unwrap()
        .pid;
    assert_eq!(execute(&mut w, &format!("sudo kill -0 {pid}")).exit_code, 0);
    assert_eq!(execute(&mut w, "systemctl -q is-active ssh").stdout, "");
    assert_ne!(execute(&mut w, &format!("kill -9 {pid}")).exit_code, 0);
    assert_eq!(execute(&mut w, "sudo pkill -x ssh").exit_code, 0);
    assert_eq!(execute(&mut w, "systemctl is-active ssh").exit_code, 3);
    let log = execute(&mut w, "journalctl -u ssh -n 1");
    assert!(log.stdout.contains("signal 15"));
    assert!(!log.stdout.contains("health"));
    assert_ne!(
        execute(&mut w, "sudo systemctl enable unknown").exit_code,
        0
    );
    for command in [
        "sudo apt --fix-broken install nmap",
        "systemctl --failed",
        "journalctl -f",
        "kill -STOP 1",
        "ps --forest",
    ] {
        assert_ne!(execute(&mut w, command).exit_code, 0);
    }
}
