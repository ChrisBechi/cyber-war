//! Virtual Coreutils handlers. Development reference execution lives in scripts/, never here.
mod bytes;
pub(crate) mod cat;
mod foundation;
mod foundation_messages;
mod foundation_options;
pub(crate) mod head;
pub(crate) mod io;
mod legacy;
mod options;
pub(crate) mod tail;
pub(crate) use legacy::help;
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
        "basename" | "dirname" | "printenv" | "whoami" | "env" => {
            foundation::execute(world, name, args, actor, invocation)
        }
        "cat" | "head" | "tail" => cat::execute(world, invocation, args, actor),
        "tee" | "base64" | "sha256sum" => bytes::execute(world, name, args, actor),
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
