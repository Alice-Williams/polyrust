//! Observe the actual mapper result and resolved compiler origin.
use super::{ConstantInput, ScalarConstantValue};
use crate::java_lower::Reader;
use portable_backend_java::ast::*;

pub(super) fn check<'tcx>(
    reader: &Reader<'tcx>,
    input: ConstantInput<'tcx>,
    value: &crate::java_lower::Value,
) {
    let (definition, expression) = input.origin();
    let rustc_hir::ExprKind::Path(ref path) = expression.kind else {
        panic!("constant path")
    };
    assert_eq!(
        reader
            .checked
            .qpath_res(path, expression.hir_id)
            .opt_def_id(),
        Some(definition)
    );
    let (primitive, kind) = match input.value() {
        ScalarConstantValue::I32(v) => (
            JavaPrimitive::Int,
            JavaExprKind::Literal(JavaLiteral::I32(v)),
        ),
        ScalarConstantValue::I64(v) => (
            JavaPrimitive::Long,
            JavaExprKind::Literal(JavaLiteral::I64(v)),
        ),
        ScalarConstantValue::Bool(v) => (
            JavaPrimitive::Boolean,
            JavaExprKind::Literal(JavaLiteral::Boolean(v)),
        ),
        ScalarConstantValue::F64(v) => (
            JavaPrimitive::Double,
            JavaExprKind::Literal(JavaLiteral::F64(v)),
        ),
        ScalarConstantValue::Infinity(sign) => (
            JavaPrimitive::Double,
            JavaExprKind::Value(JavaValueRef::KnownField(match sign {
                portable_binary64::Binary64Sign::Positive => {
                    portable_backend_java::dialect::JavaKnownField::DoublePositiveInfinity
                }
                portable_binary64::Binary64Sign::Negative => {
                    portable_backend_java::dialect::JavaKnownField::DoubleNegativeInfinity
                }
            })),
        ),
    };
    let expression = value.clone().into_expression();
    assert_eq!(expression.ty, JavaType::primitive(primitive));
    assert_eq!(expression.precedence, JavaPrecedence::Primary);
    assert_eq!(expression.kind, kind);
    eprintln!(
        "CONSTANT_AST\tjava\t{}\t{:?}",
        reader.tcx.def_path_str(definition),
        input.value()
    );
}
