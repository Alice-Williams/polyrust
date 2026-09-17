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
    let (primitive, literal) = match input.value() {
        ScalarConstantValue::I32(v) => (JavaPrimitive::Int, JavaLiteral::I32(v)),
        ScalarConstantValue::I64(v) => (JavaPrimitive::Long, JavaLiteral::I64(v)),
        ScalarConstantValue::Bool(v) => (JavaPrimitive::Boolean, JavaLiteral::Boolean(v)),
    };
    let expression = value.clone().into_expression();
    assert_eq!(expression.ty, JavaType::primitive(primitive));
    assert_eq!(expression.kind, JavaExprKind::Literal(literal));
    eprintln!(
        "CONSTANT_AST\tjava\t{}\t{:?}",
        reader.tcx.def_path_str(definition),
        input.value()
    );
}
