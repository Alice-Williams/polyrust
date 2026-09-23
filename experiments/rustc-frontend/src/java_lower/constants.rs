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
pub(super) fn value(input: ScalarConstantValue) -> (super::TypePlan, JavaScalarConstantValue) {
    match input {
        ScalarConstantValue::Bool(value) => (
            super::TypePlan::Bool,
            JavaScalarConstantValue::Boolean(value),
        ),
        ScalarConstantValue::I32(value) => {
            (super::TypePlan::I32, JavaScalarConstantValue::I32(value))
        }
        ScalarConstantValue::I64(value) => {
            (super::TypePlan::I64, JavaScalarConstantValue::I64(value))
        }
        ScalarConstantValue::Char(value) => (
            super::TypePlan::Char,
            JavaScalarConstantValue::I32(u32::from(value) as i32),
        ),
        ScalarConstantValue::F64(value) => {
            (super::TypePlan::F64, JavaScalarConstantValue::F64(value))
        }
        ScalarConstantValue::Infinity(sign) => (
            super::TypePlan::F64,
            JavaScalarConstantValue::Infinity(sign),
        ),
    }
}
pub(super) fn expression(input: ScalarConstantValue) -> (super::TypePlan, JavaExpr) {
    let (plan, value) = value(input);
    let literal = match value {
        JavaScalarConstantValue::Boolean(value) => JavaLiteral::Boolean(value),
        JavaScalarConstantValue::I32(value) => JavaLiteral::I32(value),
        JavaScalarConstantValue::I64(value) => JavaLiteral::I64(value),
        JavaScalarConstantValue::F64(value) => JavaLiteral::F64(value),
        JavaScalarConstantValue::Infinity(sign) => {
            let field = match sign {
                portable_binary64::Binary64Sign::Positive => {
                    portable_backend_java::dialect::JavaKnownField::DoublePositiveInfinity
                }
                portable_binary64::Binary64Sign::Negative => {
                    portable_backend_java::dialect::JavaKnownField::DoubleNegativeInfinity
                }
            };
            let expression = JavaExpr {
                ty: plan.java_type(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::KnownField(field)),
            };
            return (plan, expression);
        }
    };
    let expression = JavaExpr::literal(plan.java_type(), literal);
    (plan, expression)
}
