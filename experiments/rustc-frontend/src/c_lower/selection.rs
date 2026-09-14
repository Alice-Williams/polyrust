//! Public roots come from resolved compiler exports, not names or source text.
use super::{Result, origin};
use portable_codegen::{RustCrateExports, RustExportNamespace, RustExportTarget};
use rustc_hir::{def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn public_roots(tcx: TyCtxt<'_>, exports: &RustCrateExports) -> Result<Vec<LocalDefId>> {
    let mut functions = BTreeMap::new();
    for function in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let id = origin::identity(tcx, function.to_def_id());
        if functions.insert(id, function).is_some() {
            return Err("ambiguous stable compiler function identity".into());
        }
    }
    let mut roots = BTreeSet::new();
    for bindings in exports.modules.values() {
        for (name, target) in bindings {
            match target {
                RustExportTarget::Module(module) => {
                    if name.namespace != RustExportNamespace::Type
                        || module.crate_id != exports.root.crate_id
                        || !exports.modules.contains_key(module)
                    {
                        return Err("public package foreign or unsupported module binding".into());
                    }
                }
                RustExportTarget::Declaration(id) => {
                    if name.namespace != RustExportNamespace::Value
                        || id.crate_id != exports.root.crate_id
                        || !functions.contains_key(id)
                    {
                        return Err(format!(
                            "public package API mapping is not implemented for {:?} {}",
                            name.namespace, name.name
                        ));
                    }
                    if !tcx.effective_visibilities(()).is_exported(functions[id]) {
                        return Err("compiler public binding is not externally reachable".into());
                    }
                    roots.insert(*id);
                }
            }
        }
    }
    if roots.is_empty() {
        return Err("public package requires an exported scalar function".into());
    }
    // Stable source identity orders roots; ephemeral rustc allocation does not.
    Ok(roots.into_iter().map(|id| functions[&id]).collect())
}
