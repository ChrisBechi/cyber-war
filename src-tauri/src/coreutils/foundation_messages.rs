//! Observable GNU 9.7 C-locale help/version catalog. See messages/NOTICE.md.
pub(super) fn message(name: &str, invocation: &str, kind: &str) -> String {
    let template = match (name, kind) {
        ("sha256sum", "help") => include_str!("messages/sha256sum-help.txt"),
        ("sha256sum", "version") => include_str!("messages/sha256sum-version.txt"),
        ("wc", "help") => include_str!("messages/wc-help.txt"),
        ("wc", "version") => include_str!("messages/wc-version.txt"),
        ("tee", "help") => include_str!("messages/tee-help.txt"),
        ("tee", "version") => include_str!("messages/tee-version.txt"),
        ("base64", "help") => include_str!("messages/base64-help.txt"),
        ("base64", "version") => include_str!("messages/base64-version.txt"),
        ("tail", "help") => include_str!("messages/tail-help.txt"),
        ("tail", "version") => include_str!("messages/tail-version.txt"),
        ("head", "help") => include_str!("messages/head-help.txt"),
        ("head", "version") => include_str!("messages/head-version.txt"),
        ("cat", "help") => include_str!("messages/cat-help.txt"),
        ("cat", "version") => include_str!("messages/cat-version.txt"),
        ("basename", "help") => include_str!("messages/basename-help.txt"),
        ("basename", "version") => include_str!("messages/basename-version.txt"),
        ("dirname", "help") => include_str!("messages/dirname-help.txt"),
        ("dirname", "version") => include_str!("messages/dirname-version.txt"),
        ("printenv", "help") => include_str!("messages/printenv-help.txt"),
        ("printenv", "version") => include_str!("messages/printenv-version.txt"),
        ("whoami", "help") => include_str!("messages/whoami-help.txt"),
        ("whoami", "version") => include_str!("messages/whoami-version.txt"),
        _ => {
            return if kind == "help" {
                super::legacy::help(name).unwrap_or_default()
            } else {
                super::version(name)
            }
        }
    };
    let version = super::version(name)
        .split_whitespace()
        .last()
        .unwrap_or("")
        .to_string();
    template
        .replace("\r\n", "\n")
        .replace("{invocation}", invocation)
        .replace("{version}", &version)
}
