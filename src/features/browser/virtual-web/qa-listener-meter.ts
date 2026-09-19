// Counts explicitly registered window/document listeners without retaining detached DOM nodes.
export function installListenerMeter() {
  const targets = [window, document];
  const counters = targets.map((target) => {
    const active = new Map<string, Set<EventListenerOrEventListenerObject>>();
    const add = target.addEventListener.bind(target);
    const remove = target.removeEventListener.bind(target);
    const key = (type: string, options?: boolean | AddEventListenerOptions) =>
      `${type}:${typeof options === 'boolean' ? options : !!options?.capture}`;
    target.addEventListener = ((
      type: string,
      listener: EventListenerOrEventListenerObject | null,
      options?: boolean | AddEventListenerOptions,
    ) => {
      if (listener && !(typeof options === 'object' && (options.once || options.signal))) {
        const id = key(type, options),
          set = active.get(id) ?? new Set<EventListenerOrEventListenerObject>();
        set.add(listener);
        active.set(id, set);
      }
      add(type, listener as EventListener, options);
    }) as typeof target.addEventListener;
    target.removeEventListener = ((
      type: string,
      listener: EventListenerOrEventListenerObject | null,
      options?: boolean | EventListenerOptions,
    ) => {
      if (listener) {
        active.get(key(type, options))?.delete(listener);
      }
      remove(type, listener as EventListener, options);
    }) as typeof target.removeEventListener;
    return () => [...active.values()].reduce((sum, set) => sum + set.size, 0);
  });
  return () => ({
    window: counters[0](),
    document: counters[1](),
    scope: 'explicit add/remove on window and document; excludes once/signal and node listeners',
  });
}
