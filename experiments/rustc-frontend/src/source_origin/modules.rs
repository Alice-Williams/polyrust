//! Cache module payloads and ancestry; bound raw text before cloning attributes.
use super::{Result, identity, location};
use portable_codegen::{RustCrateExports, RustModuleAncestry, RustModuleDocumentation};
use rustc_hir::{def::DefKind, def_id::DefId};
use rustc_middle::ty::TyCtxt;
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub(crate) struct Cache {
    exports: Option<Arc<RustCrateExports>>,
    modules: HashMap<DefId, Arc<RustModuleDocumentation>>,
    ancestries: HashMap<DefId, RustModuleAncestry>,
    raw_bytes: usize,
    attributes: usize,
}

impl Cache {
    pub(crate) fn exports(&mut self, tcx: TyCtxt<'_>) -> Result<Arc<RustCrateExports>> {
        if let Some(exports) = &self.exports {
            return Ok(exports.clone());
        }
        let exports = Arc::new(super::exports::collect(tcx, self)?);
        self.exports = Some(exports.clone());
        Ok(exports)
    }

    pub(super) fn documentation(&mut self, tcx: TyCtxt<'_>, id: DefId) -> Result<Vec<String>> {
        let attributes = if let Some(local) = id.as_local() {
            tcx.hir_attrs(tcx.local_def_id_to_hir_id(local))
        } else {
            tcx.attrs_for_def(id)
        };
        let mut text = Vec::new();
        for attribute in attributes {
            let Some(value) = attribute.doc_str() else {
                continue;
            };
            let value = value.as_str();
            self.raw_bytes = self
                .raw_bytes
                .checked_add(value.len())
                .ok_or("source documentation byte accounting overflow")?;
            self.attributes = self
                .attributes
                .checked_add(1)
                .ok_or("source documentation attribute accounting overflow")?;
            if self.raw_bytes > 16 * 1024 * 1024 || self.attributes > 100_000 {
                return Err("source documentation extraction budget exceeded".into());
            }
            text.push(value.to_owned());
        }
        Ok(text)
    }

    pub(super) fn ancestry(&mut self, tcx: TyCtxt<'_>, owner: DefId) -> Result<RustModuleAncestry> {
        if let Some(ancestry) = self.ancestries.get(&owner) {
            return Ok(ancestry.clone());
        }
        let mut module = owner;
        let mut result = Vec::new();
        loop {
            if result.len() >= 128 || self.modules.len() >= 100_000 {
                return Err("source module extraction budget exceeded".into());
            }
            if tcx.def_kind(module) != DefKind::Mod {
                return Err("source module ancestry includes a non-module owner".into());
            }
            let parent = tcx.opt_parent(module);
            let metadata = if let Some(metadata) = self.modules.get(&module) {
                metadata.clone()
            } else {
                let metadata = Arc::new(RustModuleDocumentation {
                    declaration: identity(tcx, module),
                    parent: parent.map(|value| identity(tcx, value)),
                    location: location(tcx, tcx.def_span(module)),
                    documentation: self.documentation(tcx, module)?,
                });
                self.modules.insert(module, metadata.clone());
                metadata
            };
            result.push(metadata);
            let Some(parent) = parent else { break };
            module = parent;
        }
        result.reverse();
        let result: RustModuleAncestry = result.into();
        self.ancestries.insert(owner, result.clone());
        Ok(result)
    }
}
