//! Internal export coherence, not independent proof of compiler provenance.
use portable_codegen::{RustCrateExports, RustExportNamespace, RustExportTarget, RustSourceOrigin};
use std::collections::BTreeSet;

pub(super) fn root(origin: &RustSourceOrigin) -> Result<(), String> {
    if origin.crate_exports.root.crate_id != origin.declaration.crate_id
        || origin
            .module_ancestors
            .first()
            .map(|module| module.declaration)
            != Some(origin.crate_exports.root)
    {
        return Err("compiler export root disagrees with documentation ancestry".into());
    }
    Ok(())
}

pub(super) fn graph(exports: &RustCrateExports) -> Result<(), String> {
    if exports.modules.len() > 100_000 {
        return Err("compiler export module budget exceeded".into());
    }
    let mut pending = vec![exports.root];
    let mut seen = BTreeSet::new();
    let mut bindings = 0usize;
    let mut name_bytes = 0usize;
    while let Some(module) = pending.pop() {
        if !seen.insert(module) {
            continue;
        }
        let entries = exports
            .modules
            .get(&module)
            .ok_or("compiler export graph omits a reachable local module")?;
        for (name, target) in entries {
            bindings = bindings
                .checked_add(1)
                .ok_or("export accounting overflow")?;
            name_bytes = name_bytes
                .checked_add(name.name.len())
                .ok_or("export accounting overflow")?;
            if bindings > 100_000 || name_bytes > 16 * 1024 * 1024 {
                return Err("compiler export binding budget exceeded".into());
            }
            if let RustExportTarget::Module(destination) = target {
                if name.namespace != RustExportNamespace::Type {
                    return Err("compiler module export has the wrong namespace".into());
                }
                if destination.crate_id == exports.root.crate_id {
                    pending.push(*destination);
                }
            }
        }
    }
    if seen.len() != exports.modules.len() {
        return Err("compiler export graph contains unreachable or foreign module entries".into());
    }
    if !exports.module_ancestries.keys().eq(exports.modules.keys()) {
        return Err("compiler export documentation omits or adds a module ancestry".into());
    }
    let mut ancestry_nodes = 0usize;
    for (module, ancestry) in &exports.module_ancestries {
        if !exports.modules.contains_key(module) || ancestry.len() > 128 {
            return Err("export documentation has an unknown or overdeep module owner".into());
        }
        ancestry_nodes = ancestry_nodes
            .checked_add(ancestry.len())
            .ok_or("export ancestry overflow")?;
        if ancestry_nodes > 100_000 {
            return Err("export documentation ancestry budget exceeded".into());
        }
    }
    Ok(())
}
