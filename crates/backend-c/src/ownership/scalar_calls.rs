//! Private effect evidence derived from definitions, never a registered pure flag.
//! This is not a numeric, termination, stack or rendering certificate.
mod shape;

#[cfg(test)]
#[path = "../tests/scalar_call_boundaries.rs"]
mod boundary_tests;
#[cfg(test)]
#[path = "../tests/scalar_call_fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/scalar_call_graphs.rs"]
mod graph_tests;
#[cfg(test)]
#[path = "../tests/scalar_call_local_storage.rs"]
mod local_storage_tests;
#[cfg(test)]
#[path = "../tests/scalar_call_effects.rs"]
mod tests;

use crate::ast::{CCallableKind, CDefinitionKind, CFileItem, CFunctionRef, CSourceFile};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Default)]
pub(super) struct ScalarCalls {
    closed: BTreeSet<CFunctionRef>,
}

impl ScalarCalls {
    pub(super) fn into_functions(self) -> BTreeSet<CFunctionRef> {
        self.closed
    }

    #[cfg(test)]
    pub(super) fn derive(files: &[CSourceFile]) -> Self {
        Self::with_seeds(files, BTreeSet::new())
    }

    pub(super) fn derive_registered(
        registry: &crate::ast::CRegistry,
        files: &[CSourceFile],
    ) -> Result<Self, crate::ast::CRegistryError> {
        let mut seeds = BTreeSet::new();
        for (function, _) in registry.imported_functions() {
            // Public i32/bool witnesses were issued only after the owning
            // package's actual closed scalar-call proof and certification.
            registry.imported_function(function)?;
            seeds.insert(function.clone());
        }
        Ok(Self::with_seeds(files, seeds))
    }

    fn with_seeds(files: &[CSourceFile], seeds: BTreeSet<CFunctionRef>) -> Self {
        let mut candidates = BTreeMap::new();
        for file in files {
            for item in file.items() {
                if let CFileItem::Definition(definition) = item
                    && let CDefinitionKind::Function { function, body, .. } = definition.kind()
                    && let Some(edges) = shape::dependencies(function, body)
                {
                    candidates.insert(function.clone(), edges);
                }
            }
        }
        // Context reconstruction already rejects duplicate definitions. Keep
        // unresolved targets in the counts: they must never become leaf proofs.
        let mut remaining = BTreeMap::new();
        let mut callers: BTreeMap<_, Vec<_>> = BTreeMap::new();
        let mut pending: VecDeque<_> = seeds.into_iter().collect();
        for (function, edges) in candidates {
            remaining.insert(function.clone(), edges.len());
            if edges.is_empty() {
                pending.push_back(function.clone());
            }
            for callee in edges {
                callers.entry(callee).or_default().push(function.clone());
            }
        }
        let mut result = Self::default();
        while let Some(function) = pending.pop_front() {
            if !result.closed.insert(function.clone()) {
                continue;
            }
            for caller in callers.get(&function).into_iter().flatten() {
                let count = remaining.get_mut(caller).expect("candidate caller");
                *count -= 1;
                if *count == 0 {
                    pending.push_back(caller.clone());
                }
            }
        }
        // Cycles, effectful callees and their transitive callers never enter
        // closed. No recursive host traversal or optimistic fixed point.
        result
    }

    pub(super) fn accepts(&self, callable: &crate::ast::CCallable) -> bool {
        match callable.kind() {
            CCallableKind::Direct(function) => self.closed.contains(function.as_ref()),
            CCallableKind::Indirect { .. } | CCallableKind::Known(_) => false,
        }
    }
}
