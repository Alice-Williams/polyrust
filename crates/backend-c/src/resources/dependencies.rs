//! Cross-package proof composition; no source parsing or caller-authored costs.
use super::{measure_package, policy};
use crate::ast::{CGeneratedOrigin, CIdentifier, CRegistry};
use crate::dialect::{CDependencyPackage, CDialect};
use portable_codegen::LinkedTargetPackage;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Default)]
struct Budget {
    certificates: usize,
    edges: usize,
}

impl Budget {
    fn certificate(&mut self) -> Result<(), String> {
        self.certificates = self
            .certificates
            .checked_add(1)
            .filter(|count| *count <= 1024)
            .ok_or("C dependency certificate budget exceeded")?;
        Ok(())
    }
    fn edge(&mut self) -> Result<(), String> {
        self.edges = self
            .edges
            .checked_add(1)
            .filter(|count| *count <= 100_000)
            .ok_or("C dependency import-edge budget exceeded")?;
        Ok(())
    }
}

fn registry(package: &LinkedTargetPackage<CDialect>) -> Result<&CRegistry, String> {
    package
        .files()
        .first()
        .and_then(|file| file.items().first())
        .map(|unit| unit.unit.projection.registry.registrations())
        .ok_or_else(|| "C dependency proof requires complete package authority".into())
}

fn enqueue(
    registry: &CRegistry,
    pending: &mut VecDeque<CDependencyPackage>,
    budget: &mut Budget,
) -> Result<(), String> {
    for (function, _) in registry.imported_functions() {
        budget.edge()?;
        pending.push_back(
            registry
                .imported_function(function)
                .map_err(|error| error.to_string())?
                .package_identity(),
        );
    }
    Ok(())
}

pub(super) fn verify(package: &LinkedTargetPackage<CDialect>) -> Result<(), String> {
    let consumer = registry(package)?;
    let mut owned_crates = BTreeSet::new();
    for entry in consumer.inventory() {
        if let CGeneratedOrigin::RustSource(origin) = &entry.key.origin {
            owned_crates.insert(origin.declaration.crate_id);
            owned_crates.insert(origin.crate_exports.root.crate_id);
        }
    }
    let mut owned_names = BTreeSet::new();
    for unit in package.files().iter().flat_map(|file| file.items()) {
        for function in unit.unit.data.bindings.functions.keys() {
            owned_names.insert(
                unit.spelling
                    .functions
                    .get(function)
                    .ok_or("C dependency proof lacks an owned callable spelling")?
                    .clone(),
            );
        }
    }
    let mut budget = Budget::default();
    let mut pending = VecDeque::new();
    enqueue(consumer, &mut pending, &mut budget)?;
    let mut owners = BTreeMap::<u64, CDependencyPackage>::new();
    let mut headers = BTreeMap::new();
    let mut symbols = BTreeSet::<CIdentifier>::new();
    while let Some(owner) = pending.pop_front() {
        let crate_id = owner.root().crate_id;
        if owned_crates.contains(&crate_id) {
            return Err(
                "consumer crate identity appears in its transitive dependency closure".into(),
            );
        }
        if let Some(previous) = owners.get(&crate_id) {
            if previous != &owner {
                return Err(
                    "transitive dependencies contain different certificates for one crate".into(),
                );
            }
            continue;
        }
        budget.certificate()?;
        let header = owner.public_header();
        if headers
            .insert(header.include_path().to_owned(), owner.clone())
            .is_some()
            || consumer
                .files()
                .any(|file| header.conflicts_with_output_path(&file.path))
        {
            return Err("transitive dependency public header names collide".into());
        }
        for name in owner.public_symbols() {
            if owned_names.contains(name) || !symbols.insert(name.clone()) {
                return Err("transitive dependency complete public symbols collide".into());
            }
        }
        let original = owner.certificate().ast();
        // This internal measurement reads child costs, but never recursively
        // calls this verifier. Every child is checked separately by this queue.
        let measured = measure_package(original)?;
        if measured.total.frame_bound != owner.stack_bound_bytes()
            || owner.stack_bound_bytes() == 0
            || !policy::check(
                &measured.total,
                portable_diagnostics::SourceRef::logical(["c", "dependency"]),
            )
            .is_empty()
        {
            return Err("C dependency stack evidence differs from its original certificate".into());
        }
        enqueue(registry(original)?, &mut pending, &mut budget)?;
        owners.insert(crate_id, owner);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Budget;

    #[test]
    fn dependency_traversal_budgets_are_checked_at_the_boundary() {
        let mut budget = Budget {
            certificates: 1023,
            edges: 99_999,
        };
        assert!(budget.certificate().is_ok());
        assert!(budget.certificate().is_err());
        assert!(budget.edge().is_ok());
        assert!(budget.edge().is_err());
        let mut overflow = Budget {
            certificates: usize::MAX,
            edges: usize::MAX,
        };
        assert!(overflow.certificate().is_err());
        assert!(overflow.edge().is_err());
    }
}
