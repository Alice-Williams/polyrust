//! Bounded declared crate DAG; this is configuration validation, not Rust proof.
#[path = "graph/description.rs"]
mod description;
#[path = "graph/inputs.rs"]
mod inputs;
#[path = "graph/order.rs"]
mod order;
#[path = "graph/parse.rs"]
mod parse;
#[path = "graph/resolved_inputs.rs"]
mod resolved_inputs;
#[path = "graph/subgraph.rs"]
mod subgraph;
use crate::CrateKey;
pub use description::CrateDescription;
pub use inputs::InputMapping;
pub use resolved_inputs::ResolvedInputs;
use std::collections::{BTreeMap, BTreeSet};

const MAX_CRATES: usize = 1024;
const MAX_EDGES: usize = 100_000;
const MAX_INPUTS: usize = 4096;
const MAX_TOTAL_INPUTS: usize = 100_000;
const MAX_PATH_BYTES: usize = 8 * 1024 * 1024;

/// Immutable root-reachable topological graph of explicit build inputs.
/// No caller-provided compiler evidence or target certificate is accepted here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateGraph {
    root: CrateKey,
    crates: Vec<CrateDescription>,
}

impl CrateGraph {
    /// Transport adapters must enforce these bounds before allocating records.
    pub const MAX_ARGUMENTS: usize = parse::MAX_ARGUMENTS;
    pub const MAX_ARGUMENT_BYTES: usize = parse::MAX_ARGUMENT_BYTES;

    /// Parse the closed declared-graph protocol, never arbitrary rustc arguments.
    pub fn parse(arguments: &[String]) -> Result<Self, String> {
        parse::read(arguments)
    }

    pub fn new(root: &str, descriptions: Vec<CrateDescription>) -> Result<Self, String> {
        if descriptions.is_empty() || descriptions.len() > MAX_CRATES {
            return Err("crate graph requires 1..1024 declared crates".into());
        }
        let root = CrateKey::new(root)?;
        let mut crates = BTreeMap::new();
        let mut metadata = BTreeSet::new();
        let mut sources = BTreeSet::new();
        let mut budget = Budget::default();
        for description in descriptions {
            budget.add(&description)?;
            sources.extend(
                description
                    .inputs
                    .values()
                    .map(|input| input.physical.clone()),
            );
            if !metadata.insert(description.metadata.clone()) {
                return Err("crate graph reuses a metadata artifact path".into());
            }
            if crates
                .insert(description.key_value().clone(), description)
                .is_some()
            {
                return Err("duplicate crate graph key".into());
            }
        }
        if !metadata.is_disjoint(&sources) {
            return Err("metadata artifact overlaps a declared source input".into());
        }
        let ordered = order::checked(&root, &crates)?;
        Ok(Self {
            root,
            crates: ordered
                .into_iter()
                .map(|key| crates.remove(&key).unwrap())
                .collect(),
        })
    }

    pub fn root_key(&self) -> &str {
        &self.root.0
    }

    /// Dependencies precede consumers; unrelated ready nodes use stable key order.
    pub fn crates(&self) -> &[CrateDescription] {
        &self.crates
    }
}

#[derive(Default)]
struct Budget {
    edges: usize,
    inputs: usize,
    path_bytes: usize,
}

impl Budget {
    fn add(&mut self, description: &CrateDescription) -> Result<(), String> {
        charge(&mut self.inputs, description.inputs.len(), MAX_TOTAL_INPUTS)?;
        charge(&mut self.edges, description.dependencies.len(), MAX_EDGES)?;
        charge(
            &mut self.path_bytes,
            description.metadata().len(),
            MAX_PATH_BYTES,
        )?;
        for input in description.inputs() {
            charge(&mut self.path_bytes, input.physical().len(), MAX_PATH_BYTES)?;
            charge(&mut self.path_bytes, input.logical().len(), MAX_PATH_BYTES)?;
        }
        Ok(())
    }
}

fn charge(total: &mut usize, amount: usize, limit: usize) -> Result<(), String> {
    *total = total
        .checked_add(amount)
        .ok_or("crate graph budget overflow")?;
    if *total > limit {
        return Err("crate graph budget exceeded".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_budget_boundaries_and_overflow() {
        for limit in [MAX_EDGES, MAX_TOTAL_INPUTS, MAX_PATH_BYTES] {
            let mut value = limit - 1;
            charge(&mut value, 1, limit).unwrap();
            assert!(charge(&mut value, 1, limit).is_err());
        }
        let mut maximum = usize::MAX;
        assert!(charge(&mut maximum, 1, usize::MAX).is_err());
    }
}
