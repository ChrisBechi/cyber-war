use crate::{terminal, world::WorldState};
fn world() -> WorldState {
    WorldState::new("kali", "lifeos").unwrap()
}

#[test]
fn foundation_export_and_sudo_regressions() {
    // Preserve the original M1C shell fixtures alongside structured GNU process cases.
    let mut w = world();
    let r = terminal::execute(&mut w, "export SAMPLE=value; /usr/bin/printenv SAMPLE");
    assert_eq!(
        (r.stdout.as_str(), r.stderr.as_str(), r.exit_code),
        ("value\n", "", 0)
    );
    let r = terminal::execute(&mut w, "/usr/bin/sudo /usr/bin/whoami");
    assert_eq!(
        (r.stdout.as_str(), r.stderr.as_str(), r.exit_code),
        ("root\n", "", 0)
    );
    assert_eq!(w.terminal.user, "kali");
    let r = terminal::execute(&mut w, "export Z_ORDER=z A_ORDER=a; printenv");
    assert!(r.stdout.ends_with("SAMPLE=value\nZ_ORDER=z\nA_ORDER=a\n"));
    let r = terminal::execute(
        &mut w,
        "export Z_ORDER=updated; unset A_ORDER; export A_ORDER=last; printenv",
    );
    assert!(r
        .stdout
        .ends_with("SAMPLE=value\nZ_ORDER=updated\nA_ORDER=last\n"));
    let before = w.terminal.shell.environment_order.clone();
    let r = terminal::execute(&mut w, "Z_TEMP=z A_TEMP=a printenv");
    assert!(r.stdout.ends_with("Z_TEMP=z\nA_TEMP=a\n"));
    assert_eq!(w.terminal.shell.environment_order, before);
    assert_eq!(
        terminal::execute(&mut w, "POSIXLY_CORRECT=1; dirname a/b --zero").stdout,
        "a\0"
    );
    assert_eq!(
        terminal::execute(&mut w, "export POSIXLY_CORRECT; dirname a/b --zero").stdout,
        "a\n.\n"
    );
}

#[test]
fn foundation_effective_identity_and_raw_stderr() {
    let mut w = world();
    w.vfs
        .write(
            "/etc/passwd",
            "analyst:x:1000:1000::/home/kali:/bin/sh\n",
            "root",
        )
        .unwrap();
    assert_eq!(
        terminal::execute(&mut w, "USER=spoof LOGNAME=spoof whoami").stdout,
        "analyst\n"
    );
    w.vfs.write("/etc/passwd", "", "root").unwrap();
    let r = terminal::execute(&mut w, "whoami");
    assert_eq!(
        (r.stdout.as_str(), r.stderr.as_str(), r.exit_code),
        ("", "whoami: cannot find name for user ID 1000\n", 1)
    );
    let r = terminal::execute(&mut w, "/usr/bin/basename '-é' 2> error-bytes");
    assert_eq!(r.exit_code, 1);
    assert!(r.stderr_bytes.is_empty());
    let bytes = crate::archive::bytes(&w, "/home/kali/error-bytes", "kali").unwrap();
    assert!(bytes.windows(3).any(|s| s == [b'\'', 0xc3, b'\'']));
    assert!(std::str::from_utf8(&bytes).is_err());
    for name in ["basename", "dirname", "printenv", "whoami"] {
        let args = vec!["--version".to_owned()];
        let out = super::execute(&mut w, name, &args, "kali")
            .unwrap()
            .unwrap();
        assert!(out.stdout.starts_with(&format!("{name} (GNU coreutils) ")));
    }
    w.packages.installed.remove("coreutils");
    for name in ["basename", "dirname", "printenv", "whoami"] {
        assert_eq!(
            terminal::execute(&mut w, &format!("/usr/bin/{name} --help")).exit_code,
            126
        );
    }
}

#[test]
fn foundation_paths_options_and_environment() {
    let mut w = world();
    for (script, expected, status) in [
        ("/usr/bin/basename -az /a/b /c/d", "b\0d\0", 0),
        ("/usr/bin/basename -s .txt /a/a.txt /b/b.txt", "a\nb\n", 0),
        ("/usr/bin/basename ''", "\n", 0),
        ("/usr/bin/dirname /a/b/ //// a ''", "/a\n/\n.\n.\n", 0),
        (
            "/usr/bin/env -i X=yes /usr/bin/printenv X MISSING",
            "yes\n",
            1,
        ),
        ("/usr/bin/whoami", "kali\n", 0),
    ] {
        let r = terminal::execute(&mut w, script);
        assert_eq!(
            (r.stdout, r.exit_code),
            (expected.into(), status),
            "{script}: {}",
            r.stderr
        );
    }
    assert!(terminal::execute(&mut w, "/usr/bin/basename --bogus a")
        .stderr
        .contains("unrecognized option"));
    assert_ne!(
        terminal::execute(&mut w, "/usr/bin/whoami extra").exit_code,
        0
    );
    assert_eq!(terminal::execute(&mut w, "printenv X").exit_code, 1);
    w.packages.installed.remove("coreutils");
    assert_eq!(
        terminal::execute(&mut w, "/usr/bin/basename /a/b").exit_code,
        126
    );
}

