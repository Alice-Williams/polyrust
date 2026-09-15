//! A dependency header/object exposes all its exports, not only selected calls.
use super::{CDependencyPackage, CResolvedUnit, bindings::CValueBinding, violation};
use crate::ast::{CIdentifier, CRegistry};
use portable_codegen::AstViolation;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn inventory(
    registry: &CRegistry,
) -> Result<BTreeMap<CIdentifier, CDependencyPackage>, AstViolation> {
    let mut owners = BTreeSet::new();
    let mut names = BTreeMap::new();
    for (function, _) in registry.imported_functions() {
        let dependency = registry
            .imported_function(function)
            .map_err(|error| violation(error.to_string()))?;
        let owner = dependency.package_identity();
        if !owners.insert(owner.clone()) {
            continue;
        }
        for name in owner.public_symbols() {
            if names.insert(name.clone(), owner.clone()).is_some() {
                return Err(violation("complete dependency export inventories collide"));
            }
        }
    }
    Ok(names)
}

pub(super) fn verify_unit(unit: &CResolvedUnit) -> Result<(), AstViolation> {
    let names = inventory(unit.unit.projection.registry.registrations())?;
    for name in owned_names(unit)? {
        if names.contains_key(&name) {
            return Err(violation(
                "owned C binding collides with a complete dependency export inventory",
            ));
        }
    }
    Ok(())
}

/// Ordinary file-scope identifiers participate in both header and link-time
/// collision checks. Block-local and member namespaces do not.
pub(super) fn owned_names(unit: &CResolvedUnit) -> Result<BTreeSet<CIdentifier>, AstViolation> {
    let mut names = BTreeSet::new();
    // These are actual allocated spellings, not guesses from requested names.
    // Imported references are excluded from the owned declaration map.
    for function in unit.unit.data.bindings.functions.keys() {
        let name = unit
            .spelling
            .functions
            .get(function)
            .ok_or_else(|| violation("owned C function lacks its resolved spelling"))?;
        names.insert(name.clone());
    }
    for value in unit.unit.data.bindings.values.keys() {
        if matches!(value, CValueBinding::Global(_)) {
            let name = unit
                .spelling
                .values
                .get(value)
                .ok_or_else(|| violation("owned C global lacks its resolved spelling"))?;
            names.insert(name.clone());
        }
    }
    Ok(names)
}
