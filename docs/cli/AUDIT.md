# CYBER WAR — CLI fidelity audit

## Status: implementation in progress

This report is a checkpoint, not a declaration that the entire CLI is compatible.
The inventory covers the dispatch registry, shell-only names, catalog names and
aliases, repository executable bindings and legacy package migration bindings.
Custom `.deb` payloads can add arbitrary executable names; their allowed handler
families are `builtin`, `catalog:*`, `netscan`, `wireless` and `iot`.

## CLI Audit Summary

- Initial discovery: **1,891 command names/aliases**, 303 catalog entries, 276 software groups.
- Current discovery: **1,793 command names/aliases**. Removed 98 erroneous `Includes …` aliases produced by the catalog importer.
- Counts are names/aliases, not independent native Linux implementations.
- Current states: **0 VERIFIED, 44 PARTIAL, 1,749 UNVERIFIED**.
- Catalog inspection adapters remain REAL_COMPAT / UNVERIFIED: a real tool name
  accepting `--lab` does not constitute that tool's native CLI compatibility.
- `lab`, `edit`, `sector-ix` and repository tools `netscan`, `wireless-utils`,
  `iot-discovery` have native game contracts. `ipconfig` is an intentional
  command-not-found diagnostic, not an emulation of the Windows utility.

Full machine-readable matrices: [initial inventory](inventory.initial.json) and
[current inventory](inventory.json). [Readable inventory](inventory.md).

## Reference baseline

Keep package versions already defined by the project; do not inherit Windows
versions. This is a virtual LifeOS profile inspired by Kali default/Xfce, not a
claim that all packages match a single official Kali ISO. Unpinned entries are
explicitly null in the inventory and cannot be marked VERIFIED.

References actually consulted in this pass:

