//! Deterministic virtual wait conditions and explicitly advanced simulation time.
//! Timers never consult the operating-system clock or a rendering frame.
use crate::{
    error::GameResult,
    vfs::{domain, WatchId},
    world::WorldState,
};
use std::collections::BTreeMap;

pub const TIMER_LIMIT: usize = 1024;
pub const TICKS_PER_SECOND: u64 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimerId(u64);

#[derive(Debug, Clone)]
struct Timer {
    owner: u32,
    deadline: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Runtime {
    pub now: u64,
    next: u64,
    timers: BTreeMap<TimerId, Timer>,
    pub waiting: BTreeMap<u32, WaitSet>,
    pub signals: BTreeMap<u32, super::signals::VirtualSignal>,
}
impl Runtime {
    pub fn timer(&mut self, owner: u32, delay: u64) -> GameResult<TimerId> {
        if self.timers.len() >= TIMER_LIMIT {
            return Err(domain("virtual timer limit exceeded"));
        }
        let deadline = self
            .now
            .checked_add(delay)
            .ok_or_else(|| domain("virtual timer overflow"))?;
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| domain("virtual timer allocator exhausted"))?;
        let id = TimerId(self.next);
        self.timers.insert(id, Timer { owner, deadline });
        Ok(id)
    }
    pub fn advance(&mut self, ticks: u64) -> GameResult<()> {
        self.now = self
            .now
            .checked_add(ticks)
            .ok_or_else(|| domain("virtual time overflow"))?;
        Ok(())
    }
    pub fn fired(&self, id: TimerId) -> bool {
        self.timers.get(&id).is_none_or(|t| t.deadline <= self.now)
    }
    pub fn cancel_timer(&mut self, id: TimerId) {
        self.timers.remove(&id);
    }
    /// Discrete-event time: when every stage is blocked, jump to the earliest
    /// finite timer it is waiting on. Idle subscriptions create no periodic
    /// timers, so this cannot turn an unchanged file into a busy polling loop.
    pub fn advance_idle(&mut self, wait: &WaitSet) -> bool {
        let next = wait
            .timers
            .iter()
            .filter_map(|id| self.timers.get(id))
            .map(|t| t.deadline)
            .min();
        if let Some(next) = next.filter(|n| *n > self.now) {
            self.now = next;
            true
        } else {
            false
        }
    }
    pub fn finish(&mut self, owner: u32) {
        self.signals.remove(&owner);
        self.waiting.remove(&owner);
        self.timers.retain(|_, t| t.owner != owner);
    }
    pub fn timer_count(&self) -> usize {
        self.timers.len()
    }
    pub(crate) fn release_runtime_closed(&mut self, before: &Self, after: &Self) {
        for pid in before
            .signals
            .keys()
            .filter(|pid| !after.signals.contains_key(pid))
        {
            self.signals.remove(pid);
        }
        for id in before
            .timers
            .keys()
            .filter(|id| !after.timers.contains_key(id))
        {
            self.timers.remove(id);
        }
        for pid in before
            .waiting
            .keys()
            .filter(|pid| !after.waiting.contains_key(pid))
        {
            self.waiting.remove(pid);
        }
    }
}

#[derive(Debug, Clone)]
pub struct VfsWait {
    pub host: Option<String>,
    pub watch: WatchId,
    pub revision: u64,
}