#[test]
fn byte_pipeline_redirection_append_and_links() {
    let mut w = world();
    let bytes = vec![0, 255, 128, 0xc3, 0xa9, b'\n', b'z'];
    crate::archive::write_bytes(
        &mut w,
        "/home/kali/input",
        bytes.clone(),
        "kali",
        bytes.len() as u64,
    )
    .unwrap();
    for script in [
        "cat input | tee copy | head -c 4 > out",
        "cat input >> out",
        "ln out alias",
    ] {
        let r = terminal::execute(&mut w, script);
        assert_eq!(r.exit_code, 0, "{script}: {}", r.stderr);
    }
    let expected = [&bytes[..4], &bytes[..]].concat();
    assert_eq!(
        crate::archive::bytes(&w, "/home/kali/alias", "kali")
            .unwrap()
            .as_ref(),
        &expected
    );
    let r = terminal::execute(&mut w, "cat input | base64 -w0 | base64 -d > decoded");
    assert_eq!(r.exit_code, 0, "{}", r.stderr);
    assert_eq!(
        crate::archive::bytes(&w, "/home/kali/decoded", "kali")
            .unwrap()
            .as_ref(),
        &bytes
    );
    let r = terminal::execute(&mut w, "cat input | wc -c");
    assert_eq!(r.stdout, "7\n", "{}", r.stderr);
    assert_eq!(
        terminal::execute(&mut w, "/usr/bin/yes | /usr/bin/head -n 3").stdout,
        "y\ny\ny\n"
    );
    assert!(w
        .processes
        .iter()
        .all(|p| !["cat", "head", "tee", "yes"].contains(&p.name.as_str())));
}

#[test]
fn parser_fuzz_and_virtual_host_paths() {
    let mut w = world();
    for arg in [
        "-ðŸ’£",
        "--unknown",
        "--bytes=9999999999999999999999999",
        "--bytes=",
        "--bytes=--1",
        "--bytes=Ù¡",
    ] {
        let r = terminal::execute_parts(
            &mut w,
            Ok(vec!["head".into(), arg.into(), "/etc/passwd".into()]),
        );
        assert_ne!(r.exit_code, 0, "{arg}");
    }
    for path in [r"C:\Windows\win.ini", r"\\server\share", r"..\..\host"] {
        let r = terminal::execute_parts(&mut w, Ok(vec!["cat".into(), path.into()]));
        assert_eq!(r.exit_code, 1);
        assert!(r.stdout.is_empty());
    }
    w.vfs
        .write(
            "/etc/passwd",
            "kali:x:1000:1000::/home/kali:/bin/bash\n",
            "root",
        )
        .unwrap();
    assert!(terminal::execute(&mut w, "cat /etc/passwd")
        .stdout
        .contains("kali"));
}

#[test]
fn finite_commands_and_options_are_independent_of_shell_rendering() {
    let mut w = world();
    w.terminal.io.stdin_tty = false;
    w.terminal.stdin_bytes = Some(vec![0, 255, 195, 169]);
    let result = super::execute(&mut w, "head", &["-c".into(), "2".into()], "kali")
        .unwrap()
        .unwrap();
    assert_eq!(result.binary, Some(vec![0, 255]));
    let result = super::execute(
        &mut w,
        "basename",
        &["-s".into(), "--help".into(), "file--help".into()],
        "kali",
    )
    .unwrap()
    .unwrap();
    assert_eq!(result.stdout, "file\n");
    for seed in 0..512u32 {
        let arg = format!(
            "--{}{}",
            char::from_u32(0x400 + (seed % 256)).unwrap(),
            seed.wrapping_mul(2654435761)
        );
        for name in ["cat", "head", "tail", "base64", "basename", "dirname"] {
            let result = super::execute(&mut w, name, std::slice::from_ref(&arg), "kali")
                .unwrap()
                .unwrap();
            assert_ne!(result.status, 0, "{name} {arg}");
        }
    }
}
#[test]
fn binary_aliases_unlinked_handles_and_split_utf8_preserve_identity() {
    let mut w = world();
    let h = w
        .vfs
        .open(
            "/home/kali/a",
            crate::vfs::OpenFlags {
                write: true,
                create: true,
                ..Default::default()
            },
            0o600,
            "kali",
        )
        .unwrap();
    super::io::write_handle(&mut w, h, &[0xc3]).unwrap();
    w.vfs.link("/home/kali/a", "/home/kali/b", "kali").unwrap();
    w.vfs.unlink("/home/kali/a", "kali").unwrap();
    super::io::write_handle(&mut w, h, &[0xa9]).unwrap();
    assert_eq!(w.vfs.read("/home/kali/b", "kali").unwrap(), "é");
    w.vfs.close(h).unwrap();
    w.vfs.check_invariants().unwrap();
}

#[test]
fn text_adapter_rejects_unpersistable_payloads_and_empty_writes_do_not_extend() {
    let mut w = world();
    w.vfs.write("/home/kali/text", "é", "kali").unwrap();
    let h = w
        .vfs
        .open(
            "/home/kali/text",
            crate::vfs::OpenFlags {
                write: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    assert!(w.vfs.write_handle(h, b"x").is_err());
    assert_eq!(w.vfs.read("/home/kali/text", "kali").unwrap(), "é");
    assert!(w.vfs.write_handle(h, &vec![b'x'; 1024 * 1024 + 1]).is_err());
    assert_eq!(w.vfs.read("/home/kali/text", "kali").unwrap(), "é");
    w.vfs.seek(h, 100).unwrap();
    let before = w.vfs.stat("/home/kali/text", "kali").unwrap().modified_at;
    super::io::write_handle(&mut w, h, b"").unwrap();
    let node = w.vfs.stat("/home/kali/text", "kali").unwrap();
    assert_eq!(node.logical_size(), 2);
    assert_eq!(node.modified_at, before);
    w.vfs.seek(h, 0).unwrap();
    assert_eq!(w.vfs.write_handle(h, b"ok").unwrap(), 2);
    assert_eq!(w.vfs.read("/home/kali/text", "kali").unwrap(), "ok");
    w.vfs.close(h).unwrap();
    w.vfs.check_invariants().unwrap();
}
