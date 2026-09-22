//! Compiler-to-existing-C-AST bridge; shared by the adapter and compiler probes.
pub(crate) mod assembly;
mod capabilities;
mod constants;
mod control;
mod expressions;
mod functions;
mod initializers;
mod origin;
mod package;
mod types;

use portable_backend_c::ast::*;
use portable_codegen::RustSourceNode;
use rustc_hir::{
    self as hir,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{TyCtxt, TypeckResults};
use std::collections::HashMap;

type Result<T> = std::result::Result<T, String>;

fn c<T>(result: std::result::Result<T, impl std::fmt::Display>) -> Result<T> {
    result.map_err(|error| format!("C typed lowering: {error}"))
}

/// Called only inside after_analysis; the entry mapping runs before body access.
/// Success is evidence for this bridge, not a RenderReadyPackage certificate.
pub fn check(tcx: TyCtxt<'_>, root: LocalDefId) -> Result<(CFrozenRegistry, CSourceFile)> {
    let package = lower(tcx, Selection::Entry(root))?;
    let [source]: [CSourceFile; 1] = package
        .sources
        .try_into()
        .map_err(|_| "expected one selected-entry source")?;
    Ok((package.registry, source))
}

/// Source-selection policy; public API mode has no synthetic entry ABI.
#[derive(Clone, Copy)]
pub enum Selection {
    Entry(LocalDefId),
    PublicApi,
}

/// Typed lowering output, not permission to render before target certification.
pub struct LoweredPackage {
    pub source_types: portable_codegen::RustSourceTypes,
    pub registry: CFrozenRegistry,
    pub sources: Vec<CSourceFile>,
    pub exports: std::sync::Arc<portable_codegen::RustCrateExports>,
    pub constants: std::collections::BTreeMap<
        portable_codegen::RustDeclarationId,
        (CObjectRef, CScalarConstantValue),
    >,
    pub constant_imports: std::collections::BTreeMap<
        portable_codegen::RustDeclarationId,
        (CObjectRef, portable_backend_c::dialect::CDependencyConstant),
    >,
    pub functions: std::collections::BTreeMap<portable_codegen::RustDeclarationId, CFunctionRef>,
    pub imports: std::collections::BTreeMap<
        portable_codegen::RustDeclarationId,
        (
            CFunctionRef,
            portable_backend_c::dialect::CDependencyFunction,
        ),
    >,
}

pub fn lower(tcx: TyCtxt<'_>, selection: Selection) -> Result<LoweredPackage> {
    assembly::lower(tcx, selection, None)
}

/// Internal checked-driver hook; not a public unchecked import API.
pub(crate) struct ForeignLookup<'a> {
    pub function: &'a dyn Fn(DefId) -> Result<portable_backend_c::dialect::CDependencyFunction>,
    pub constant: &'a dyn Fn(DefId) -> Result<portable_backend_c::dialect::CDependencyConstant>,
}

pub(crate) struct Reader<'tcx> {
    tcx: TyCtxt<'tcx>,
    checked: &'tcx TypeckResults<'tcx>,
    mappings: capabilities::CBindings,
    registry: CRegistry,
    file: CFileRef,
    function: CFunctionRef,
    functions: HashMap<LocalDefId, CFunctionRef>,
    foreign_functions: HashMap<DefId, CFunctionRef>,
    parameters: Vec<CParameterRef>,
    root: LocalDefId,
    header: Option<CFileRef>,
    constants: constants::OwnedConstants,
    foreign_constants: constants::ImportedConstants,
    records: HashMap<DefId, CStructRef>,
    declarations: Vec<CFileItem>,
    bindings: HashMap<hir::HirId, CPlace>,
    control_scopes: HashMap<hir::HirId, CScopeRef>,
    origins: origin::Cache,
    next_binding: usize,
    next_scope: usize,
    active_scope: Option<CScopeRef>,
    prelude: Vec<CStatement>,
    next_temporary: usize,
}

impl Reader<'_> {
    fn expressions(&self) -> CExpressions<'_> {
        CExpressions::new(&self.registry)
    }
    fn statements(&self) -> Result<CStatements<'_>> {
        c(CStatements::new(&self.registry, self.function.clone()))
    }
    fn key(&mut self, id: hir::HirId, node: RustSourceNode, name: &str) -> Result<CDeclarationKey> {
        origin::key(
            self.tcx,
            &mut self.origins,
            self.root.to_def_id(),
            node,
            self.tcx.hir_span(id),
            name,
        )
    }
}
