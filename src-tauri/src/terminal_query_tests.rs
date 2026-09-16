use crate::{terminal::execute, world::WorldState};
fn world() -> WorldState {
    let mut w = WorldState::new("neo", "pc").unwrap();
    w.vfs.mkdir("/home/kali/data", "kali").unwrap();
    w.vfs
        .write("/home/kali/data/a.txt", "alpha\nBeta\na+b\nab\n", "kali")
        .unwrap();
    w.vfs.write("/home/kali/data/.hidden", "x", "kali").unwrap();
    w
}
#[test]
fn grep_basic_extended_fixed_counts_statuses_and_partial_errors() {
    let mut w = world();
    assert_eq!(
        execute(&mut w, "grep -n '^alpha$' data/a.txt").stdout,
        "1:alpha\n"
    );
    assert_eq!(execute(&mut w, "grep 'a+b' data/a.txt").stdout, "a+b\n");
    assert_eq!(execute(&mut w, "grep -Ex 'a+b' data/a.txt").stdout, "ab\n");
    assert_eq!(execute(&mut w, "grep -F 'a+b' data/a.txt").stdout, "a+b\n");
    assert_eq!(
        execute(&mut w, "grep -ic -e alpha -e beta data/a.txt").stdout,
        "2\n"
    );
    assert_eq!(execute(&mut w, "grep -m1 a data/a.txt").stdout, "alpha\n");
    assert_eq!(execute(&mut w, "grep absent data/a.txt").exit_code, 1);
    assert_eq!(execute(&mut w, "grep '[' data/a.txt").exit_code, 2);
    let partial = execute(&mut w, "grep -hn alpha missing data/a.txt");
    assert_eq!(partial.exit_code, 2);
    assert_eq!(partial.stdout, "1:alpha\n");
    assert!(partial.stderr.contains("missing"));
    assert_eq!(
        execute(&mut w, "grep -q alpha missing data/a.txt").exit_code,
        0
    );
    assert_eq!(execute(&mut w, "grep -c -v Beta data/a.txt").stdout, "3\n");
    assert_eq!(execute(&mut w, "grep -w alp data/a.txt").exit_code, 1);
    assert_eq!(
        execute(&mut w, "grep -L absent data/a.txt").stdout,
        "data/a.txt\n"
    );
}
#[test]
fn ls_hidden_dot_entries_file_operands_sort_and_symbolic_modes() {
    let mut w = world();
    assert_eq!(execute(&mut w, "ls data").stdout, "a.txt\n");
    assert_eq!(execute(&mut w, "ls -A data").stdout, ".hidden\na.txt\n");
    assert_eq!(
        execute(&mut w, "ls -a data").stdout,
        ".\n..\n.hidden\na.txt\n"
    );
    assert_eq!(execute(&mut w, "ls -dF data").stdout, "data/\n");
    assert!(execute(&mut w, "ls -l data/a.txt")
        .stdout
        .starts_with("-rw-r--r-- 1 kali kali 18 "));
    assert_eq!(execute(&mut w, "ls -ASr data").stdout, ".hidden\na.txt\n");
    let failed = execute(&mut w, "ls missing data/a.txt");
    assert_eq!(failed.exit_code, 2);
    assert!(failed.stdout.contains("data/a.txt"));
}
#[test]
fn find_predicates_include_root_respect_depth_and_do_not_hide_permissions() {
    let mut w = world();
    w.vfs.mkdir("/home/kali/data/nested", "kali").unwrap();
    w.vfs
        .write("/home/kali/data/nested/b.TXT", "", "kali")
        .unwrap();
    assert_eq!(execute(&mut w, "find data -maxdepth 0").stdout, "data\n");
    assert_eq!(
        execute(&mut w, "find data -type f -iname '*.txt'").stdout,
        "data/a.txt\ndata/nested/b.TXT\n"
    );
    assert_eq!(
        execute(&mut w, "find data -mindepth 1 -maxdepth 1 -type d").stdout,
        "data/nested\n"
    );
    w.vfs.chmod("/home/kali/data/nested", "kali", 0).unwrap();
    let r = execute(&mut w, "find data -type f");
    assert_eq!(r.exit_code, 1);
    assert!(r.stderr.contains("Permission denied"));
    assert_ne!(execute(&mut w, "find data -delete").exit_code, 0);
    assert!(w.vfs.nodes.contains_key("/home/kali/data/a.txt"));
}
#[test]
fn chmod_symbolic_recursive_atomic_and_chown_independent_group() {
    let mut w = world();
    assert_eq!(execute(&mut w, "chmod -R u=rwX,g=rX,o= data").exit_code, 0);
    assert_eq!(w.vfs.nodes["/home/kali/data"].mode, 0o750);
    assert_eq!(w.vfs.nodes["/home/kali/data/a.txt"].mode, 0o640);
    assert_ne!(
        execute(&mut w, "chmod 777 data/a.txt /etc/os-release").exit_code,
        0
    );
    assert_eq!(w.vfs.nodes["/home/kali/data/a.txt"].mode, 0o640);
    assert_eq!(
        execute(&mut w, "sudo chown vex:root data/a.txt").exit_code,
        0
    );
    assert_eq!(execute(&mut w, "sudo chown kali data/a.txt").exit_code, 0);
    assert_eq!(w.vfs.nodes["/home/kali/data/a.txt"].group, "root");
    assert_eq!(execute(&mut w, "sudo chown :kali data/a.txt").exit_code, 0);
    assert_eq!(w.vfs.nodes["/home/kali/data/a.txt"].owner, "kali");
    assert_ne!(execute(&mut w, "chown root data/a.txt").exit_code, 0);
}
#[test]
fn unknown_options_are_honest_and_query_manuals_are_current() {
    let mut w = world();
    for command in [
        "ls --color=invalid data",
        "grep -P a data/a.txt",
        "find data -exec",
        "chmod --reference=data 777 data",
        "chown --from=root kali data",
    ] {
        assert_ne!(execute(&mut w, command).exit_code, 0, "{command}");
    }
    for name in ["ls", "grep", "find", "chmod", "chown"] {
        assert_eq!(
            execute(&mut w, &format!("{name} --help")).stdout,
            execute(&mut w, &format!("man {name}")).stdout
        );
    }
}
