use super::{
    expansion,
    syntax::{Condition, List, Pipeline, RedirectKind},
    Flow,
};
use crate::{
    error::GameResult,
    shell_pipeline::{Redirect, Stage},
    terminal_io::Output,
    vfs::domain,
    world::WorldState,
};

pub fn append(total: &mut Output, mut output: Output) -> GameResult<()> {
    if total.stdout.len() + total.stderr.len() + output.stdout.len() + output.stderr.len()
        > 4 * 1024 * 1024
    {
        return Err(domain("shell: output limit: 4 MiB"));
    }
    for (fd, text) in [(1, &output.stdout), (2, &output.stderr)] {
        let emitted: usize = output
            .ordered
            .iter()
            .filter(|(f, _)| *f == fd)
            .map(|(_, s)| s.len())
            .sum();
        if emitted < text.len() {
            output.ordered.push((fd, text[emitted..].into()));
        }
    }
    total.binary.get_or_insert_with(Vec::new).extend(
        output
            .binary
            .take()
            .unwrap_or_else(|| output.stdout.as_bytes().to_vec()),
    );
    if output.byte_ordered.is_empty() {
        output.byte_ordered.extend(
            output
                .ordered
                .iter()
                .map(|(fd, text)| (*fd, text.as_bytes().to_vec())),
        );
    }
    total
        .byte_ordered
        .extend(std::mem::take(&mut output.byte_ordered));
    total.stdout.push_str(&output.stdout);
    total.stderr.push_str(&output.stderr);
    total.ordered.extend(output.ordered);
    total.status = output.status;
    total.archive_job = output.archive_job;
    Ok(())
}
fn expand_pipeline(
    world: &mut WorldState,
    pipeline: &Pipeline,
    output: &mut Output,
) -> GameResult<Vec<Stage>> {
    let mut stages = Vec::new();
    for command in &pipeline.commands {
        let mut stage = Stage::default();
        let original_env = world.terminal.env.clone();
        let original_unset = world.terminal.shell.unset.clone();
        let result = (|| {
            let mut prefix = true;
            for item in &command.words {
                if prefix {
                    if let Some((key, rhs)) = item.assignment() {
                        let expanded = expansion::word(world, &rhs, true)?;
                        stage.assignment_status =
                            expanded.substitution_status.or(stage.assignment_status);
                        super::control::emit(2, &expanded.side_output.stderr);
                        append(output, expanded.side_output)?;
                        let value = expanded.values.join("");
                        world.terminal.env.insert(key.clone(), value.clone());
                        world.terminal.shell.unset.remove(&key);
                        if !stage.assignment_order.contains(&key) {
                            stage.assignment_order.push(key.clone());
                        }
                        stage.assignments.insert(key, value);
                        if stage
                            .assignments
                            .iter()
                            .map(|(key, value)| key.len() + value.len())
                            .sum::<usize>()
                            > 1024 * 1024
                        {
                            return Err(domain("shell: assignment prefix limit"));
                        }
                        continue;
                    }
                }
                world.terminal.env = original_env.clone();
                world.terminal.shell.unset = original_unset.clone();
                prefix = false;
                let export_value = stage.arguments.first().is_some_and(|s| s == "export")
                    && item.assignment().is_some();
                let expanded = expansion::word(world, item, export_value)?;
                stage.assignment_status = expanded.substitution_status.or(stage.assignment_status);
                super::control::emit(2, &expanded.side_output.stderr);
                append(output, expanded.side_output)?;
                stage.arguments.extend(expanded.values);
                expansion::check_limit(&stage.arguments)?;
            }
            world.terminal.env = original_env.clone();
            world.terminal.shell.unset = original_unset.clone();
            for redirect in &command.redirects {
                let expanded = expansion::word(world, &redirect.target, false)?;
                super::control::emit(2, &expanded.side_output.stderr);
                append(output, expanded.side_output)?;
                if expanded.values.len() != 1 {
                    return Err(domain("bash: ambiguous redirect"));
                }
                let target = expanded.values[0].clone();
                stage.redirects.push(match redirect.kind {
                    RedirectKind::Read => Redirect::Input(target),
                    RedirectKind::Write => Redirect::Output(redirect.fd, target, false),
                    RedirectKind::Append => Redirect::Output(redirect.fd, target, true),
                    RedirectKind::Duplicate if target == "1" || target == "2" => {
                        Redirect::Duplicate(redirect.fd, target.parse().unwrap())
                    }
                    _ => return Err(domain("bash: unsupported file descriptor redirection")),
                });
            }
            expansion::check_limit(&stage.arguments)
        })();
        world.terminal.env = original_env;
        world.terminal.shell.unset = original_unset;
        result?;
        stages.push(stage);
    }
    Ok(stages)
}
pub fn execute_list(world: &mut WorldState, list: &List) -> GameResult<Output> {
    let mut output = Output {
        status: if list.items.is_empty() {
            0
        } else {
            world.terminal.last_status
        },
        ..Output::default()
    };
    for item in &list.items {
        for (condition, pipeline) in &item.pipelines {
            if (*condition == Condition::Success && output.status != 0)
                || (*condition == Condition::Failure && output.status == 0)
            {
                continue;
            }
            if super::control::cancelled() {
                append(
                    &mut output,
                    Output {
                        status: 130,
                        ..Output::default()
                    },
                )?;
                break;
            }
            let stages = match expand_pipeline(world, pipeline, &mut output) {
                Ok(stages) => stages,
                Err(error) => {
                    append(
                        &mut output,
                        Output {
                            stderr: format!("{error}\n"),
                            status: 1,
                            ..Output::default()
                        },
                    )?;
                    world.terminal.last_status = output.status;
                    continue;
                }
            };
            if item.background
                && (item.pipelines.len() != 1
                    || stages.len() != 1
                    || !stages[0].redirects.is_empty()
                    || !stages[0].assignments.is_empty())
            {
                return Err(domain(
                    "bash: background execution is supported only for archive jobs",
                ));
            }
            if stages.len() == 1
                && stages[0].redirects.is_empty()
                && stages[0].assignments.is_empty()
            {
                let mut parts = stages[0].arguments.clone();
                if item.background {
                    parts.push("\0&".into());
                }
                if let Some(result) = crate::archive::jobs::maybe_start(world, &parts) {
                    world.terminal.shell.last_background = result.archive_job;
                    append(
                        &mut output,
                        Output {
                            stdout: result.stdout,
                            stderr: result.stderr,
                            status: result.exit_code,
                            archive_job: result.archive_job,
                            ordered: result.ordered,
                            ..Output::default()
                        },
                    )?;
                    world.terminal.last_status = output.status;
                    continue;
                }
            }
            if item.background {
                return Err(domain(
                    "bash: background execution is supported only for archive jobs",
                ));
            }
            let previous_foreground = world.terminal.foreground.clone();
            let mut result = crate::shell_pipeline::run_stages(world, &stages)?;
            if result.status != 0
                && stages
                    .first()
                    .and_then(|s| s.arguments.first())
                    .is_some_and(|name| crate::archive::cli::COMMANDS.contains(&name.as_str()))
            {
                let path = stages[0]
                    .arguments
                    .iter()
                    .skip(1)
                    .filter(|p| !p.starts_with('-'))
                    .find_map(|p| {
                        let path = crate::vfs::normalize(p, &world.terminal.cwd).ok()?;
                        world
                            .fs()
                            .ok()?
                            .nodes
                            .get(&path)
                            .filter(|n| n.kind == "file")
                            .map(|_| path)
                    });
                if let Some(path) = path {
                    crate::archive::record_failure(world, &path, &result.stderr, None);
                }
            }
            if world.terminal.shell_depth > 0 && world.terminal.nano.is_some() {
                world.terminal.nano = None;
                world.terminal.foreground = previous_foreground;
                result = Output {
                    stderr: "shell: interactive editors cannot run inside scripts\n".into(),
                    status: 2,
                    ..Output::default()
                };
            }
            append(&mut output, result)?;
            world.terminal.last_status = output.status;
            if matches!(world.terminal.shell.flow, Some(Flow::Exit | Flow::Return)) {
                return Ok(output);
            }
        }
        if super::control::cancelled() {
            break;
        }
    }
    Ok(output)
}
