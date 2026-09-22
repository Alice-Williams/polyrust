//! Observe the actual mapper result and resolved compiler origin.
use super::{ConstantInput, ScalarConstantValue};
use crate::c_lower::Reader;
use portable_backend_c::ast::*;

pub(super) fn check<'tcx>(reader: &Reader<'tcx>, input: ConstantInput<'tcx>, value: &CValue) {
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
    let (scalar, literal) = match input.value() {
        ScalarConstantValue::I32(v) => (CScalarType::I32, CLiteral::Signed(CSignedLiteral::I32(v))),
        ScalarConstantValue::I64(v) => (CScalarType::I64, CLiteral::Signed(CSignedLiteral::I64(v))),
        ScalarConstantValue::Bool(v) => (CScalarType::Bool, CLiteral::Bool(v)),
        ScalarConstantValue::F64(v) => (CScalarType::F64, CLiteral::F64(v)),
    };
    assert_eq!(value.ty().kind(), &CObjectTypeKind::Scalar(scalar));
    assert_eq!(value.kind(), &CValueKind::Literal(literal));
    eprintln!(
        "CONSTANT_AST\tc\t{}\t{:?}",
        reader.tcx.def_path_str(definition),
        input.value()
    );
}
