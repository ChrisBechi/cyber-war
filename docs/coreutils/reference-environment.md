# GNU reference environment — Foundation

The baseline is read from `content/cli-compatibility/manifest.json`. The lock at
`content/cli-compatibility/coreutils-environment.json` must agree exactly. The
current environment is Linux x86_64, Alpine 3.22.0, musl and GNU Coreutils 9.7-r1.
The official minirootfs, every APK and the four GNU binaries have SHA-256 hashes in the lock. APK signature
verification uses the keys supplied by the official Alpine rootfs. Package version,
origin and build date are recorded; no floating Coreutils version is accepted.

## Prepare and capture

```sh
python scripts/cli/coreutils-environment.py prepare
python scripts/cli/coreutils-environment.py run --capture --command basename
python scripts/cli/coreutils-environment.py run --verify --command basename
```

Windows uses the dedicated `CyberWar-GNU97` WSL2 distribution. Linux/macOS uses a
Docker image built from the same locked files. The Docker build context includes
only those artifacts. Cached files are hash checked on every preparation. If a
pinned upstream APK becomes unavailable, preparation fails; mirror the identical
bytes instead of silently substituting newer packages.

Docker permits the nested bubblewrap namespaces with `SYS_ADMIN` and explicit
seccomp/AppArmor settings. The outer container still has no network, a read-only
root/repository and bounded writable temporary/output mounts.

Existing captures require explicit `--capture --replace`. Verification never
writes a golden. On Docker, new captures are written to
`artifacts/coreutils-reference-output`; review and copy the selected JSON files
to `tests/cli/gnu/coreutils/<manifest-version>/` explicitly. Verification reads
the checked-in files through a read-only repository mount. `--wave foundation`
selects all four commands; `--probe INPUT.json --output-dir DIR` produces
`GNU_PROBE`, which cannot satisfy a certification gate.

## Isolation and evidence

Each case uses a fresh bubblewrap namespace with networking disabled, controlled
ordered environment, explicit `LC_ALL=C`, controlled UID and passwd database,
read-only Linux binaries/libraries and a writable temporary `/home/kali` fixture.
Cases cannot write the repository or developer home. Arbitrary setup scripts are
rejected. The worker executes only the chosen Coreutils binary. Resource bounds:
5 seconds wall time, 4 CPU seconds, 512 MiB address space, 16 MiB file size,
4 MiB stdout and 64 KiB stderr. The outer Docker container is additionally limited.

Every case runs twice and must match before capture is accepted. Evidence records
exact stdout/stderr hex bytes, exit code, before/after filesystem snapshots,
canonical input digest, harness and lock hashes, actual binary SHA-256/version,
installed packages, Linux environment and capture time. Only temporary inode
numbers are represented by stable identity groups. Output bytes are never
normalized. Input fixtures, options, environment order, identities and transports
are part of the digest. A changed request cannot reuse an old GNU row.

Certification compares current project execution with the captured GNU bytes and
state, independently of declarative expectations. A mismatch fails verification.
Missing execution remains missing evidence. Unsupported locales, non-UTF-8 argv
and resources beyond the virtual process limits remain outside the documented
process model; this environment does not claim equivalence to every libc/platform.

## Project loop

```sh
pnpm cli:compat --command basename
pnpm cli:verify --command basename --strict
pnpm cli:compat --wave foundation
pnpm cli:verify --wave foundation --strict
pnpm cli:check
```

Scoped runs include the required shell/VFS evidence and Foundation commands with
existing GNU captures, so shared changes cannot silently regress a closed command.
They retain only previously captured cases with current fingerprints.
They do not fabricate evidence for skipped cases or refresh performance captures.
Run the full suite before milestone delivery. Shared shell/VFS/parser changes
invalidate their dependents. Per-command messages, fixtures and contracts do not
invalidate unrelated direct Coreutils cases. GNU evidence has separate freshness
checks for reference environment and inputs.

## Runtime boundary

All Python, WSL, Docker and reference execution is DEV/CI only. Gameplay uses the
Rust virtual implementation, package registry, virtual streams, credentials and
VFS. The Windows executable does not require Linux, Coreutils, Docker or WSL.
