# CYBER WAR — CLI Compatibility

Generated from runtime registrations and current evidence. PARTIAL is not certification.

| Metric               | Value                                          |
| -------------------- | ---------------------------------------------- |
| commandNames         | 1794                                           |
| uniqueExecutables    | 1523                                           |
| aliases              | 1                                              |
| launcherNames        | 270                                            |
| catalogLaunchers     | 303                                            |
| guiLaunchers         | 19                                             |
| softwareGroups       | 279                                            |
| realCompat           | 1516                                           |
| fictionalNative      | 7                                              |
| catalogOnly          | 1406                                           |
| commands             | {"VERIFIED":0,"PARTIAL":117,"UNVERIFIED":1406} |
| software             | {"VERIFIED":0,"PARTIAL":42,"UNVERIFIED":237}   |
| commandVerification  | {"verified":0,"total":1516,"percent":0}        |
| softwareVerification | {"verified":0,"total":243,"percent":0}         |
| pinnedSoftware       | 12                                             |
| unpinnedSoftware     | 231                                            |
| commandsTested       | 9                                              |
| gatesPassed          | 3739                                           |
| requiredGates        | {"passed":3190,"total":35022}                  |
| pilotCases           | {"PASS":68,"FAIL":0,"SKIPPED":0}               |
| flagCasesPassed      | 8                                              |
| errorCasesPassed     | 12                                             |

## Subsystem readiness

| Subsystem       | State   | Reason                                                                                                               |
| --------------- | ------- | -------------------------------------------------------------------------------------------------------------------- |
| SHELL           | PARTIAL | Typed shell foundation is implemented; general jobs and interruptible execution of legacy handlers remain partial.   |
| VFS             | PARTIAL | No complete inode, symlink traversal or sticky-bit model.                                                            |
| USERS           | PARTIAL | Fixed virtual identities; no general user database.                                                                  |
| PERMISSIONS     | PARTIAL | Limited mode/owner/group permissions.                                                                                |
| PROCESS         | PARTIAL | Virtual process list; no general process lifecycle API.                                                              |
| TTY             | PARTIAL | Virtual stdin/EOF and bounded acknowledged output reach xterm; raw/cooked terminal discipline remains partial.       |
| SIGNALS         | PARTIAL | Foreground shell Ctrl+C returns 130; archive cancellation remains supported; general signals/process groups pending. |
| CLOCK           | PARTIAL | Virtual playtime and VFS counter timestamps.                                                                         |
| NETWORK         | PARTIAL | Seeded hosts/services; no general sockets/listeners.                                                                 |
| DNS             | PARTIAL | Seeded resolver; incomplete DNS protocol.                                                                            |
| HTTP            | PARTIAL | Virtual routes/downloads; incomplete HTTP semantics.                                                                 |
| PACKAGES        | PARTIAL | Package DB tested; full native contracts still partial.                                                              |
| ARCHIVES        | PARTIAL | Binary formats tested; full flags/stream semantics partial.                                                          |
| WIFI            | PARTIAL | Access-point metadata exists; no frame/handshake engine.                                                             |
| PACKETS         | MISSING | No packet capture/frame model.                                                                                       |
| REMOTE_HOSTS    | PARTIAL | Seeded SSH contexts; no arbitrary remote lifecycle.                                                                  |
| SERVICES        | PARTIAL | Virtual service state; incomplete service manager.                                                                   |
| VIRTUAL_TARGETS | PARTIAL | Mission targets only; no general exploit target engine.                                                              |
| SESSIONS        | PARTIAL | Terminal and SSH contexts exist; no general framework sessions.                                                      |

## Next work

{
"softwareId": "coreutils",
"action": "NEEDS_SUBSYSTEM",
"blockers": [
"SUBSYSTEM:VFS:PARTIAL"
]
}
