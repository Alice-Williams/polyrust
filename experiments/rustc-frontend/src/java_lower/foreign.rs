//! Join compiler declarations/signatures before creating consumer references.
use super::{DependencyLookup, Result, capabilities};
use capabilities::{FunctionInput, FunctionSignatures, Mapping, Supports};
use portable_backend_java::dialect::{
    JavaDependencyBindings, JavaDependencyScope, JavaImportedCallable,
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

pub(super) struct Registered {
    pub(super) functions: HashMap<DefId, JavaImportedCallable>,
    pub(super) bindings: JavaDependencyBindings,
}

pub(super) fn register(
    tcx: TyCtxt<'_>,
    definitions: &[DefId],
    mappings: &capabilities::JavaBindings,
    lookup: Option<&DependencyLookup<'_>>,
) -> Result<Registered> {
    let mut scope = JavaDependencyScope::new();
    let mut functions = HashMap::new();
    for definition in definitions {
        let lookup = lookup
            .ok_or("foreign Java direct calls require an authenticated dependency certificate")?;
        let function = lookup(*definition)?;
        let expected = Supports::<FunctionSignatures>::mapping(mappings).lower(
            &mut (),
            FunctionInput {
                tcx,
                function: *definition,
            },
        )?;
        #[cfg(java_graph_wrong_signature)]
        let expected = crate::java_graph::mutations::signature(expected);
        if function.declaration() != crate::source_origin::identity(tcx, *definition)
            || function.signature() != &expected
        {
            return Err("foreign compiler identity/signature differs from Java certificate".into());
        }
        let (next, imported) = scope.import(function);
        scope = next;
        if functions.insert(*definition, imported).is_some() {
            return Err("duplicate Java foreign declaration registration".into());
        }
    }
    Ok(Registered {
        functions,
        bindings: scope.finish(),
    })
}
