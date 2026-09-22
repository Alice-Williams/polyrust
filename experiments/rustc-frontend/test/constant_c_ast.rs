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
        ScalarConstantValue::I32(v) => (
            CScalarType::I32,
            Some(CLiteral::Signed(CSignedLiteral::I32(v))),
        ),
        ScalarConstantValue::I64(v) => (
            CScalarType::I64,
            Some(CLiteral::Signed(CSignedLiteral::I64(v))),
        ),
        ScalarConstantValue::Bool(v) => (CScalarType::Bool, Some(CLiteral::Bool(v))),
        ScalarConstantValue::F64(v) => (CScalarType::F64, Some(CLiteral::F64(v))),
        ScalarConstantValue::Infinity(sign) => {
            match (sign, value.kind()) {
                (
                    portable_binary64::Binary64Sign::Positive,
                    CValueKind::KnownConstant(CKnownConstant::DoubleInfinity),
                ) => {}
                (
                    portable_binary64::Binary64Sign::Negative,
                    CValueKind::Unary {
                        operator: CUnaryOperator::Negate,
                        operand,
                    },
                ) => {
                    assert_eq!(operand.ty(), &CObjectType::scalar(CScalarType::F64));
                    assert_eq!(
                        operand.kind(),
                        &CValueKind::KnownConstant(CKnownConstant::DoubleInfinity)
                    );
                }
                _ => panic!("wrong typed infinity sign or shape"),
            }
            (CScalarType::F64, None)
        }
    };
    assert_eq!(value.ty().kind(), &CObjectTypeKind::Scalar(scalar));
    if let Some(literal) = literal {
        assert_eq!(value.kind(), &CValueKind::Literal(literal));
    }
    eprintln!(
        "CONSTANT_AST\tc\t{}\t{:?}",
        reader.tcx.def_path_str(definition),
        input.value()
    );
}
