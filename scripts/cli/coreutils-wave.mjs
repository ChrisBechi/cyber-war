// The existing evidence-derived queue owns selection. This policy classifies
// that exact next entry; it cannot skip a difficult executable or edit source.
export const EXECUTABLE_LIMIT = 12;
export function waveDecision(queue, processed = []) {
  if (new Set(processed).size !== processed.length || processed.length > EXECUTABLE_LIMIT)
    throw new Error('Invalid processed executable count');
  if (processed.length && processed[0] !== 'head') throw new Error('The wave must begin with head');
  const next = queue.next;
  if (processed.length === EXECUTABLE_LIMIT)
    return { next, classification: null, continue: false, stopReason: 'EXECUTABLE_LIMIT' };
  if (!next)
    return { next: null, classification: null, continue: false, stopReason: 'QUEUE_COMPLETE' };
  const blockers = next.blockers ?? [];
  const architecture = blockers.filter((b) =>
    /follow.*notifications|file.*notifications|job control|process group|external process|locale database/i.test(
      b,
    ),
  );
  if (architecture.length)
    return {
      next,
      classification: 'ARCHITECTURAL_BOUNDARY',
      continue: false,
      stopReason: 'ARCHITECTURAL_BOUNDARY',
      reasons: architecture,
    };
  if (!['reading', 'text'].includes(next.area))
    return {
      next,
      classification: 'OUT_OF_SCOPE_FAMILY',
      continue: false,
      stopReason: 'OUT_OF_SCOPE_FAMILY',
      reasons: [`Semantic area: ${next.area}`],
    };
  const subsystem = blockers.filter((b) => b.startsWith('SUBSYSTEM:'));
  if (subsystem.length)
    return {
      next,
      classification: 'ARCHITECTURAL_BOUNDARY',
      continue: false,
      stopReason: 'ARCHITECTURAL_BOUNDARY',
      reasons: subsystem,
    };
  return {
    next,
    classification: blockers.some((b) => b.startsWith('GAP:'))
      ? 'SMALL_SHARED_EXTENSION'
      : 'SAME_WAVE',
    continue: true,
    stopReason: null,
    reasons: blockers,
  };
}
