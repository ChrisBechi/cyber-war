# VFS compatibility â€” Milestone 1B

Generated from current captures. READY refers to the documented virtual subset; it does not certify GNU Coreutils.

| Capability              | State   | Required cases                                                                                                                                                                                                             | Limits                                                                                                                                                                                                                                                                                                                                      |
| ----------------------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| VFS.PATHS               | READY   | vfs/path/components, vfs/path/trailing-slash, vfs/path/root, vfs/path/unicode-case                                                                                                                                         |                                                                                                                                                                                                                                                                                                                                             |
| VFS.REGULAR_FILES       | READY   | vfs/hardlink/shared-content, vfs/inode/copy-new-id                                                                                                                                                                         |                                                                                                                                                                                                                                                                                                                                             |
| VFS.DIRECTORIES         | READY   | vfs/directories/rename-cwd, vfs/directories/list-link, vfs/directories/rmdir                                                                                                                                               |                                                                                                                                                                                                                                                                                                                                             |
| VFS.INODES              | READY   | vfs/inode/rename-preserves-id, vfs/inode/copy-new-id                                                                                                                                                                       |                                                                                                                                                                                                                                                                                                                                             |
| VFS.SYMLINKS            | READY   | vfs/symlink/relative-target, vfs/symlink/absolute, vfs/symlink/dangling, vfs/symlink/create-target, vfs/symlink/loop, vfs/symlink/unlink-only-link, vfs/symlink/glob-directory, vfs/symlink/executable, vfs/symlink/script |                                                                                                                                                                                                                                                                                                                                             |
| VFS.HARDLINKS           | READY   | vfs/hardlink/shared-content, vfs/hardlink/unlink-one, vfs/save/hardlink-roundtrip                                                                                                                                          |                                                                                                                                                                                                                                                                                                                                             |
| VFS.PERMISSIONS         | READY   | vfs/permissions/directory-search, vfs/permissions/search-without-read, vfs/permissions/readonly-file                                                                                                                       |                                                                                                                                                                                                                                                                                                                                             |
| VFS.SPECIAL_PERMISSIONS | READY   | vfs/permissions/umask, vfs/sticky/deny-unlink, vfs/sticky/setgid-directory, vfs/metadata/special-bits                                                                                                                      | Deterministic ticks; noatime for immutable text/blob queries, explicit atime for handle reads and touch.; Special mode bits are metadata; setgid directory inheritance and sticky deletion enforced. No setuid/setgid execution or capabilities.; Only null and bounded zero devices; no host mounts, ACLs, random or tty device emulation. |
| VFS.METADATA            | READY   | vfs/metadata/shared-mode, vfs/symlink/relative-target, vfs/metadata/stat-lstat                                                                                                                                             |                                                                                                                                                                                                                                                                                                                                             |
| VFS.TIMESTAMPS          | READY   | vfs/timestamps/touch-preserves-data                                                                                                                                                                                        | Deterministic ticks; noatime for immutable text/blob queries, explicit atime for handle reads and touch.; Special mode bits are metadata; setgid directory inheritance and sticky deletion enforced. No setuid/setgid execution or capabilities.; Only null and bounded zero devices; no host mounts, ACLs, random or tty device emulation. |
| VFS.DEVICES             | PARTIAL | vfs/device/null, vfs/device/hardlink-null                                                                                                                                                                                  | Deterministic ticks; noatime for immutable text/blob queries, explicit atime for handle reads and touch.; Special mode bits are metadata; setgid directory inheritance and sticky deletion enforced. No setuid/setgid execution or capabilities.; Only null and bounded zero devices; no host mounts, ACLs, random or tty device emulation. |
| VFS.SAVE_ROUNDTRIP      | READY   | vfs/save/hardlink-roundtrip, vfs/save/rename-roundtrip                                                                                                                                                                     |                                                                                                                                                                                                                                                                                                                                             |

## Coreutils blockers

SUBSYSTEM:SHELL.JOBS:PARTIAL

## Evidence

