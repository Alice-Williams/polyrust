//! Join compiler declarations/signatures before creating consumer references.
use super::{DependencyLookup, Result, capabilities};
use capabilities::{FunctionInput, FunctionSignatures, Mapping, Supports};
use portable_backend_java::dialect::{
    JavaDependencyBindings, JavaDependencyScope, JavaImportedCallable, JavaImportedValue,
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

pub(super) struct Registered {
    pub(super) functions: HashMap<DefId, JavaImportedCallable>,
    pub(super) bindings: JavaDependencyBindings,
    pub(super) constants: HashMap<DefId, JavaImportedValue>,
}

pub(super) fn register(
    tcx: TyCtxt<'_>,
    definitions: &[DefId],
    constant_definitions: &[DefId],
    mappings: &capabilities::JavaBindings,
    lookup: Option<&DependencyLookup<'_>>,
) -> Result<Registered> {
    let mut scope = JavaDependencyScope::new();
    let mut functions = HashMap::new();
    for definition in definitions {
        let lookup = lookup
            .ok_or("foreign Java direct calls require an authenticated dependency certificate")?;
        let function = (lookup.function)(*definition)?;
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
            || function.declaration_signature() != &expected
        {
            return Err("foreign compiler identity/signature differs from Java certificate".into());
        }
        let original = crate::source_origin::types::signature(tcx, *definition)?;
        if function.source_signature() != Some(&original) {
            return Err(
                "foreign original Rust signature differs from Java source type facts".into(),
            );
        }
        let (next, imported) = scope
            .import(function)
            .map_err(|errors| format!("{errors:?}"))?;
        scope = next;
        if functions.insert(*definition, imported).is_some() {
            return Err("duplicate Java foreign declaration registration".into());
        }
    }
    let mut constants = HashMap::new();
    for definition in constant_definitions {
        let lookup =
            lookup.ok_or("foreign public constant reads require a certified producer mapping")?;
        let proof = (lookup.constant)(*definition)?;
        let input = capabilities::ConstantImportInput::read(tcx, *definition)?;
        let mut registration = capabilities::ImportState { scope, proof };
        let value = Supports::<capabilities::PublicConstantImports>::mapping(mappings)
            .lower(&mut registration, input)?;
        scope = registration.scope;
        if constants.insert(*definition, value).is_some() {
            return Err("duplicate Java foreign constant registration".into());
        }
    }
    Ok(Registered {
        constants,
        functions,
        bindings: scope.finish(),
    })
}
