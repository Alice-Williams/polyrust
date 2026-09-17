//! Foreign source aliases select original scoped producer witnesses, not storage.
use super::{JavaDependencyBindings, JavaImportedValue};
use portable_codegen::{
    CheckedRustDocumentation, RustCrateExports, RustDeclarationId, RustExportName,
    RustExportNamespace, RustExportTarget,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Default)]
pub(super) struct Selection {
    pub foreign: BTreeMap<(RustDeclarationId, RustExportName), JavaImportedValue>,
}

pub(super) fn collect(
    exports: &Arc<RustCrateExports>,
    dependencies: &JavaDependencyBindings,
) -> Result<Selection, String> {
    CheckedRustDocumentation::check_with_exports(exports, []).map_err(|error| error.to_string())?;
    let mut imports = BTreeMap::new();
    for value in dependencies.values() {
        if !dependencies.contains_value(value) {
            return Err(
                "Java foreign export is absent from its original frozen consumer scope".into(),
            );
        }
        if let Some(prior) = imports.insert(value.constant().declaration(), value)
            && prior != value
        {
            return Err("Java foreign exports have conflicting constant producer witnesses".into());
        }
    }
    let mut selected = Selection::default();
    for (module, bindings) in &exports.modules {
        for (name, target) in bindings {
            let id = match target {
                RustExportTarget::Module(id) | RustExportTarget::Declaration(id) => id,
            };
            if id.crate_id == exports.root.crate_id {
                continue;
            }
            if !matches!(target, RustExportTarget::Declaration(_))
                || name.namespace != RustExportNamespace::Value
            {
                return Err(
                    "Java foreign public export is not a supported scalar constant binding".into(),
                );
            }
            let value = imports.get(id).ok_or(
                "Java foreign constant export has no matching certified imported constant",
            )?;
            selected
                .foreign
                .insert((*module, name.clone()), (*value).clone());
        }
    }
    Ok(selected)
}

/// Structural roots only, not admission: package verification calls collect()
/// and rejects unsupported/unwitnessed metadata before linking checked input.
pub(crate) fn references(
    exports: &RustCrateExports,
    dependencies: &JavaDependencyBindings,
) -> Vec<JavaImportedValue> {
    let selected: BTreeSet<_> = exports
        .modules
        .values()
        .flat_map(|bindings| bindings.iter())
        .filter_map(|(name, target)| match target {
            RustExportTarget::Declaration(id)
                if name.namespace == RustExportNamespace::Value
                    && id.crate_id != exports.root.crate_id =>
            {
                Some(*id)
            }
            _ => None,
        })
        .collect();
    dependencies
        .values()
        .filter(|value| selected.contains(&value.constant().declaration()))
        .cloned()
        .collect()
}
