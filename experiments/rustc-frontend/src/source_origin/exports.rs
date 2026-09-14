//! Shared reader for rustc's resolved public binding graph, not textual uses.
mod budget;
use super::{Result, identity};
use portable_codegen::{RustCrateExports, RustExportName, RustExportNamespace, RustExportTarget};
use rustc_hir::{
    def::{DefKind, Namespace, Res},
    def_id::CRATE_DEF_ID,
};
use rustc_middle::ty::{TyCtxt, Visibility};
use std::collections::{BTreeMap, HashSet};

pub(super) fn collect(tcx: TyCtxt<'_>, cache: &mut super::Cache) -> Result<RustCrateExports> {
    let mut result = RustCrateExports {
        root: identity(tcx, CRATE_DEF_ID.to_def_id()),
        modules: BTreeMap::new(),
        module_ancestries: BTreeMap::new(),
    };
    let mut pending = vec![CRATE_DEF_ID];
    let mut seen = HashSet::from([CRATE_DEF_ID]);
    let mut budget = budget::Budget::default();
    while let Some(module) = pending.pop() {
        result.module_ancestries.insert(
            identity(tcx, module.to_def_id()),
            cache.ancestry(tcx, module.to_def_id())?,
        );
        let mut bindings = BTreeMap::new();
        for child in tcx.module_children_local(module) {
            budget.binding()?;
            if child.vis != Visibility::Public {
                continue;
            }
            let Res::Def(kind, target) = child.res else {
                return Err("source public binding lacks a supported declaration identity".into());
            };
            let namespace = match child.res.ns() {
                Some(Namespace::TypeNS) => RustExportNamespace::Type,
                Some(Namespace::ValueNS) => RustExportNamespace::Value,
                Some(Namespace::MacroNS) => RustExportNamespace::Macro,
                None => return Err("source public binding lacks a resolved namespace".into()),
            };
            let spelling = child.ident.name.as_str();
            budget.name(spelling.len())?;
            let name = RustExportName {
                namespace,
                name: spelling.to_owned(),
            };
            let declaration = identity(tcx, target);
            let target = if kind == DefKind::Mod {
                // Foreign modules remain explicit dependency edges. Do not
                // traverse them and silently merge a dependency into this crate.
                if let Some(local) = target.as_local()
                    && seen.insert(local)
                {
                    pending.push(local);
                }
                RustExportTarget::Module(declaration)
            } else {
                RustExportTarget::Declaration(declaration)
            };
            if bindings.insert(name, target).is_some() {
                return Err("source public binding has an ambiguous namespace/name".into());
            }
        }
        result
            .modules
            .insert(identity(tcx, module.to_def_id()), bindings);
    }
    Ok(result)
}
