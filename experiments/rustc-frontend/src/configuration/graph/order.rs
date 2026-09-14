//! Iterative graph validation and deterministic dependency-first order.
use super::CrateDescription;
use crate::CrateKey;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn checked(
    root: &CrateKey,
    crates: &BTreeMap<CrateKey, CrateDescription>,
) -> Result<Vec<CrateKey>, String> {
    if !crates.contains_key(root) {
        return Err("crate graph root is not declared".into());
    }
    let mut counts = BTreeMap::new();
    let mut consumers = BTreeMap::<CrateKey, BTreeSet<CrateKey>>::new();
    for (key, description) in crates {
        let dependencies: BTreeSet<_> = description.dependencies.values().cloned().collect();
        for dependency in &dependencies {
            if !crates.contains_key(dependency) {
                return Err("crate graph dependency is not declared".into());
            }
            consumers
                .entry(dependency.clone())
                .or_default()
                .insert(key.clone());
        }
        counts.insert(key.clone(), dependencies.len());
    }
    let mut reached = BTreeSet::new();
    let mut pending = vec![root.clone()];
    while let Some(key) = pending.pop() {
        if reached.insert(key.clone()) {
            pending.extend(crates[&key].dependencies.values().cloned());
        }
    }
    if reached.len() != crates.len() {
        return Err("crate graph includes a node unreachable from its root".into());
    }
    let mut ready: BTreeSet<_> = counts
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(key, _)| key.clone())
        .collect();
    let mut result = Vec::new();
    while let Some(key) = ready.pop_first() {
        if let Some(users) = consumers.get(&key) {
            for user in users {
                let count = counts.get_mut(user).unwrap();
                *count = count
                    .checked_sub(1)
                    .ok_or("crate graph edge count underflow")?;
                if *count == 0 {
                    ready.insert(user.clone());
                }
            }
        }
        result.push(key);
    }
    if result.len() != crates.len() {
        return Err("crate graph contains a dependency cycle".into());
    }
    Ok(result)
}
