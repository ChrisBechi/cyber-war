//! ln's confirmation reader on shared pipes, virtual descriptors and VirtualTty.
use super::*;
pub(super) fn step(
    world: &mut WorldState,
    process: &mut Process,
    pipes: &mut [Pipe],
) -> GameResult<bool> {
    let Engine::Links(links) = &mut process.engine else {
        unreachable!()
    };
    let output = if links.waiting {
        if let Some(output) = links.answer(world) {
            output
        } else {
            match process.input.read(world, pipes) {
                Ok(Read::Data(data)) => links.feed(data),
                Ok(Read::Eof) => links.eof(),
                Ok(Read::Pending | Read::Interrupted) => return Ok(false),
                Err(_) => links.read_error(),
            }
            return Ok(true);
        }
    } else {
        links.advance(world)
    };
    process.status = process.status.max(output.status);
    chunks(&mut process.pending, 1, output.stdout.into_bytes());
    chunks(&mut process.pending, 2, output.stderr.into_bytes());
    if links.finished {
        process.engine = Engine::Finished;
        process.input.close(pipes);
    }
    Ok(true)
}
