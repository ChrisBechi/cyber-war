//! Runtime discovery facade. Certification stays in development tooling.
use std::{collections::BTreeSet, sync::LazyLock};

pub static NAMES: LazyLock<BTreeSet<String>> = LazyLock::new(|| {
    let mut names: BTreeSet<String> = crate::terminal::COMMANDS
        .iter()
        .chain(crate::shell::SHELL_ONLY.iter())
        .map(|s| s.to_string())
        .collect();
    for entry in &crate::software::CATALOG.entries {
        names.insert(entry.id.clone());
        names.insert(entry.name.clone());
        names.extend(entry.commands.clone());
    }
    for repo in crate::packages::repository::REPOSITORIES.values() {
        for (_, entry) in &repo.entries {
            names.extend(
                entry
                    .package
                    .files
                    .iter()
                    .filter(|f| f.binding.is_some())
                    .filter_map(|f| f.path.rsplit('/').next().map(String::from)),
            );
        }
    }
    names
});

pub fn available(world: &crate::world::WorldState, name: &str) -> bool {
    world.terminal.host.is_some()
        || !crate::packages::executables::managed(name)
        || crate::packages::executables::resolve(world, name, &world.terminal.user).is_some()
}
