//! Compiler scalar signatures map to the existing Java method signature.
use super::{FunctionInput, FunctionSignatures, Mapping};
use crate::java_lower::{Result, TypePlan};
use portable_backend_java::ast::{JavaMethodSignature, JavaPrimitive, JavaType};
use rustc_abi::ExternAbi;
use rustc_hir::def::DefKind;
use rustc_middle::ty;

#[derive(Clone, Copy)]
pub(crate) struct JavaFunctionSignatures;

impl Mapping for JavaFunctionSignatures {
    type Capability = FunctionSignatures;
    type Context<'tcx> = ();
    type Output = JavaMethodSignature;

    fn lower<'tcx>(&self, _: &mut (), input: FunctionInput<'tcx>) -> Result<Self::Output> {
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
        let scalar = |ty: ty::Ty<'tcx>| match ty.kind() {
            ty::Int(ty::IntTy::I32) => Ok(TypePlan::I32.java_type()),
            ty::Int(ty::IntTy::I64) => Ok(TypePlan::I64.java_type()),
            ty::Bool => Ok(TypePlan::Bool.java_type()),
            _ => Err("direct-call signatures support only i32, i64 and bool".to_owned()),
        };
        Ok(JavaMethodSignature {
            receiver: None,
            parameters: signature
                .inputs()
                .iter()
                .map(|ty| scalar(*ty))
                .collect::<Result<_>>()?,
            result: match signature.output().kind() {
                ty::Tuple(fields) if fields.is_empty() => JavaType::primitive(JavaPrimitive::Void),
                _ => scalar(signature.output())?,
            },
            checked_exceptions: vec![],
            nullable_result: false,
            // The complete body/call-graph admission permits only immutable
            // scalar computations. This is not inferred from Rust's `fn` type.
            pure: true,
        })
    }
}
