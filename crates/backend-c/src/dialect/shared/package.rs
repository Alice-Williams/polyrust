//! Whole-package authority is distinct from each file's derived projection.
use super::{CStdType, bindings::CBindings, documentation::CDocumentation};
use crate::ast::{CFrozenRegistry, CSourceFile};
use std::{collections::BTreeSet, sync::Arc};

#[cfg(test)]
#[path = "../../tests/shared_package_authority.rs"]
mod tests;

/// One immutable authority shared by every unit in a package. Source trees
/// retain their existing CFileRef identities; no parallel file AST is created.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CProjection {
    pub registry: CFrozenRegistry,
    pub sources: Vec<Arc<CSourceFile>>,
    pub bindings: CBindings,
}

/// Derived per-file data, independently reconstructed by package verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CUnitData {
    pub source: Arc<CSourceFile>,
    pub bindings: CBindings,
    pub declarations: Vec<portable_codegen::GeneratedSymbolId>,
    pub standards: BTreeSet<CStdType>,
    pub documentation: CDocumentation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CProjectedUnit {
    pub(super) projection: Arc<CProjection>,
    pub(super) data: Arc<CUnitData>,
}
