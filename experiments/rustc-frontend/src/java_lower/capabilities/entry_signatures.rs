//! Selected-entry harness, distinct from unrestricted scalar parameter lists.
use super::{EntryInput, EntrySignatures, FunctionInput, JavaFunctionSignatures, Mapping};
use crate::java_lower::{Result, TypePlan};
use portable_backend_java::ast::JavaMethodSignature;

#[derive(Clone, Copy)]
pub(crate) struct JavaEntrySignatures;

impl Mapping for JavaEntrySignatures {
    type Capability = EntrySignatures;
    type Context<'tcx> = ();
    type Output = JavaMethodSignature;

    fn lower<'tcx>(&self, _: &mut (), input: EntryInput<'tcx>) -> Result<Self::Output> {
        let EntryInput { tcx, root } = input;
        let signature = JavaFunctionSignatures.lower(
            &mut (),
            FunctionInput {
                tcx,
                function: root.to_def_id(),
            },
        )?;
        if signature.parameters != [TypePlan::I32.java_type()]
            || signature.result != TypePlan::I32.java_type()
            || tcx.hir_body_owned_by(root).params.len() != 1
        {
            return Err("entry signature must be fn(i32) -> i32".into());
        }
        Ok(signature)
    }
}
