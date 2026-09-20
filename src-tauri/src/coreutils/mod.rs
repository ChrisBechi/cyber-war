//! Virtual Coreutils handlers. Development reference execution lives in scripts/, never here.
mod backup;
pub(crate) mod base64;
pub(crate) mod cat;
mod directories;
#[cfg(test)]
mod directories_tests;
mod foundation;
mod foundation_messages;
mod foundation_options;
pub(crate) mod head;
pub(crate) mod io;
mod legacy;
pub(crate) mod links;
#[cfg(test)]
mod links_tests;
mod options;
mod pathnames;
#[cfg(test)]
mod pathnames_tests;
pub(crate) mod sha256sum;
#[cfg(test)]
mod sha256sum_tests;
pub(crate) mod tail;
pub(crate) mod tee;
#[cfg(test)]
mod tee_tests;
pub(crate) mod wc;
#[cfg(test)]
mod wc_tests;
pub(crate) use legacy::help;
#[cfg(test)]
mod base64_tests;
#[cfg(test)]
mod performance_tests;
#[cfg(test)]
mod tail_tests;
#[cfg(test)]
mod tests;

use crate::{error::GameResult, terminal_io::Output, world::WorldState};

pub(crate) fn execute(
    world: &mut WorldState,
    invocation: &str,
    args: &[String],
    actor: &str,
) -> Option<GameResult<Output>> {
    let name = invocation.rsplit('/').next().unwrap_or(invocation);
    let result = match name {
        "mkdir" | "rmdir" => directories::execute(world, name, invocation, args, actor),
        "readlink" | "realpath" => pathnames::execute(world, name, invocation, args, actor),
        "basename" | "dirname" | "printenv" | "whoami" | "env" => {
            foundation::execute(world, name, args, actor, invocation)
        }
        "cat" | "head" | "tail" | "base64" | "tee" | "wc" | "sha256sum" | "ln" => {
            cat::execute(world, invocation, args, actor)
        }
        _ => match legacy::validate(name, args) {
            Ok(Some(out)) => Ok(out),
            Ok(None) => return None,
            Err(error) => Err(error),
        },
    };
    Some(Ok(result.unwrap_or_else(|error| Output {
        stderr: format!("{error}\n"),
        status: if matches!(name, "sort" | "printenv") {
            2
        } else if name == "env" {
            125
        } else {
            1
        },
        ..Default::default()
    })))
}

pub(crate) fn version(name: &str) -> String {
    // One source of truth shared with the development compatibility pipeline.
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../content/cli-compatibility/manifest.json"
    ))
    .expect("validated compatibility manifest");
    format!(
        "{name} (GNU coreutils) {}\n",
        manifest["software"]["coreutils"]["referenceVersion"]
            .as_str()
            .expect("pinned baseline")
    )
}
