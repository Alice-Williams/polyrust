//! Foreign public bindings select original producer evidence, never new storage.
use super::CDependencyConstant;
use crate::ast::{CObjectRef, CRegistry};
use portable_codegen::{RustDeclarationId, RustExportName, RustExportNamespace, RustExportTarget};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct ForeignBinding {
    pub object: CObjectRef,
    pub dependency: CDependencyConstant,
}

#[derive(Default)]
pub(super) struct Selection {
    pub owned: BTreeSet<RustDeclarationId>,
    pub foreign: BTreeMap<(RustDeclarationId, RustExportName), ForeignBinding>,
}

pub(super) fn collect(registry: &CRegistry) -> Result<Selection, String> {
    let Some(source) = registry.source_package() else {
        return Ok(Selection::default());
    };
    let exports = source.exports();
    super::documentation::check_export_graph(exports)?;
    let mut imports = BTreeMap::new();
    for (object, _) in registry.imported_constants() {
        let proof = registry
            .imported_constant(object)
            .map_err(|error| error.to_string())?;
        if imports
            .insert(proof.declaration(), (object, proof))
            .is_some()
        {
            return Err("C foreign exports have conflicting imported constant identities".into());
        }
    }
    let mut selected = Selection::default();
    for (module, bindings) in &exports.modules {
        for (name, target) in bindings {
            match target {
                RustExportTarget::Module(id)
                    if name.namespace == RustExportNamespace::Type
                        && id.crate_id == exports.root.crate_id
                        && exports.modules.contains_key(id) => {}
                RustExportTarget::Declaration(id)
                    if name.namespace == RustExportNamespace::Value =>
                {
                    if id.crate_id == exports.root.crate_id {
                        selected.owned.insert(*id);
                    } else {
                        let (object, proof) = imports.get(id).ok_or(
                            "C foreign constant export has no matching certified imported constant",
                        )?;
                        selected.foreign.insert(
                            (*module, name.clone()),
                            ForeignBinding {
                                object: (*object).clone(),
                                dependency: (*proof).clone(),
                            },
                        );
                    }
                }
                _ => {
                    return Err(
                        "C public export is not a local module/value or certified foreign constant"
                            .into(),
                    );
                }
            }
        }
    }
    Ok(selected)
}
