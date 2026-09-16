//! Owned field identity stays separate from its source spelling.
use crate::source_capabilities::LiteralValue;
use portable_backend_java::ast::*;
use portable_codegen::GeneratedValueId;

#[derive(Clone)]
#[cfg_attr(local_constant_ast_probe, derive(Debug))]
pub(crate) struct Constant {
    pub id: GeneratedValueId,
    pub definition: rustc_hir::def_id::DefId,
    pub field: JavaField,
    pub value: LiteralValue,
}
pub(super) fn literal(value: LiteralValue) -> (super::TypePlan, JavaLiteral) {
    match value {
        LiteralValue::Bool(value) => (super::TypePlan::Bool, JavaLiteral::Boolean(value)),
        LiteralValue::I32(value) => (super::TypePlan::I32, JavaLiteral::I32(value)),
        LiteralValue::I64(value) => (super::TypePlan::I64, JavaLiteral::I64(value)),
    }
}
