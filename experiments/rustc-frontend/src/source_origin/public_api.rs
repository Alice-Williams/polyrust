//! Closed compiler declaration inventory; target capability admission is separate.
mod foreign_constants;
use super::{Cache, Result, identity};
pub(crate) use foreign_constants::ForeignConstantDeclaration;
use foreign_constants::Policy;
use portable_codegen::{
    RustCrateExports, RustDeclarationId, RustExportNamespace, RustExportTarget,
};
use rustc_hir::{def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;
use std::{collections::BTreeMap, sync::Arc};

/// A source kind, not a claim that a backend implements its type or initializer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclarationKind {
    Function,
    Constant,
}

/// Constructed only while resolving the checked compiler's actual export graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Declaration {
    definition: LocalDefId,
    kind: DeclarationKind,
}

impl Declaration {
    pub(crate) fn definition(self) -> LocalDefId {
        self.definition
    }

    pub(crate) fn kind(self) -> DeclarationKind {
        self.kind
    }
}

/// No caller-supplied metadata or target symbol can construct this inventory.
pub(crate) struct Inventory {
    exports: Arc<RustCrateExports>,
    declarations: BTreeMap<RustDeclarationId, Declaration>,
    foreign_constants: BTreeMap<RustDeclarationId, ForeignConstantDeclaration>,
}

impl Inventory {
    pub(crate) fn read(tcx: TyCtxt<'_>, cache: &mut Cache) -> Result<Self> {
        Self::read_policy(tcx, cache, Policy::OwnedOnly)
    }

    /// Extended classification only; production lowering remains on read().
    pub(crate) fn read_with_constant_reexports(tcx: TyCtxt<'_>, cache: &mut Cache) -> Result<Self> {
        Self::read_policy(tcx, cache, Policy::ForeignConstants)
    }

    fn read_policy(tcx: TyCtxt<'_>, cache: &mut Cache, policy: Policy) -> Result<Self> {
        let exports = cache.exports(tcx)?;
        let mut definitions = BTreeMap::new();
        for (index, definition) in tcx.hir_body_owners().enumerate() {
            if index >= 100_000 {
                return Err("public declaration inventory scan budget exceeded".into());
            }
            let kind = match tcx.def_kind(definition) {
                DefKind::Fn => DeclarationKind::Function,
                DefKind::Const {
                    is_type_const: false,
                } if tcx.def_kind(tcx.parent(definition.to_def_id())) == DefKind::Mod => {
                    DeclarationKind::Constant
                }
                _ => continue,
            };
            let declaration = Declaration { definition, kind };
            if definitions
                .insert(identity(tcx, definition.to_def_id()), declaration)
                .is_some()
            {
                return Err("ambiguous stable compiler declaration identity".into());
            }
        }
        let mut declarations = BTreeMap::new();
        let mut foreign_constants = BTreeMap::new();
        for bindings in exports.modules.values() {
            for (name, target) in bindings {
                match target {
                    RustExportTarget::Module(module) => {
                        if name.namespace != RustExportNamespace::Type
                            || module.crate_id != exports.root.crate_id
                            || !exports.modules.contains_key(module)
                        {
                            return Err(
                                "public package foreign or unsupported module binding".into()
                            );
                        }
                    }
                    RustExportTarget::Declaration(id)
                        if id.crate_id != exports.root.crate_id
                            && name.namespace == RustExportNamespace::Value
                            && matches!(policy, Policy::ForeignConstants) =>
                    {
                        let definition = cache
                            .export_definition(tcx, *id)?
                            .ok_or("foreign binding lacks its checked compiler definition")?;
                        if identity(tcx, definition) != *id {
                            return Err("foreign binding compiler identity disagrees".into());
                        }
                        foreign_constants
                            .insert(*id, ForeignConstantDeclaration::read(tcx, definition)?);
                    }
                    RustExportTarget::Declaration(id) => {
                        let declaration = definitions
                            .get(id)
                            .filter(|_| {
                                name.namespace == RustExportNamespace::Value
                                    && id.crate_id == exports.root.crate_id
                            })
                            .ok_or_else(|| {
                                format!(
                                    "public package API mapping is not implemented for {:?} {}",
                                    name.namespace, name.name
                                )
                            })?;
                        if cache.export_definition(tcx, *id)?
                            != Some(declaration.definition.to_def_id())
                        {
                            return Err(
                                "local binding differs from its checked compiler definition".into(),
                            );
                        }
                        if !tcx
                            .effective_visibilities(())
                            .is_exported(declaration.definition)
                        {
                            return Err(
                                "compiler public binding is not externally reachable".into()
                            );
                        }
                        declarations.insert(*id, *declaration);
                    }
                }
                if declarations.len() + foreign_constants.len() > 4096 {
                    return Err("public declaration inventory size budget exceeded".into());
                }
            }
        }
        if declarations.is_empty() && foreign_constants.is_empty() {
            return Err("public package requires an exported function or constant".into());
        }
        Ok(Self {
            exports,
            declarations,
            foreign_constants,
        })
    }

    pub(crate) fn exports(&self) -> &Arc<RustCrateExports> {
        &self.exports
    }

    pub(crate) fn declarations(&self) -> &BTreeMap<RustDeclarationId, Declaration> {
        &self.declarations
    }

    pub(crate) fn foreign_constants(
        &self,
    ) -> &BTreeMap<RustDeclarationId, ForeignConstantDeclaration> {
        &self.foreign_constants
    }

    /// Transitional production policy: inventory classification does not enable output.
    pub(crate) fn function_roots(&self) -> Result<Vec<LocalDefId>> {
        if !self.foreign_constants.is_empty() {
            return Err("function-only selection does not accept foreign constant exports".into());
        }
        self.declarations
            .values()
            .map(|declaration| match declaration.kind {
                DeclarationKind::Function => Ok(declaration.definition),
                DeclarationKind::Constant => {
                    Err("public package API mapping is not implemented for constant exports".into())
                }
            })
            .collect()
    }
}
