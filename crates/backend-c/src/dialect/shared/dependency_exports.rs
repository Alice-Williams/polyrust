//! A dependency header/object exposes all its exports, not only selected calls.
use super::{CDependencyPackage, CResolvedUnit, violation};
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
    // These are actual allocated spellings, not guesses from requested names.
    // Imported references are excluded from the owned declaration map.
    for function in unit.unit.data.bindings.functions.keys() {
        let name = unit
            .spelling
            .functions
            .get(function)
            .ok_or_else(|| violation("owned C function lacks its resolved spelling"))?;
        if names.contains_key(name) {
            return Err(violation(
                "owned C binding collides with a complete dependency export inventory",
            ));
        }
    }
    Ok(())
}
