//! Exact per-crate scope derived from an already validated immutable graph.
use super::{CrateGraph, CrateKey};
use std::collections::{BTreeMap, BTreeSet};

impl CrateGraph {
    /// Retain this crate and only its declared transitive dependency closure.
    /// Preserves descriptors and aliases; grants no compiler/target authority.
    pub fn rooted_at(&self, key: &str) -> Result<Self, String> {
        let root = CrateKey::new(key)?;
        let nodes: BTreeMap<_, _> = self
            .crates
            .iter()
            .map(|node| (node.key_value(), node))
            .collect();
        if !nodes.contains_key(&root) {
            return Err("requested subgraph root is not declared".into());
        }
        let mut seen = BTreeSet::new();
        let mut pending = vec![&root];
        while let Some(key) = pending.pop() {
            if seen.insert(key) {
                let node = nodes
                    .get(key)
                    .ok_or("checked graph contains a missing dependency")?;
                pending.extend(node.dependencies.values());
            }
        }
        Self::new(
            key,
            self.crates
                .iter()
                .filter(|node| seen.contains(node.key_value()))
                .cloned()
                .collect(),
        )
    }
}
