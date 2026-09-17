//! Guard legacy subsets before they discard unknown options. The existing
//! command-specific parsers still own operand/value semantics.
use crate::{error::GameResult, terminal_io::Output};
use std::sync::LazyLock;
static CONTRACTS: LazyLock<serde_json::Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../content/cli-compatibility/coreutils.json"
    ))
    .expect("validated Coreutils contracts")
});

pub(crate) fn help(name: &str) -> Option<String> {
    let spec = CONTRACTS["commands"].get(name)?;
    let flags = spec["flags"]
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let scope = spec["contracts"][0]["description"].as_str()?;
    let gaps = spec["knownGaps"]
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    Some(format!("{name}(1) — CYBER WAR virtual subset\n\nUsage: {name} {flags} [OPERAND]...\n\n{scope}\n\nLimits: {gaps}\n--help and --version are supported. Unlisted options are unsupported.\n"))
}

pub(super) fn validate(name: &str, args: &[String]) -> GameResult<Option<Output>> {
    let Some(spec) = CONTRACTS["commands"].get(name) else {
        return Ok(None);
    };
    let flags: Vec<&str> = spec["flags"]
        .as_array()
        .expect("flags")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    if let Some(out) =
        super::options::early(name, args, &format!("{} [OPERAND]...", flags.join(" ")))
    {
        return Ok(Some(out));
    }
    let values = match name {
        "mkdir" => "m",
        "cut" => "df",
        "seq" => "s",
        "stat" => "c",
        "cp" | "mv" => "t",
        "sort" => "o",
        "uniq" => "fsw",
        _ => "",
    };
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if arg == "--" {
            break;
        }
        if !arg.starts_with('-') || arg == "-" || (name == "seq" && arg.parse::<f64>().is_ok()) {
            continue;
        }
        if arg.starts_with("--") {
            let flag = arg.split('=').next().unwrap_or(arg);
            if !flags.contains(&flag) {
                return Err(super::options::operand_error(
                    name,
                    &format!("unrecognized option '{arg}'"),
                ));
            }
            skip = !arg.contains('=')
                && matches!(
                    flag,
                    "--mode"
                        | "--delimiter"
                        | "--fields"
                        | "--separator"
                        | "--format"
                        | "--target-directory"
                        | "--output"
                        | "--skip-fields"
                        | "--skip-chars"
                        | "--check-chars"
                );
        } else {
            let mut chars = arg[1..].chars().peekable();
            while let Some(c) = chars.next() {
                if !flags.iter().any(|f| *f == format!("-{c}")) {
                    return Err(super::options::operand_error(
                        name,
                        &format!("invalid option -- '{c}'"),
                    ));
                }
                if values.contains(c) {
                    skip = chars.peek().is_none();
                    break;
                }
            }
        }
    }
    Ok(None)
}
