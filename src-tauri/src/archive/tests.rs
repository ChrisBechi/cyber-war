use super::*;
use crate::{terminal, vfs::HOME, world::WorldState};
use std::collections::BTreeMap;
fn world() -> WorldState {
    let mut w = WorldState::new("kali", "kali").unwrap();
    w.vfs.mkdir("/home/kali/test", "kali").unwrap();
    w.vfs.mkdir("/home/kali/test/nested", "kali").unwrap();
    for (p, c) in [
        ("a.txt", "alpha\nadmin\n"),
        ("b.txt", "beta\n"),
        (".hidden", "secret\n"),
        ("nested/c.txt", "nested\n"),
        ("executable.sh", "echo hello\n"),
    ] {
        w.vfs.write(&format!("{HOME}/test/{p}"), c, "kali").unwrap();
    }
    w.vfs
        .nodes
        .get_mut("/home/kali/test/executable.sh")
        .unwrap()
        .mode = 0o755;
    w.vfs
        .symlink("/home/kali/test/latest", "nested/c.txt", "kali")
        .unwrap();
    w
}
fn run(w: &mut WorldState, command: &str) -> String {
    let r = terminal::execute(w, command);
    assert_eq!(r.exit_code, 0, "{command}: {}", r.stderr);
    r.stdout
}
fn extraction(destination: &str) -> ExtractOptions {
    ExtractOptions {
        destination: destination.into(),
        overwrite: Overwrite::Replace,
        selected: Vec::new(),
        decisions: BTreeMap::new(),
    }
}
#[test]
fn formats_roundtrip_hidden_permissions_symlinks_and_nested() {
    for (format, extension) in [
        (ArchiveFormat::Zip, "zip"),
        (ArchiveFormat::Tar, "tar"),
        (ArchiveFormat::TarGzip, "tar.gz"),
        (ArchiveFormat::TarBzip2, "tar.bz2"),
        (ArchiveFormat::TarXz, "tar.xz"),
    ] {
        let mut w = world();
        let service = ArchiveService::default();
        let before = w.vfs.nodes.clone();
        let path = format!("{HOME}/backup.{extension}");
        service
            .create_archive(&mut w, &path, &["test".into()], format, true, None, "kali")
            .unwrap();
        assert_eq!(service.list_archive(&w, &path, "kali").unwrap().len(), 8);
        service.test_archive(&w, &path, None, "kali").unwrap();
        w.vfs.remove("/home/kali/test", "kali", true).unwrap();
        service
            .extract_archive(&mut w, &path, &extraction(HOME), None, "kali")
            .unwrap();
        for (p, n) in before
            .iter()
            .filter(|(p, _)| p.starts_with("/home/kali/test"))
        {
            let actual = &w.vfs.nodes[p];
            assert_eq!(actual.content, n.content, "{extension}: {p}");
            assert_eq!(actual.mode, n.mode);
            assert_eq!(actual.kind, n.kind);
            assert_eq!(actual.modified_at, n.modified_at);
        }
        service
            .create_archive(
                &mut w,
                "outer.zip",
                &[format!("backup.{extension}")],
                ArchiveFormat::Zip,
                false,
                None,
                "kali",
            )
            .unwrap();
        service
            .extract_archive(
                &mut w,
                "outer.zip",
                &extraction("/tmp/nested"),
                None,
                "kali",
            )
            .unwrap();
        service
            .test_archive(&w, &format!("/tmp/nested/backup.{extension}"), None, "kali")
            .unwrap();
    }
}
#[test]
fn cli_parsers_streams_pipes_redirection_and_magic() {
    let mut w = world();
    for cmd in [
        "tar -cvf a.tar test",
        "tar cvf b.tar test",
        "tar -czvf a.tar.gz test",
        "tar -cjvf a.tar.bz2 test",
        "tar -cJvf a.tar.xz test",
        "zip -r a.zip test",
    ] {
        run(&mut w, cmd);
    }
    for cmd in [
        "tar -tvf a.tar",
        "tar tvf b.tar",
        "tar -tzvf a.tar.gz",
        "tar -tjvf a.tar.bz2",
        "tar -tJvf a.tar.xz",
        "unzip -l a.zip",
        "unzip -t a.zip",
    ] {
        assert!(!run(&mut w, cmd).is_empty());
    }
    run(&mut w, "mv a.zip photo.jpg");
    assert!(run(&mut w, "file photo.jpg").contains("Zip archive data"));
    run(&mut w, "unzip photo.jpg -d /tmp/extracted");
    run(&mut w, "tar -xf a.tar -C /tmp");
    for (compress, decompress, ext) in [
        ("gzip", "gunzip", "gz"),
        ("bzip2", "bunzip2", "bz2"),
        ("xz", "unxz", "xz"),
    ] {
        run(&mut w, &format!("{compress} -k test/a.txt"));
        assert!(w.vfs.nodes.contains_key("/home/kali/test/a.txt"));
        run(&mut w, &format!("{decompress} -f test/a.txt.{ext}"));
        run(&mut w, &format!("{compress} test/a.txt"));
        assert!(!w.vfs.nodes.contains_key("/home/kali/test/a.txt"));
        run(&mut w, &format!("{compress} -d test/a.txt.{ext}"));
        assert_eq!(
            w.vfs.read("/home/kali/test/a.txt", "kali").unwrap(),
            "alpha\nadmin\n"
        );
        assert_ne!(
            terminal::execute(&mut w, &format!("{compress} test")).exit_code,
            0
        );
    }
    run(&mut w, "gzip -c test/a.txt > logs.gz");
    let before = w.vfs.nodes.clone();
    assert_eq!(run(&mut w, "zcat logs.gz | grep admin"), "admin\n");
    assert_eq!(w.vfs.nodes, before);
    run(&mut w, "zcat logs.gz > logs.txt");
    assert_eq!(
        w.vfs.read("/home/kali/logs.txt", "kali").unwrap(),
        "alpha\nadmin\n"
    );
    assert!(run(&mut w, "tar -tf a.tar | grep hidden").contains(".hidden"));
}
#[test]
fn encrypted_passwords_prompts_overwrite_and_no_secret_in_snapshot() {
    let mut w = world();
    let service = ArchiveService::default();
    service
        .create_archive(
            &mut w,
            "private.zip",
            &["test/a.txt".into()],
            ArchiveFormat::Zip,
            false,
            Some("correct-horse"),
            "kali",
        )
        .unwrap();
    assert!(!serde_json::to_string(&w).unwrap().contains("correct-horse"));
    let before = w.vfs.clone();
    assert!(service
        .extract_archive(
            &mut w,
            "private.zip",
            &extraction("/tmp/private"),
            Some("wrong"),
            "kali"
        )
        .unwrap_err()
        .to_string()
        .contains("password"));
    assert_eq!(w.vfs, before);
    let r = terminal::execute(&mut w, "unzip private.zip -d /tmp/private");
    assert!(r.archive_prompt.unwrap().secret);
    let r = ipc::input(&mut w, "correct-horse", false);
    assert_eq!(r.exit_code, 0, "{}", r.stderr);
    assert_eq!(
        w.vfs.read("/tmp/private/test/a.txt", "kali").unwrap(),
        "alpha\nadmin\n"
    );
    run(&mut w, "zip plain.zip test/a.txt");
    let r = terminal::execute(&mut w, "unzip plain.zip");
    assert!(!r.archive_prompt.unwrap().secret);
    assert_eq!(ipc::input(&mut w, "n", false).exit_code, 0);
    terminal::execute(&mut w, "unzip plain.zip");
    assert_eq!(ipc::input(&mut w, "", true).exit_code, 130);
    assert!(w.terminal.archive_pending.is_none());
    assert!(!w
        .terminal
        .history
        .iter()
        .any(|h| h.contains("correct-horse")));
}
#[test]
fn disk_permissions_sparse_and_failed_operations_are_atomic() {
    let mut w = world();
    let service = ArchiveService::default();
    w.vfs
        .nodes
        .get_mut("/home/kali/test/a.txt")
        .unwrap()
        .metadata
        .insert(
            "logicalSize".into(),
            (4u64 * 1024 * 1024 * 1024).to_string(),
        );
    service
        .create_archive(
            &mut w,
            "large.tar.gz",
            &["test".into()],
            ArchiveFormat::TarGzip,
            true,
            None,
            "kali",
        )
        .unwrap();
    let info = service.inspect_archive(&w, "large.tar.gz", "kali").unwrap();
    assert!(info.storage_size < 10_000);
    assert!(info.original_size > 4 * 1024 * 1024 * 1024);
    let before = w.vfs.clone();
    assert!(service
        .extract_archive(&mut w, "large.tar.gz", &extraction("/root"), None, "kali")
        .is_err());
    assert_eq!(w.vfs, before);
    w.vfs.capacity_bytes = w.vfs.used_bytes() + 2 * 1024 * 1024 * 1024;
    let before = w.vfs.clone();
    assert!(service
        .extract_archive(
            &mut w,
            "large.tar.gz",
            &extraction("/tmp/large"),
            None,
            "kali"
        )
        .unwrap_err()
        .to_string()
        .contains("space"));
    assert_eq!(w.vfs, before);
    w.vfs.capacity_bytes = 64 * 1024 * 1024 * 1024;
    service
        .extract_archive(&mut w, "large.tar.gz", &extraction("/root"), None, "root")
        .unwrap();
    assert_eq!(
        w.vfs.nodes["/root/test/a.txt"].logical_size(),
        4 * 1024 * 1024 * 1024
    );
}
fn entry(path: &str) -> StoredEntry {
    StoredEntry {
        info: ArchiveEntry {
            path: path.into(),
            kind: "file".into(),
            original_size: 1,
            compressed_size: None,
            permissions: 0o644,
            modified_at: 1,
            owner: None,
            group: None,
            crc: None,
            link_target: None,
            encrypted: false,
        },
        data: vec![b'x'],
        metadata: None,
    }
}
#[test]
fn malicious_paths_duplicates_links_and_limits() {
    let limits = ArchiveSafetyLimits::default();
    for p in [
        "../escape.txt",
        "../../escape.txt",
        "/absolute/path",
        "a/../../escape",
        "a/./b",
        "C:/Windows/file",
        "a\\..\\x",
        "a//x",
    ] {
        assert!(limits.entries(&[entry(p)], 100).is_err(), "{p}");
    }
    assert!(limits.entries(&[entry(&"a".repeat(256))], 100).is_err());
    assert!(limits
        .entries(&[entry(&vec!["a"; 33].join("/"))], 100)
        .is_err());
    assert!(limits.entries(&vec![entry("a"); 4097], 100).is_err());
    assert!(limits.entries(&[entry("a"), entry("a")], 100).is_err());
    let mut symlink = entry("link");
    symlink.info.kind = "symlink".into();
    symlink.info.link_target = Some("../../escape".into());
    assert!(limits.entries(&[symlink.clone()], 100).is_err());
    symlink.info.link_target = Some("valid".into());
    assert!(limits.entries(&[symlink, entry("link/file")], 100).is_err());
    let mut huge = entry("huge");
    huge.info.original_size = u64::MAX;
    assert!(limits.entries(&[huge], 100).is_err());
}
#[test]
fn invalid_truncated_bombs_and_zip_slip_never_write() {
    use std::io::{Cursor, Write};
    let mut w = world();
    let service = ArchiveService::default();
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("../escape.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"escape").unwrap();
    let data = zip.finish().unwrap().into_inner();
    write_bytes(&mut w, "/home/kali/evil.zip", data, "kali", 0).unwrap();
    let before = w.vfs.clone();
    assert!(service
        .extract_archive(&mut w, "evil.zip", &extraction("/tmp/out"), None, "kali")
        .is_err());
    assert_eq!(w.vfs, before);
    service
        .create_archive(
            &mut w,
            "test.tar",
            &["test".into()],
            ArchiveFormat::Tar,
            true,
            None,
            "kali",
        )
        .unwrap();
    let mut data = bytes(&w, "/home/kali/test.tar", "kali")
        .unwrap()
        .as_ref()
        .clone();
    data.truncate(data.len() - 700);
    write_bytes(&mut w, "/home/kali/truncated.tar", data, "kali", 0).unwrap();
    assert!(service
        .test_archive(&w, "truncated.tar", None, "kali")
        .is_err());
    w.vfs
        .write("/home/kali/invalid.zip", "garbage", "kali")
        .unwrap();
    assert!(service
        .test_archive(&w, "invalid.zip", None, "kali")
        .is_err());
    let data = codec::compress(&vec![0; 2 * 1024 * 1024], ArchiveFormat::Gzip).unwrap();
    let limits = ArchiveSafetyLimits {
        max_compression_ratio: 10,
        ..ArchiveSafetyLimits::default()
    };
    write_bytes(&mut w, "/home/kali/bomb.gz", data, "kali", 0).unwrap();
    assert!(ArchiveService { limits }
        .decompress_file(
            &mut w,
            "bomb.gz",
            ArchiveFormat::Gzip,
            true,
            false,
            false,
            "kali"
        )
        .is_err());
}

#[test]
fn downloaded_packages_mail_and_gui_use_the_same_persistent_vfs() {
    let mut game =
        crate::service::GameService::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
    game.new_game(1, "neo", "pc", false).unwrap();
    game.mutate(
        |_, w, _| {
            w.network.connected = true;
            downloads::download(
                w,
                downloads::FIRMWARE_URL,
                "/home/kali/Downloads/firmware.tar.gz",
                "kali",
            )?;
            let r = ipc::operate(
                w,
                ipc::Request {
                    operation: "extract".into(),
                    path: "/home/kali/Downloads/firmware.tar.gz".into(),
                    format: None,
                    inputs: Vec::new(),
                    options: Some(extraction("/home/kali/Downloads")),
                    password: None,
                    as_root: false,
                },
            );
            assert!(r.error.is_none(), "{:?}", r.error);
            assert!(run(w, "cat Downloads/firmware/README").contains("AXR550"));
            assert!(crate::mission::Condition::FileContains {
                path: "/home/kali/Downloads/firmware/README".into(),
                text: "AXR550".into()
            }
            .matches(w));
            w.notify("Nexora", "Documentos anexados.");
            let message = w.messages.last_mut().unwrap();
            message.attachments.push(downloads::ATTACHMENT_URL.into());
            let id = message.id.clone();
            downloads::attachment(w, &id, 0)?;
            Ok(())
        },
        false,
    )
    .unwrap();
    game.end_session().unwrap();
    game.load(1, false).unwrap();
    let w = &mut game.active.as_mut().unwrap().world;
    run(w, "unzip Downloads/documentos.zip -d /tmp/mail");
    assert!(run(w, "cat /tmp/mail/documentos/notes.txt").contains("Nexora"));
    run(w, "tar -tzf Downloads/firmware.tar.gz");
}

#[test]
fn background_jobs_cancel_without_partial_output() {
    let mut w = world();
    let before = w.vfs.clone();
    let r = terminal::execute(&mut w, "tar -czf cancelled.tar.gz test &");
    assert_eq!(r.exit_code, 0, "{}", r.stderr);
    let id = w.archive_jobs[0].state.id;
    assert!(run(&mut w, "jobs").contains("Running"));
    jobs::cancel(id);
    jobs::tick(&mut w);
    assert_eq!(w.vfs, before);
    assert!(run(&mut w, "jobs").contains("Cancelled"));
    terminal::execute(&mut w, "tar -czf complete.tar.gz test &");
    std::thread::sleep(std::time::Duration::from_millis(300));
    jobs::tick(&mut w);
    assert!(w.vfs.nodes.contains_key("/home/kali/complete.tar.gz"));
    assert!(!serde_json::to_string(&w).unwrap().contains("archiveJobs"));
}

#[test]
fn gui_paths_encrypted_links_media_and_session_cleanup() {
    let mut w = world();
    let original_session = w.terminal.clone();
    let r = ipc::operate(
        &mut w,
        ipc::Request {
            operation: "create".into(),
            path: format!("{HOME}/Downloads/gui.zip"),
            inputs: vec![
                format!("{HOME}/test"),
                format!("{HOME}/Pictures/kali-waves.png"),
            ],
            format: Some(ArchiveFormat::Zip),
            options: None,
            password: Some("hidden".into()),
            as_root: false,
        },
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    let info = r.inspection.unwrap();
    assert!(info.entries.iter().any(|e| e.path == "test/latest"));
    assert!(info.entries.iter().all(|e| !e.path.starts_with("home/")));
    assert_eq!(w.terminal.cwd, original_session.cwd);
    ArchiveService::default()
        .extract_archive(
            &mut w,
            &info.path,
            &extraction("/tmp/gui"),
            Some("hidden"),
            "kali",
        )
        .unwrap();
    assert_eq!(
        run(&mut w, "readlink /tmp/gui/test/latest"),
        "nested/c.txt\n"
    );
    assert_eq!(
        w.vfs.nodes["/tmp/gui/Pictures/kali-waves.png"].metadata["mediaSource"],
        "/assets/kali-waves.png"
    );
    assert!(w.vfs.nodes["/tmp/gui/Pictures/kali-waves.png"]
        .blob
        .is_none());
    run(&mut w, "gzip -k test/a.txt");
    let r = ipc::operate(
        &mut w,
        ipc::Request {
            operation: "decompress".into(),
            path: format!("{HOME}/test/a.txt.gz"),
            inputs: Vec::new(),
            format: None,
            options: Some(extraction("/tmp/stream")),
            password: None,
            as_root: false,
        },
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(
        w.vfs.read("/tmp/stream/a.txt", "kali").unwrap(),
        "alpha\nadmin\n"
    );
    run(&mut w, "tar -czf running.tar.gz test &");
    let mut restored: WorldState =
        serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    crate::terminal_sessions::reset(&mut restored);
    assert!(!restored
        .processes
        .iter()
        .any(|p| p.name.starts_with("tar ")));
    crate::terminal_sessions::reset(&mut w);
    assert!(w.archive_jobs.is_empty());
}

#[test]
fn real_external_tar_dot_prefix_and_selected_extraction() {
    use std::io::Cursor;
    let mut builder = tar::Builder::new(Vec::new());
    for name in ["./", "./dir/", "./dir/file.txt"] {
        let mut h = tar::Header::new_gnu();
        h.set_mode(0o755);
        h.set_mtime(17);
        h.set_entry_type(if name.ends_with('/') {
            tar::EntryType::Directory
        } else {
            tar::EntryType::Regular
        });
        let body = if name.ends_with('/') {
            b"".as_slice()
        } else {
            b"external\n".as_slice()
        };
        h.set_size(body.len() as u64);
        h.set_cksum();
        builder
            .append_data(&mut h, name, Cursor::new(body))
            .unwrap();
    }
    let data = builder.into_inner().unwrap();
    let mut w = world();
    write_bytes(&mut w, "/home/kali/external.tar", data, "kali", 0).unwrap();
    let mut options = extraction("/tmp/external");
    options.selected = vec!["dir/file.txt".into()];
    ArchiveService::default()
        .extract_archive(&mut w, "external.tar", &options, None, "kali")
        .unwrap();
    assert_eq!(
        w.vfs.read("/tmp/external/dir/file.txt", "kali").unwrap(),
        "external\n"
    );
    assert_eq!(w.vfs.nodes["/tmp/external/dir/file.txt"].mode, 0o755);
}

#[test]
fn hostile_tar_headers_and_zip_directory_counts_are_rejected() {
    use std::io::Cursor;
    for name in [
        "../escape.txt",
        "../../escape.txt",
        "/absolute/path",
        "nested/../../escape",
        "duplicate",
    ] {
        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(1);
        header.set_entry_type(tar::EntryType::Regular);
        header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
        header.set_cksum();
        builder.append(&header, Cursor::new(b"x")).unwrap();
        if name == "duplicate" {
            builder.append(&header, Cursor::new(b"x")).unwrap();
        }
        let mut w = world();
        write_bytes(
            &mut w,
            "/home/kali/hostile.tar",
            builder.into_inner().unwrap(),
            "kali",
            0,
        )
        .unwrap();
        let before = w.vfs.clone();
        assert!(ArchiveService::default()
            .extract_archive(
                &mut w,
                "hostile.tar",
                &extraction("/tmp/hostile"),
                None,
                "kali"
            )
            .is_err());
        assert_eq!(w.vfs, before);
    }
    let mut w = world();
    run(&mut w, "zip count.zip test/a.txt");
    let mut data = bytes(&w, "/home/kali/count.zip", "kali")
        .unwrap()
        .as_ref()
        .clone();
    let end = data.len() - 22;
    data[end + 8..end + 12].copy_from_slice(&[254, 255, 254, 255]);
    write_bytes(&mut w, "/home/kali/count.zip", data, "kali", 0).unwrap();
    assert!(ArchiveService::default()
        .inspect_archive(&w, "count.zip", "kali")
        .unwrap_err()
        .to_string()
        .contains("count"));
    assert!(crate::mission::Condition::ArchiveEvent {
        event: "ARCHIVE_CREATED".into(),
        path: "/home/kali/count.zip".into()
    }
    .matches(&w));
}

#[test]
fn large_password_command_defers_after_input_and_cancels_atomically() {
    let mut w = world();
    w.vfs
        .nodes
        .get_mut("/home/kali/test/a.txt")
        .unwrap()
        .metadata
        .insert("logicalSize".into(), (1024u64 * 1024 * 1024).to_string());
    let prompt = terminal::execute(&mut w, "zip -e delayed.zip test/a.txt");
    assert!(prompt.archive_prompt.unwrap().secret);
    let before = w.vfs.clone();
    let result = ipc::input(&mut w, "hidden-secret", false);
    let id = result
        .archive_job
        .expect("large password operation becomes a job");
    assert!(!serde_json::to_string(&w).unwrap().contains("hidden-secret"));
    jobs::cancel(id);
    jobs::tick(&mut w);
    assert_eq!(w.vfs, before);
    assert_eq!(w.archive_jobs[0].state.status, "Cancelled");
}
