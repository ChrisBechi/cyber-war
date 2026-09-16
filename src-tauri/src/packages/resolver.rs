use super::{model::*, version};
use crate::{error::GameResult, vfs::domain};
use std::collections::{BTreeMap, BTreeSet};

pub fn satisfies(p: &Definition, r: &Relation) -> bool {
    (p.name == r.name && version::matches(&p.version, &r.op, &r.version))
        || p.provides.iter().any(|v| {
            v.name == r.name
                && (r.op.is_empty()
                    || !v.version.is_empty() && version::matches(&v.version, &r.op, &r.version))
        })
}
pub fn candidates(state: &PackageState) -> Vec<IndexEntry> {
    let mut entries = state
        .indexes
        .values()
        .flatten()
        .filter(|e| ["amd64", "all"].contains(&e.package.architecture.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        a.package
            .name
            .cmp(&b.package.name)
            .then_with(|| version::compare(&b.package.version, &a.package.version))
            .then_with(|| a.repository.cmp(&b.repository))
    });
    entries.dedup_by(|a, b| a.package.key() == b.package.key());
    entries
}
pub fn broken(p: &Definition, installed: &BTreeMap<String, Installed>) -> Vec<String> {
    p.depends
        .iter()
        .filter(|clause| {
            !clause.iter().any(|r| {
                installed
                    .values()
                    .any(|i| i.status == Status::Installed && satisfies(&i.definition, r))
            })
        })
        .map(|clause| {
            clause
                .iter()
                .map(|r| format!("{} {} {}", r.name, r.op, r.version))
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .collect()
}
pub fn resolve(
    state: &PackageState,
    requested: &[String],
    repair: bool,
) -> GameResult<Vec<IndexEntry>> {
    let available = candidates(state);
    let mut result = Vec::new();
    let mut selected: BTreeMap<String, Definition> = BTreeMap::new();
    let mut visiting = BTreeSet::new();
    let mut budget = 10000usize;
    fn visit(
        e: IndexEntry,
        state: &PackageState,
        available: &[IndexEntry],
        selected: &mut BTreeMap<String, Definition>,
        visiting: &mut BTreeSet<String>,
        result: &mut Vec<IndexEntry>,
        budget: &mut usize,
    ) -> GameResult<()> {
        if *budget == 0 {
            return Err(domain("E: Dependency resolution work limit exceeded"));
        }
        *budget -= 1;
        let name = e.package.name.clone();
        if visiting.len() >= 128 {
            return Err(domain("E: Dependency graph exceeds the safe depth of 128"));
        }
        if visiting.contains(&name) {
            return Err(domain(format!("E: Dependency cycle involving {name}")));
        }
        if let Some(old) = selected.get(&name) {
            if old.version == e.package.version {
                return Ok(());
            }
            return Err(domain(format!(
                "E: Conflicting version constraints for {name}"
            )));
        }
        if state
            .installed
            .get(&name)
            .is_some_and(|i| i.held && i.definition.version != e.package.version)
        {
            return Err(domain(format!("E: {name} is held")));
        }
        for other in selected.values().chain(
            state
                .installed
                .values()
                .filter(|i| {
                    i.status != Status::ConfigFiles && !selected.contains_key(&i.definition.name)
                })
                .map(|i| &i.definition),
        ) {
            if other.name != name
                && (e.package.conflicts.iter().any(|r| satisfies(other, r))
                    || other.conflicts.iter().any(|r| satisfies(&e.package, r)))
            {
                return Err(domain(format!("E: {name} conflicts with {}", other.name)));
            }
        }
        visiting.insert(name.clone());
        selected.insert(name.clone(), e.package.clone());
        for clause in &e.package.depends {
            if clause.iter().any(|r| {
                state.installed.values().any(|i| {
                    i.status == Status::Installed
                        && !selected.contains_key(&i.definition.name)
                        && satisfies(&i.definition, r)
                })
            }) {
                continue;
            }
            if clause.iter().any(|r| {
                selected
                    .values()
                    .any(|p| !visiting.contains(&p.name) && satisfies(p, r))
            }) {
                continue;
            }
            let mut resolved = false;
            let mut last_error = domain(format!("E: Unmet dependencies for {name}"));
            for dependency in clause
                .iter()
                .flat_map(|r| available.iter().filter(move |p| satisfies(&p.package, r)))
            {
                let mut next_selected = selected.clone();
                let mut next_visiting = visiting.clone();
                let mut next_result = result.clone();
                match visit(
                    dependency.clone(),
                    state,
                    available,
                    &mut next_selected,
                    &mut next_visiting,
                    &mut next_result,
                    budget,
                ) {
                    Ok(()) => {
                        *selected = next_selected;
                        *visiting = next_visiting;
                        *result = next_result;
                        resolved = true;
                        break;
                    }
                    Err(error) => last_error = error,
                }
            }
            if !resolved {
                return Err(last_error);
            }
        }
        visiting.remove(&name);
        result.push(e);
        Ok(())
    }
    let mut requests = requested.to_vec();
    if repair {
        requests.extend(
            state
                .installed
                .values()
                .filter(|i| matches!(i.status, Status::Unpacked | Status::HalfConfigured))
                .map(|i| i.definition.name.clone()),
        );
    }
    for request in requests {
        let (name, version) = request
            .split_once('=')
            .map_or((request.as_str(), None), |(n, v)| (n, Some(v)));
        let entry = available
            .iter()
            .find(|e| e.package.name == name && version.is_none_or(|v| v == e.package.version))
            .cloned()
            .or_else(|| {
                if repair {
                    state.installed.get(name).map(|i| IndexEntry {
                        package: i.definition.clone(),
                        repository: "installed".into(),
                        suite: "local".into(),
                        component: "main".into(),
                        checksum: String::new(),
                    })
                } else {
                    None
                }
            })
            .ok_or_else(|| domain(format!("E: Unable to locate package {request}")))?;
        if let Some(old) = selected.get(name) {
            if old.version != entry.package.version {
                return Err(domain(format!(
                    "E: Conflicting version constraints for {name}"
                )));
            }
            continue;
        }
        visit(
            entry,
            state,
            &available,
            &mut selected,
            &mut visiting,
            &mut result,
            &mut budget,
        )?;
    }
    let final_packages = state
        .installed
        .values()
        .filter(|i| i.status != Status::ConfigFiles && !selected.contains_key(&i.definition.name))
        .map(|i| &i.definition)
        .chain(selected.values())
        .collect::<Vec<_>>();
    for p in &final_packages {
        for other in &final_packages {
            if p.name != other.name && p.conflicts.iter().any(|r| satisfies(other, r)) {
                return Err(domain(format!(
                    "E: {} conflicts with {}; remove it first",
                    p.name, other.name
                )));
            }
        }
        if p.depends.iter().any(|clause| {
            !clause
                .iter()
                .any(|r| final_packages.iter().any(|p| satisfies(p, r)))
        }) {
            return Err(domain(format!(
                "E: Unmet reverse dependency for {}",
                p.name
            )));
        }
    }
    Ok(result)
}
pub fn orphans(state: &PackageState) -> Vec<String> {
    let mut needed = state
        .installed
        .values()
        .filter(|i| {
            i.status != Status::ConfigFiles && (!i.automatic || i.held || i.definition.essential)
        })
        .map(|i| i.definition.name.clone())
        .collect::<BTreeSet<_>>();
    let mut pending = needed.iter().cloned().collect::<Vec<_>>();
    while let Some(name) = pending.pop() {
        if let Some(p) = state.installed.get(&name) {
            for clause in &p.definition.depends {
                if let Some(dependency) = clause.iter().find_map(|r| {
                    state
                        .installed
                        .values()
                        .find(|i| i.status != Status::ConfigFiles && satisfies(&i.definition, r))
                }) {
                    if needed.insert(dependency.definition.name.clone()) {
                        pending.push(dependency.definition.name.clone());
                    }
                }
            }
        }
    }
    state
        .installed
        .values()
        .filter(|i| {
            i.automatic && !needed.contains(&i.definition.name) && i.status != Status::ConfigFiles
        })
        .map(|i| i.definition.name.clone())
        .collect()
}