| Case                                | Result | Reason                                             |
| ----------------------------------- | ------ | -------------------------------------------------- |
| vfs/path/components                 | PASS   | All declared stream/status/state assertions passed |
| vfs/path/trailing-slash             | PASS   | All declared stream/status/state assertions passed |
| vfs/path/root                       | PASS   | All declared stream/status/state assertions passed |
| vfs/path/unicode-case               | PASS   | All declared stream/status/state assertions passed |
| vfs/inode/rename-preserves-id       | PASS   | All declared stream/status/state assertions passed |
| vfs/inode/copy-new-id               | PASS   | All declared stream/status/state assertions passed |
| vfs/hardlink/shared-content         | PASS   | All declared stream/status/state assertions passed |
| vfs/hardlink/unlink-one             | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/relative-target         | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/absolute                | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/dangling                | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/create-target           | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/loop                    | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/unlink-only-link        | PASS   | All declared stream/status/state assertions passed |
| vfs/permissions/directory-search    | PASS   | All declared stream/status/state assertions passed |
| vfs/permissions/search-without-read | PASS   | All declared stream/status/state assertions passed |
| vfs/permissions/readonly-file       | PASS   | All declared stream/status/state assertions passed |
| vfs/permissions/umask               | PASS   | All declared stream/status/state assertions passed |
| vfs/sticky/deny-unlink              | PASS   | All declared stream/status/state assertions passed |
| vfs/sticky/setgid-directory         | PASS   | All declared stream/status/state assertions passed |
| vfs/metadata/shared-mode            | PASS   | All declared stream/status/state assertions passed |
| vfs/metadata/special-bits           | PASS   | All declared stream/status/state assertions passed |
| vfs/timestamps/touch-preserves-data | PASS   | All declared stream/status/state assertions passed |
| vfs/device/null                     | PASS   | All declared stream/status/state assertions passed |
| vfs/device/hardlink-null            | PASS   | All declared stream/status/state assertions passed |
| vfs/save/hardlink-roundtrip         | PASS   | All declared stream/status/state assertions passed |
| vfs/save/rename-roundtrip           | PASS   | All declared stream/status/state assertions passed |
| vfs/isolation/windows-looking-name  | PASS   | All declared stream/status/state assertions passed |
| vfs/isolation/host-executable       | PASS   | All declared stream/status/state assertions passed |
| vfs/directories/rename-cwd          | PASS   | All declared stream/status/state assertions passed |
| vfs/directories/list-link           | PASS   | All declared stream/status/state assertions passed |
| vfs/metadata/stat-lstat             | PASS   | All declared stream/status/state assertions passed |
| vfs/directories/rmdir               | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/glob-directory          | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/executable              | PASS   | All declared stream/status/state assertions passed |
| vfs/symlink/script                  | PASS   | All declared stream/status/state assertions passed |

## Performance

```json
{
  "depth120": {
    "iterations": 1000,
    "meanMs": 0.4947314
  },
  "profile": "cargo test / debug; bounded virtual VFS; wall clock; no host program execution",
  "samples": [
    {
      "actual": 1000,
      "createMs": 170.0525,
      "entries": 1045,
      "enumeration": {
        "iterations": 25,
        "meanMs": 0.512096
      },
      "load": {
        "iterations": 3,
        "meanMs": 127.56396666666666
      },
      "requested": 1000,
      "save": {
        "iterations": 3,
        "meanMs": 37.58793333333333
      },
      "snapshotBytes": 247818,
      "stat": {
        "iterations": 10000,
        "meanMs": 0.03146366
      }
    },
    {
      "actual": 9955,
      "createMs": 1518.1513,
      "entries": 10000,
      "enumeration": {
        "iterations": 25,
        "meanMs": 2.383244
      },
      "load": {
        "iterations": 3,
        "meanMs": 1013.3739333333333
      },
      "requested": 10000,
      "save": {
        "iterations": 3,
        "meanMs": 259.50926666666663
      },
      "snapshotBytes": 2434847,
      "stat": {
        "iterations": 10000,
        "meanMs": 0.021903549999999997
      }
    }
  ],
  "schemaVersion": 1,
  "symlinks40": {
    "iterations": 1000,
    "meanMs": 0.2680576
  }
}
```

## Save compatibility

VFS wire 2 preserves stable inode IDs and deduplicates hardlinks. Legacy nodes migrate deterministically without inventing hardlinks. See [migration](../vfs/save-migration.md).

## Host isolation

PASS: Source/build boundary guard, not an OS sandbox proof. Virtual command tests exercise the actual dispatch; there is no host-executor implementation in production.

## Known gaps

- Deterministic ticks; noatime for immutable text/blob queries, explicit atime for handle reads and touch.
- Special mode bits are metadata; setgid directory inheritance and sticky deletion enforced. No setuid/setgid execution or capabilities.
- Only null and bounded zero devices; no host mounts, ACLs, random or tty device emulation.

See [architecture](../vfs/architecture.md) and [subset](../vfs/posix-linux-subset.md).
