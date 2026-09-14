//! Exact selected-entry admission; not complete portable Functions support.
use super::{EntryInput, EntrySignatures, Mapping};
use crate::c_lower::{Result, c};
use portable_backend_c::ast::{
    CFunctionType, CObjectType, CParameterType, CReturnType, CReturnValue, CScalarType,
};
use rustc_abi::ExternAbi;
use rustc_hir::def::DefKind;

#[derive(Clone, Copy)]
pub(crate) struct CEntrySignatures;
impl Mapping for CEntrySignatures {
    type Capability = EntrySignatures;
    type Context<'tcx> = ();
    type Output = CFunctionType;

    fn lower<'tcx>(&self, _: &mut (), input: EntryInput<'tcx>) -> Result<CFunctionType> {
        let EntryInput { tcx, root } = input;
        if tcx.def_kind(root) != DefKind::Fn {
            return Err("entry must be an ordinary function".into());
        }
        if tcx.generics_of(root).count() != 0 {
            return Err("generic entry functions are not implemented".into());
        }
        let body = tcx.hir_body_owned_by(root);
        let signature = tcx.fn_sig(root).instantiate_identity().skip_binder();
        if signature.abi() != ExternAbi::Rust
            || !signature.safety().is_safe()
            || signature.c_variadic()
            || signature.splatted().is_some()
        {
            return Err("entry must use a safe non-variadic ordinary Rust signature".into());
        }
        if signature.inputs().len() != 1
            || signature.inputs()[0] != tcx.types.i32
            || signature.output() != tcx.types.i32
            || body.params.len() != 1
        {
            return Err("entry signature must be fn(i32) -> i32".into());
        }
        let scalar = CObjectType::scalar(CScalarType::I32);
        Ok(CFunctionType::new(
            CReturnType::Value(c(CReturnValue::new(scalar.clone()))?),
            vec![c(CParameterType::new(scalar))?],
        ))
    }
}