| Program                    | Version                         | Primary reference                                                                                                                                                                                                                                                                                                                                                                                            |
| -------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Bash                       | 5.2.37, Debian 5.2.37-2+b9      | [Debian Bash manual](https://manpages.debian.org/trixie/bash/bash.1.en.html)                                                                                                                                                                                                                                                                                                                                 |
| cat, head, tail, tee, ls   | GNU coreutils 9.7, Debian 9.7-3 | [cat](https://manpages.debian.org/trixie/coreutils/cat.1.en.html), [head](https://manpages.debian.org/trixie/coreutils/head.1.en.html), [tail](https://manpages.debian.org/trixie/coreutils/tail.1.en.html), [tee](https://manpages.debian.org/trixie/coreutils/tee.1.en.html), [ls](https://manpages.debian.org/trixie/coreutils/ls.1.en.html)                                                              |
| cp, mv, rm, wc, sort, uniq | GNU coreutils 9.7               | [cp](https://manpages.debian.org/trixie/coreutils/cp.1.en.html), [mv](https://manpages.debian.org/trixie/coreutils/mv.1.en.html), [rm](https://manpages.debian.org/trixie/coreutils/rm.1.en.html), [wc](https://manpages.debian.org/trixie/coreutils/wc.1.en.html), [sort](https://manpages.debian.org/trixie/coreutils/sort.1.en.html), [uniq](https://manpages.debian.org/trixie/coreutils/uniq.1.en.html) |
| grep                       | GNU grep 3.11, Debian 3.11-4    | [Debian grep manual](https://manpages.debian.org/trixie/grep/grep.1.en.html)                                                                                                                                                                                                                                                                                                                                 |

Reference access date: 2026-09-16. The fixtures are authored from these documented
contracts, not captured output from a running Linux oracle. No usable Docker/WSL
reference environment was available. Current GNU web manuals may describe newer
versions; the versioned Debian pages above anchor this pass. No complete manpage
or upstream implementation is copied into the project.

## Programs Updated

### Shell / Bash subset — PARTIAL

- Per-stage input and output descriptors for `<`, `0<`, `>`, `1>`, `>>`, `1>>`,
  `2>`, `2>>`, `2>&1`, `1>&2` and `>&2`.
- Redirections work before, within and after command arguments, and are applied
  from left to right. Quoted/escaped/expanded digits stay ordinary arguments.
- Pipes connect stdout only; explicit duplication may include stderr. A file
  redirect overrides the pipeline endpoint. Last stage determines exit status.
- Opening/truncating a redirect occurs before command execution and survives a
  command failure. Syntax errors are detected before any file is opened.
- Redirection-only commands and virtual `/dev/null` input/output work.
- Pipeline stages isolate cwd, user, host and environment while sharing VFS
  effects. Redirect targets retain the host selected when they were opened.
- Ordered output preserves stdout/stderr interleaving across script commands,
  descriptor duplication and the frontend renderer.
- Separate stdin/stdout/stderr TTY state is runtime-only and restored afterward.
- Syntax errors return 2; missing executables return 127; denied/invalid package
  executables return 126. A package CLI now respects PATH; absolute invocation
  no longer re-resolves the basename through PATH. Relative PATH entries work.
- Explicit VFS script paths check execution permission and run the virtual
  interpreter. No host executable is launched.

### Text and listing commands — PARTIAL

- `cat`, `head`, `tail`: default stdin and `-` operands consume textual pipeline
  and redirected input; repeated stdin operands do not replay the same data.
- `cat`: numbering/squeezing persist across file boundaries; errors and data are
  emitted in operand order. Common VFS read errors use standard English messages.
- `ls`: adds `-R`/`--recursive`, `--color[=always|auto|never]`; auto checks stdout's
  TTY state. `-aA` includes dot entries; long columns align; symlink type, target
  and `@` classification appear; human sizes support K through E.
- `grep`: adds `-r`/`--recursive`, directory traversal with VFS permissions,
  recursive filename prefixes and partial failure handling. Encountered symlinks
  are skipped; dereferencing is still unsupported. Stdin labels are conventional.
- Shared getopt-style errors distinguish unknown long/short options, with a help
  hint. APT, archive and other callers retain their own status handling.
- `fastfetch` / `neofetch`: redirected or piped output is plain automatically.
  Their native upstream versions and full option sets remain unverified.

### File operations — PARTIAL

- `cp` and `mv` accept multiple sources, `-t`/`--target-directory` and
  `-T`/`--no-target-directory`. Unknown flags fail before mutations.
- Recursive `cp` merges existing directory trees, includes hidden entries, keeps
  unrelated destination entries and supports the common `source/.` operand.
- `cp -f` retries unwritable destination files; `-n` prevents replacement;
  `-u` uses virtual modification times. Copies preserve binary blobs and media
  metadata, create fresh timestamps and apply the current fixed virtual umask 022.
- `mv` performs a VFS rename, checks parent permissions without requiring read
  access to moved contents, and preserves modes/owners/timestamps. Replacing a
  directory requires an empty destination directory.
- `cp -P` and recursive copies preserve final symlinks. Non-recursive `cp` follows
  final source links with a 40-link bound; ancestor/destination links remain out
  of scope. Capacity checks use logical size, including virtual sparse metadata.
- `rm -r` checks every directory/parent, does not follow symlinks, and keeps
  failed entries. `-d` removes empty directories. `-f` ignores missing paths,
  while permission errors still fail. Dot/dot-dot, root and trash infrastructure
  remain protected. Successful entries survive later errors in all three tools.
- Ordered verbose output and errors survive descriptor merging and persistence.
  Desktop copy collision numbering continues through the separate GUI operation.

### Text pipelines — PARTIAL

- `wc` consumes stdin/`-` and multiple files; supports line, word, character and
  byte counts, conventional labels, totals and `--total` modes. Read errors retain
  successful counts. Byte/character behavior observes C versus C.UTF-8.
- `sort` combines all inputs, compares decimal prefixes without floating-point
  precision loss, supports reverse/stable/unique/case/blank options, NUL record
  delimiters and sortedness checking. Disorder returns 1; errors return 2.
  `-o` reads inputs before writing, including when input equals output.
- `uniq` counts adjacent groups rather than occurrences across the whole file;
  supports repeated-only, unique-only, case folding, field/character skips,
  comparison widths, NUL records and a positional output file.
- Input/output remain bounded to 4 MiB. Sort has a 65,536-record limit; uniq has
  a 65,536-group limit. The VFS per-file size limit still applies.
- Help/man describe the actual subset. Version output is only the pinned first
  line, not a claim of complete upstream banner or behavioral compatibility.

## Compatibility Tests

Run **`pnpm cli:compat`** independently. After build dependencies have been fetched,
the command uses `cargo test --offline`; it never fetches documentation or starts
an external reference command. `pnpm cli:inventory` regenerates discovery.

`tests/cli/compat/shell.json` contains 76 exact cases with command, stdin,
environment, terminal width, TTY state, initial/expected VFS, stdout, stderr,
exit code and absent paths. No whitespace, numbers or dates are normalized.
Case coverage by program (a pipeline case can cover more than one program):

| Program            | Exact cases |
| ------------------ | ----------: |
| cat                |          14 |
| cp                 |           8 |
| mv                 |           3 |
| rm                 |           4 |
| wc                 |           8 |
| sort               |          12 |
| uniq               |           8 |
| head / tail / grep |      2 each |
| tee                |           1 |

The remaining cases exercise shell descriptors, redirection-only commands,
quoting, Bash scripts, exit status and command lookup. Counts describe tested
scenarios, not complete option coverage.

Rust integration tests additionally cover:

- runtime discovery against the checked-in inventory;
- 40/80/120/200-column TTY and pipe behavior;
- PATH, executable permissions and explicit script execution;
- recursive listing, symlinks, colors and recursive grep failures;
- redirection failures, persistent side effects and pipeline isolation;
- 1,728 deterministic lexer inputs and malformed descriptor syntax;
- virtual-only paths/network and a source guard against host process/network APIs;
- copy merges, readonly sources/destinations, rename metadata, full virtual disks,
  symlink handling, partial removal, and save/reload of failed multi-file commands;
- chained text pipelines, exact numeric sorting, adjacent groups, locale selection,
  record limits and preservation of output files when sorting input fails.

Frontend tests verify that validated IPC results retain stdout/stderr order.
The pre-existing archive, package, mission, persistence and terminal tests remain
part of full regression. Passing these cases does not verify every flag of a tool.

## Intentional Deviations / Unsupported Features

- Text pipelines are bounded and sequential, not concurrent kernel pipes.
  Binary pipes/append, streaming backpressure, SIGPIPE and arbitrary descriptor
  numbers/closure are unsupported. Archive binary stdout can still target a VFS
  file. Maximum output is 4 MiB; text VFS files are limited to 1 MiB.
- Live stdin editing/EOF for ordinary commands is not implemented. Foreground
  nano/archive/package interaction uses the existing adapters; redirected
  interactive prompts fail explicitly. General process signal/cancel work remains.
- Shell lists (`;`, `&&`, `||`), general globbing, command substitution, functions,
  loops, here documents and file descriptor persistence are outside this pass.
- Native scripts requiring those features are not advertised as supported.
- Symlink traversal, inode/link counts and physical disk allocation are not fully
  modeled. `ls` still uses single-column layout; color environment customization
  and locale-sensitive ordering remain unverified.
- File operation recursion is capped at 256 levels. A failed `cp -f` retry rolls
  back that individual file. General overwrite prompts, sticky bits and full
  inode semantics are unsupported. Removing the cwd returns the session to home.
- `wc` display widths (`-L`) and binary reads, sort field keys/merge/locale-specific
  collation, and uniq byte slicing/group separator modes remain unsupported.
  C and C.UTF-8 are the supported text locales; other locale names are rejected.
- Package versions in the repository do not prove CLI fidelity. The inventory
  keeps legacy tools and catalog adapters explicitly unverified.

## Security Boundary

All new operations use selected virtual filesystems and virtual package bindings.
`/dev/tcp` has no socket behavior. Windows paths are rejected by VFS normalization.
No player-provided command reaches a host shell, process launcher or real network.
On Windows the offline runner invokes the fixed project Cargo executable directly
with the bundled toolchain environment; it does not change PowerShell policies.
The offline test runner exists only under development scripts and is not imported
by production code. Real notebook battery detection is an existing separate UI
feature and is not exposed as CLI system telemetry.

## Remaining implementation order

1. Finish P0: streaming/cancellation, remaining shell builtins and complete PATH
   coverage for legacy/catalog commands; ambiguity and expansion edge cases.
2. P1/P2: interactive file-operation prompts, remaining cp/mv metadata and backup
   flags, cut/tr streams, locale and binary edge cases, ls layout, version/help
   consistency and symlink traversal. The implemented wc/sort/uniq subsets still
   have explicit limits documented above.
3. P3/P4: process/time/environment, network-specific parsers, curl versus wget,
   deterministic progress and accurate per-program failure codes.
4. P5/P6: compare archive and package contracts to pinned references, preserving
   their already tested binary formats, prompts and transactional VFS behavior.
5. P7: audit each real catalog tool by package/version; either implement the
   supported native subset or explicitly classify functionality outside scope.
6. Pin a reproducible Linux oracle when infrastructure permits; add differential
   fixtures without silently promoting document-derived tests to oracle results.

## Regression Results

Validated on Windows on 2026-09-16:

| Gate                              | Result                                                                             |
| --------------------------------- | ---------------------------------------------------------------------------------- |
| Frontend tests                    | 208 passed, 34 test files                                                          |
| Full Rust suite                   | 152 passed, 0 failed, 2 intentionally ignored; binary/doc test targets also passed |
| Clippy                            | All targets/features, warnings denied: passed                                      |
| TypeScript                        | Passed                                                                             |
| ESLint / Stylelint                | Passed with no lint errors                                                         |
| Rustfmt / Prettier                | Passed                                                                             |
| Content validation                | Passed: 12 missions, 11 threads, 8 hosts, 303 catalog entries                      |
| Web production build              | Passed; existing large-chunk and third-party annotation warnings                   |
| Standalone offline CLI runner     | Passed: 16 tests including 76 exact fixtures; inventory rediscovery matches        |
| Native Windows release executable | Passed: optimized game-hacker.exe, embedded production frontend                    |

The ignored Rust tests are the pre-existing package performance measurement and
Fastfetch visual-fixture export. Native UI E2E was not run in this checkpoint;
frontend tests and the Rust campaign/persistence integration tests ran. No Linux
oracle or differential execution was available. Installer packaging (MSI/NSIS)
is not included in the native executable build.

Full-project CLI compatibility remains **in progress**. Zero tools are marked
VERIFIED; passing regression gates does not replace upstream comparison.

Native output: `src-tauri/target/release/game-hacker.exe`. Build and test logs are
under `artifacts/cli-*.log`.
