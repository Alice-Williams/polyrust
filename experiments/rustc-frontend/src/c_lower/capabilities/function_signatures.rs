//! Scalar helper signatures; entry ABI selection is a separate capability.
use super::{FunctionInput, FunctionSignatures, Mapping};
use crate::c_lower::{Result, c};
use portable_backend_c::ast::*;
use rustc_abi::ExternAbi;
use rustc_hir::def::DefKind;
use rustc_middle::ty;

#[derive(Clone, Copy)]
pub(crate) struct CFunctionSignatures;
impl Mapping for CFunctionSignatures {
    type Capability = FunctionSignatures;
    type Context<'tcx> = ();
    type Output = CFunctionType;

    fn lower<'tcx>(&self, _: &mut (), input: FunctionInput<'tcx>) -> Result<CFunctionType> {
        let FunctionInput { tcx, function } = input;
        if tcx.def_kind(function) != DefKind::Fn || tcx.generics_of(function).count() != 0 {
            return Err("direct calls require nongeneric ordinary functions".into());
        }
        let signature = tcx.fn_sig(function).instantiate_identity().skip_binder();
        if signature.abi() != ExternAbi::Rust
            || !signature.safety().is_safe()
            || signature.c_variadic()
            || signature.splatted().is_some()
        {
            return Err("direct calls require safe non-variadic ordinary Rust signatures".into());
        }
        let scalar = |ty: ty::Ty<'tcx>| -> Result<CObjectType> {
            Ok(CObjectType::scalar(match ty.kind() {
                ty::Int(ty::IntTy::I32) => CScalarType::I32,
                ty::Int(ty::IntTy::I64) => CScalarType::I64,
                ty::Bool => CScalarType::Bool,
                ty::Char => CScalarType::U32,
                ty::Float(ty::FloatTy::F64) => CScalarType::F64,
                _ => {
                    return Err(
                        "direct-call signatures require admitted scalar source types".into(),
                    );
                }
            }))
        };
        let parameters = signature
            .inputs()
            .iter()
            .map(|ty| c(CParameterType::new(scalar(*ty)?)))
            .collect::<Result<_>>()?;
        Ok(CFunctionType::new(
            match signature.output().kind() {
                ty::Tuple(fields) if fields.is_empty() => CReturnType::Void,
                _ => CReturnType::Value(c(CReturnValue::new(scalar(signature.output())?))?),
            },
            parameters,
        ))
    }
}