#[derive(Debug, Clone, Default)]
pub struct WaitSet {
    pub world: Option<u64>,
    pub vfs: Vec<VfsWait>,
    pub processes: Vec<(u32, bool)>,
    pub timers: Vec<TimerId>,
    pub input: bool,
}
impl WaitSet {
    pub fn ready(&self, world: &WorldState) -> bool {
        if self.world.is_some_and(|id| id != world.runtime_id) {
            return false;
        }
        self.vfs.iter().any(|w| {
            let fs = match &w.host {
                None => Some(&world.vfs),
                Some(host) => world.network.hosts.get(host).map(|h| &h.files),
            };
            fs.and_then(|fs| fs.watch_revision(w.watch)) != Some(w.revision)
        }) || self.processes.iter().any(|(pid, alive)| {
            world.processes.iter().any(|p| p.pid == *pid && p.running) != *alive
        }) || self.timers.iter().any(|id| world.scheduler.fired(*id))
    }
    pub fn extend(&mut self, other: &Self) {
        self.vfs.extend(other.vfs.iter().cloned());
        self.processes.extend(other.processes.iter().copied());
        self.timers.extend(other.timers.iter().copied());
        self.input |= other.input;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::WatchTarget;
    #[test]
    fn timers_have_bounded_ownership_deadlines_and_checked_overflow() {
        let mut runtime = Runtime::default();
        let late = runtime.timer(1, 100).unwrap();
        let early = runtime.timer(2, 7).unwrap();
        let wait = WaitSet {
            timers: vec![late, early],
            ..Default::default()
        };
        assert!(runtime.advance_idle(&wait));
        assert_eq!(runtime.now, 7);
        assert!(runtime.fired(early));
        assert!(!runtime.fired(late));
        assert!(!runtime.advance_idle(&wait));
        runtime.finish(2);
        assert_eq!(runtime.timer_count(), 1);
        assert!(runtime.advance_idle(&wait));
        assert_eq!(runtime.now, 100);
        runtime.cancel_timer(late);
        assert_eq!(runtime.timer_count(), 0);
        for index in 0..TIMER_LIMIT {
            runtime.timer(3, index as u64).unwrap();
            assert_eq!(runtime.timer_count(), index + 1);
        }
        assert!(runtime.timer(4, 1).is_err());
        assert_eq!(runtime.timer_count(), TIMER_LIMIT);
        runtime.finish(3);
        assert_eq!(runtime.timer_count(), 0);
        runtime.now = u64::MAX;
        assert!(runtime.timer(1, 1).is_err());
        assert!(runtime.advance(1).is_err());
        assert_eq!(runtime.now, u64::MAX);
        assert_eq!(runtime.timer_count(), 0);
    }

    #[test]
    fn event_or_timer_wait_is_world_local_and_runtime_is_not_saved() {
        let mut world = WorldState::new("kali", "pc").unwrap();
        world.vfs.write("/home/kali/log", "start", "kali").unwrap();
        let watch = world
            .vfs
            .subscribe(WatchTarget::Path("/home/kali/log".into()))
            .unwrap();
        let timer = world.scheduler.timer(17, 20).unwrap();
        let wait = WaitSet {
            world: Some(world.runtime_id),
            vfs: vec![VfsWait {
                host: None,
                watch,
                revision: world.vfs.watch_revision(watch).unwrap(),
            }],
            timers: vec![timer],
            ..Default::default()
        };
        world.scheduler.waiting.insert(17, wait.clone());
        world
            .scheduler
            .signals
            .insert(17, super::super::signals::VirtualSignal::Term);
        let handle = world
            .vfs
            .open(
                "/home/kali/log",
                crate::vfs::OpenFlags {
                    read: true,
                    ..Default::default()
                },
                0,
                "kali",
            )
            .unwrap();
        let restored: WorldState =
            serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
        assert_ne!(restored.runtime_id, world.runtime_id);
        assert_eq!(restored.vfs.watcher_count(), 0);
        assert_eq!(restored.vfs.open_handle_count(), 0);
        assert_eq!(restored.scheduler.timer_count(), 0);
        assert!(restored.scheduler.waiting.is_empty());
        assert!(restored.scheduler.signals.is_empty());
        assert!(!wait.ready(&restored));
        world
            .vfs
            .write("/home/kali/log", "changed", "kali")
            .unwrap();
        assert!(wait.ready(&world));
        assert_eq!(
            world.scheduler.now, 0,
            "the event wins before the timer deadline"
        );
        world.scheduler.finish(17);
        world.vfs.unsubscribe(watch);
        world.vfs.close(handle).unwrap();
        assert_eq!(world.scheduler.timer_count(), 0);
        assert!(world.scheduler.signals.is_empty());
        assert!(world.scheduler.waiting.is_empty());
    }
    #[test]
    fn virtual_wait_sets_combine_changes_timers_and_process_lifetime() {
        let mut world = WorldState::new("kali", "pc").unwrap();
        let watch = world
            .vfs
            .subscribe(WatchTarget::Path("/home/kali/log".into()))
            .unwrap();
        let timer = world.scheduler.timer(17, TICKS_PER_SECOND).unwrap();
        let mut wait = WaitSet {
            vfs: vec![VfsWait {
                host: None,
                watch,
                revision: world.vfs.watch_revision(watch).unwrap(),
            }],
            timers: vec![timer],
            ..Default::default()
        };
        assert!(!wait.ready(&world));
        world.scheduler.advance(TICKS_PER_SECOND - 1).unwrap();
        assert!(!wait.ready(&world));
        world.scheduler.advance(1).unwrap();
        assert!(wait.ready(&world));
        world.scheduler.cancel_timer(timer);
        wait.timers.clear();
        assert!(!wait.ready(&world));
        world
            .vfs
            .write("/home/kali/other", "unrelated", "kali")
            .unwrap();
        assert!(!wait.ready(&world));
        world.vfs.write("/home/kali/log", "new", "kali").unwrap();
        assert!(wait.ready(&world));
        world.scheduler.waiting.insert(17, wait);
        world.scheduler.timer(17, 200).unwrap();
        world.scheduler.finish(17);
        world.vfs.unsubscribe(watch);
        assert_eq!(world.scheduler.timer_count(), 0);
        assert!(world.scheduler.waiting.is_empty());
        let restored: WorldState =
            serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
        assert_eq!(restored.scheduler.now, 0);
        assert_eq!(restored.scheduler.timer_count(), 0);
    }
}
