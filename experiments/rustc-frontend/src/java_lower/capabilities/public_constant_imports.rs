//! Authenticate a foreign scalar before admitting a dependency-scoped value.
use super::{ConstantImportInput, Mapping, PublicConstantImports};
use crate::java_lower::{Result, constants};
use portable_backend_java::dialect::{
    JavaDependencyConstant, JavaDependencyScope, JavaImportedValue,
};

pub(crate) struct ImportState {
    pub scope: JavaDependencyScope,
    pub proof: JavaDependencyConstant,
}
#[derive(Clone, Copy)]
pub(crate) struct JavaPublicConstantImports;
impl Mapping for JavaPublicConstantImports {
    type Capability = PublicConstantImports;
    type Context<'tcx> = ImportState;
    type Output = JavaImportedValue;
    fn lower<'tcx>(
        &self,
        state: &mut ImportState,
        input: ConstantImportInput<'tcx>,
    ) -> Result<JavaImportedValue> {
        let value = input.value();
        #[cfg(any(constant_import_wrong_type, constant_import_wrong_value))]
        let value = crate::source_capabilities::import_mutations::value(value);
        let (plan, expected) = constants::value(value);
        let id = crate::source_origin::identity(input.tcx(), input.definition());
        if state.proof.source_value() != Some(value.original()) {
            return Err(
                "foreign compiler identity/type/value differs: original Rust value differs from Java source constant facts".into(),
            );
        }
        if state.proof.declaration() != id
            || state.proof.package_identity().root().crate_id != id.crate_id
            || state.proof.ty() != &plan.java_type()
            || state.proof.value() != &expected
        {
            return Err(
                "foreign compiler identity/type/value differs from Java constant certificate"
                    .into(),
            );
        }
        let scope = std::mem::replace(&mut state.scope, JavaDependencyScope::new());
        let (scope, value) = scope
            .import_constant(state.proof.clone())
            .map_err(|errors| format!("{errors:?}"))?;
        state.scope = scope;
        Ok(value)
    }
}
