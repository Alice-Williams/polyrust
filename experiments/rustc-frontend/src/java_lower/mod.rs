//! Compiler-to-existing-Java-AST bridge; no target syntax is assembled here.
mod assembly;
#[cfg(java_ast_probe)]
#[path = "../../test/java_source_assertions.rs"]
mod assertions;
mod capabilities;
mod constants;
#[cfg(java_ast_probe)]
#[path = "../../test/java_expression_assertions.rs"]
mod expression_assertions;
mod expressions;
mod foreign;
mod functions;
mod package;
mod records;
mod representation;

use portable_backend_java::{
    ast::*,
    dialect::{JavaDependencyFunction, JavaDialect, JavaImportedCallable},
};
use portable_codegen::{GeneratedCallableId, TargetAstBuilder, TargetAstPackage};
use rustc_hir::{
    self as hir,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{TyCtxt, TypeckResults};
use std::collections::HashMap;

use representation::{Place, TypePlan, Value};
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy)]
pub enum Selection {
    Entry(LocalDefId),
    PublicApi,
}

pub(crate) struct DependencyLookup<'a> {
    pub function: &'a dyn Fn(DefId) -> Result<JavaDependencyFunction>,
    pub constant:
        &'a dyn Fn(DefId) -> Result<portable_backend_java::dialect::JavaDependencyConstant>,
}

/// Called only after successful compiler analysis and declared-input checking.
/// This is still an unresolved target package, never permission to render.
pub fn lower(
    tcx: TyCtxt<'_>,
    selection: Selection,
    dependencies: Option<&DependencyLookup<'_>>,
) -> Result<TargetAstPackage<JavaDialect>> {
    assembly::lower(tcx, selection, dependencies)
}

#[derive(Clone)]
#[cfg_attr(local_constant_ast_probe, derive(Debug))]
struct Callable {
    id: GeneratedCallableId,
    name: JavaIdentifier,
    signature: JavaMethodSignature,
    visibility: JavaVisibility,
}

pub(crate) struct Reader<'tcx> {
    tcx: TyCtxt<'tcx>,
    checked: &'tcx TypeckResults<'tcx>,
    mappings: capabilities::JavaBindings,
    builder: TargetAstBuilder<JavaDialect>,
    functions: HashMap<LocalDefId, Callable>,
    imported: HashMap<DefId, JavaImportedCallable>,
    public_api: bool,
    constants: HashMap<DefId, constants::Constant>,
    foreign_constants: HashMap<DefId, portable_backend_java::dialect::JavaImportedValue>,
    records: HashMap<DefId, records::Record>,
    bindings: HashMap<hir::HirId, Place>,
    origins: crate::source_origin::Cache,
    prelude: Vec<JavaStmt>,
    next_local: usize,
    active_scope: Option<hir::HirId>,
    scopes: HashMap<hir::HirId, Option<hir::HirId>>,
    depth: usize,
    remaining: usize,
    #[cfg(java_ast_probe)]
    expression_observations: Vec<(hir::HirId, usize, Value)>,
}

fn name(value: &str) -> Result<JavaIdentifier> {
    JavaIdentifier::new(value).map_err(|error| format!("Java identifier: {error:?}"))
}

fn source() -> portable_diagnostics::SourceRef {
    portable_diagnostics::SourceRef::logical(["rustc-java-source"])
}
