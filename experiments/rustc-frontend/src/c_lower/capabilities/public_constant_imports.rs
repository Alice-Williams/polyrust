//! Join a checked compiler declaration to its original producer certificate.
use super::{ConstantImportInput, Mapping, PublicConstantImports};
use crate::c_lower::{Result, c, constants};
use portable_backend_c::{ast::*, dialect::CDependencyConstant};

/// Registration owns the registry; no borrowed body reader or metadata authority.
pub(crate) struct ImportState {
    pub registry: CRegistry,
    pub proof: CDependencyConstant,
}
#[derive(Clone, Copy)]
pub(crate) struct CPublicConstantImports;
impl Mapping for CPublicConstantImports {
    type Capability = PublicConstantImports;
    type Context<'tcx> = ImportState;
    type Output = CObjectRef;
    fn lower<'tcx>(
        &self,
        state: &mut ImportState,
        input: ConstantImportInput<'tcx>,
    ) -> Result<CObjectRef> {
        let value = input.value();
        #[cfg(any(constant_import_wrong_type, constant_import_wrong_value))]
        let value = crate::source_capabilities::import_mutations::value(value);
        let expected = constants::value(value);
        let id = crate::source_origin::identity(input.tcx(), input.definition());
        if state.proof.declaration() != id
            || state.proof.package_identity().root().crate_id != id.crate_id
            || state.proof.read_type() != &expected.ty()
            || state.proof.value() != &expected
        {
            return Err(
                "foreign compiler identity/type/value differs from C constant certificate".into(),
            );
        }
        c(state.registry.import_constant(state.proof.clone()))
    }
}
