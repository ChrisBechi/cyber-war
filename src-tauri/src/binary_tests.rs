use crate::{
    binary, save,
    service::GameService,
    terminal,
    vfs::{HOME, TRASH_FILES},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::Connection;

fn game() -> GameService {
    let mut game = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
    game.new_game(1, "neo", "pc", false).unwrap();
    game
}
fn put(
    game: &mut GameService,
    path: &str,
    data: &[u8],
    expected: Option<u64>,
) -> crate::error::GameResult<()> {
    game.mutate(
        |_, w, _| {
            binary::import(
                w,
                path,
                &STANDARD.encode(data),
                "image/png",
                expected,
                "kali",
            )
        },
        false,
    )
}

#[test]
fn bytes_roundtrip_are_not_serialized_and_copies_share_one_blob() {
    let mut game = game();
    let bytes = [0, 255, 128, 10, 13];
    put(&mut game, "Pictures/a.png", &bytes, None).unwrap();
    game.mutate(
        |_, w, _| {
            assert_eq!(
                terminal::execute(w, "cp Pictures/a.png Pictures/b.png").exit_code,
                0
            );
            assert_ne!(terminal::execute(w, "cat Pictures/a.png").exit_code, 0);
            assert_ne!(
                terminal::execute(w, "echo text >> Pictures/a.png").exit_code,
                0
            );
            Ok(())
        },
        true,
    )
    .unwrap();
    let (json, _) = save::encode(game.world().unwrap()).unwrap();
    assert!(!json.contains(&STANDARD.encode(bytes)));
    assert!(!json.contains("\"blobs\""));
    assert_eq!(
        game.connection
            .query_row("SELECT count(*) FROM vfs_blobs", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    game.end_session().unwrap();
    game.load(1, false).unwrap();
    assert_eq!(
        binary::read(game.world().unwrap(), "Pictures/b.png", "kali")
            .unwrap()
            .base64,
        STANDARD.encode(bytes)
    );
}

#[test]
fn versions_permissions_overwrite_and_trash_preserve_bytes() {
    let mut game = game();
    put(&mut game, "Pictures/a.png", b"old", None).unwrap();
    let version = game.world().unwrap().vfs.nodes[&format!("{HOME}/Pictures/a.png")].modified_at;
    assert!(put(&mut game, "Pictures/a.png", b"new", None).is_err());
    assert!(put(&mut game, "Pictures/a.png", b"new", Some(version + 1)).is_err());
    put(&mut game, "Pictures/a.png", b"new", Some(version)).unwrap();
    game.mutate(
        |_, w, _| {
            w.vfs
                .chmod(&format!("{HOME}/Pictures/a.png"), "kali", 0o200)?;
            assert!(binary::read(w, "Pictures/a.png", "kali").is_err());
            w.vfs
                .chmod(&format!("{HOME}/Pictures/a.png"), "kali", 0o600)?;
            w.vfs
                .move_to_trash(&format!("{HOME}/Pictures/a.png"), "kali", false)?;
            assert!(binary::read(w, &format!("{TRASH_FILES}/a.png"), "kali").is_err());
            w.vfs
                .restore_from_trash(&format!("{TRASH_FILES}/a.png"), "kali")?;
            Ok(())
        },
        false,
    )
    .unwrap();
    assert_eq!(
        binary::read(game.world().unwrap(), "Pictures/a.png", "kali")
            .unwrap()
            .base64,
        STANDARD.encode(b"new")
    );
}

#[test]
fn sql_failure_rolls_back_blob_and_file_and_corruption_preserves_live_world() {
    let mut game = game();
    game.connection.execute_batch("CREATE TRIGGER reject_binary_save BEFORE UPDATE ON save_slots BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(put(&mut game, "Pictures/a.png", b"payload", None).is_err());
    assert!(!game
        .world()
        .unwrap()
        .vfs
        .nodes
        .contains_key(&format!("{HOME}/Pictures/a.png")));
    assert_eq!(
        game.connection
            .query_row("SELECT count(*) FROM vfs_blobs", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    game.connection
        .execute_batch("DROP TRIGGER reject_binary_save;")
        .unwrap();
    put(&mut game, "Pictures/a.png", b"payload", None).unwrap();
    let before = save::encode(game.world().unwrap()).unwrap().0;
    game.connection
        .execute("UPDATE vfs_blobs SET data=?1", [b"corrupt".as_slice()])
        .unwrap();
    assert!(game.load(1, false).is_err());
    assert_eq!(save::encode(game.world().unwrap()).unwrap().0, before);
}

#[test]
fn manual_checkpoint_and_old_text_snapshots_remain_loadable() {
    let mut game = game();
    game.save(true).unwrap();
    put(&mut game, "Pictures/a.png", b"first", None).unwrap();
    game.save(false).unwrap();
    let checkpoint = save::checkpoints(&game.connection, 1).unwrap()[0]
        .id
        .clone();
    let version = game.world().unwrap().vfs.nodes[&format!("{HOME}/Pictures/a.png")].modified_at;
    put(&mut game, "Pictures/a.png", b"second", Some(version)).unwrap();
    game.load(1, true).unwrap();
    assert!(binary::read(game.world().unwrap(), "Pictures/a.png", "kali").is_err());
    game.restore(&checkpoint).unwrap();
    assert_eq!(
        binary::read(game.world().unwrap(), "Pictures/a.png", "kali")
            .unwrap()
            .base64,
        STANDARD.encode(b"first")
    );
    game.load(1, true).unwrap();
    assert_eq!(
        binary::read(game.world().unwrap(), "Pictures/a.png", "kali")
            .unwrap()
            .base64,
        STANDARD.encode(b"first")
    );
    assert!(game
        .world()
        .unwrap()
        .vfs
        .read("/etc/os-release", "kali")
        .is_ok());
    assert_eq!(
        game.connection
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i32>(0))
            .unwrap(),
        3
    );
}

#[test]
fn invalid_imports_never_create_files() {
    let mut game = game();
    for (path, data, mime) in [
        ("C:\\Windows\\a", "YQ==", "image/png"),
        ("Pictures/a", "!", "image/png"),
        ("Pictures/a", "YQ==", "text/html;script"),
        ("/root/a", "YQ==", "image/png"),
    ] {
        assert!(game
            .mutate(
                |_, w, _| binary::import(w, path, data, mime, None, "kali"),
                false
            )
            .is_err());
    }
    let huge = STANDARD.encode(vec![0; binary::MAX_BLOB + 1]);
    assert!(game
        .mutate(
            |_, w, _| binary::import(w, "Pictures/a", &huge, "image/png", None, "kali"),
            false
        )
        .is_err());
}
