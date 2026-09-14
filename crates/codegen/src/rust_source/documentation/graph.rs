use super::{Budget, Result, RustDocumentationError};
use crate::{RustCrateExports, RustDeclarationId, RustExportNamespace, RustExportTarget};
use std::collections::BTreeSet;

// Module payloads were independently joined against the canonical module table.
// Compare the graph and ancestry identities, not recursively repeated doc text.
pub(super) fn same_inventory(left: &RustCrateExports, right: &RustCrateExports) -> bool {
    left.root == right.root
        && left.modules == right.modules
        && left.module_ancestries.len() == right.module_ancestries.len()
        && left
            .module_ancestries
            .iter()
            .zip(&right.module_ancestries)
            .all(|((left_id, left_path), (right_id, right_path))| {
                left_id == right_id
                    && left_path
                        .iter()
                        .map(|module| module.declaration)
                        .eq(right_path.iter().map(|module| module.declaration))
            })
}

pub(super) fn verify(
    exports: &RustCrateExports,
    budget: &mut Budget,
) -> Result<BTreeSet<RustDeclarationId>> {
    budget.modules(exports.modules.len())?;
    if !exports.module_ancestries.keys().eq(exports.modules.keys()) {
        return Err(RustDocumentationError::Structure(
            "export ancestry keys differ from modules",
        ));
    }
    let mut pending = vec![exports.root];
    let mut seen = BTreeSet::new();
    let mut modules = BTreeSet::from([exports.root]);
    let mut declarations = BTreeSet::new();
    while let Some(module) = pending.pop() {
        if !seen.insert(module) {
            continue;
        }
        let entries = exports
            .modules
            .get(&module)
            .ok_or(RustDocumentationError::Structure(
                "export graph omits a reachable local module",
            ))?;
        for (name, target) in entries {
            budget.binding(&name.name)?;
            if let RustExportTarget::Module(destination) = target {
                modules.insert(*destination);
                if name.namespace != RustExportNamespace::Type {
                    return Err(RustDocumentationError::Structure(
                        "module export has the wrong namespace",
                    ));
                }
                if destination.crate_id == exports.root.crate_id {
                    pending.push(*destination);
                }
            } else if let RustExportTarget::Declaration(destination) = target {
                declarations.insert(*destination);
            }
        }
    }
    if seen.len() != exports.modules.len() {
        return Err(RustDocumentationError::Structure(
            "unreachable or foreign module entry",
        ));
    }
    if !modules.is_disjoint(&declarations) {
        return Err(RustDocumentationError::Structure(
            "export identity is both module and declaration",
        ));
    }
    Ok(declarations)
}
