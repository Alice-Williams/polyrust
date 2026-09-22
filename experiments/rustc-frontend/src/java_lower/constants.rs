//! Owned field identity stays separate from its source spelling.
use crate::source_capabilities::ScalarConstantValue;
use portable_backend_java::ast::*;
use portable_codegen::GeneratedValueId;

#[derive(Clone)]
#[cfg_attr(local_constant_ast_probe, derive(Debug))]
pub(crate) struct Constant {
    pub id: GeneratedValueId,
    pub definition: rustc_hir::def_id::DefId,
    pub field: JavaField,
    pub value: ScalarConstantValue,
}
pub(super) fn literal(value: ScalarConstantValue) -> (super::TypePlan, JavaLiteral) {
    match value {
        ScalarConstantValue::Bool(value) => (super::TypePlan::Bool, JavaLiteral::Boolean(value)),
        ScalarConstantValue::I32(value) => (super::TypePlan::I32, JavaLiteral::I32(value)),
        ScalarConstantValue::I64(value) => (super::TypePlan::I64, JavaLiteral::I64(value)),
        ScalarConstantValue::F64(value) => (super::TypePlan::F64, JavaLiteral::F64(value)),
    }
}
