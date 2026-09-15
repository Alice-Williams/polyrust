//! Inspect the actual mapped Bool -> Int -> Bool promotion structure.
use super::{EagerBooleanInput, EagerBooleanOperator};
use portable_backend_c::ast::*;

pub(super) fn check(input: &EagerBooleanInput<'_>, value: &CValue) {
    assert_eq!(
        value.ty().kind(),
        &CObjectTypeKind::Scalar(CScalarType::Bool)
    );
    let CValueKind::Convert {
        conversion: CConversion::Numeric(CScalarType::Bool),
        operand,
    } = value.kind()
    else {
        panic!("Boolean normalization")
    };
    assert_eq!(
        operand.ty().kind(),
        &CObjectTypeKind::Scalar(CScalarType::Int)
    );
    let CValueKind::Binary {
        operator,
        left,
        right,
    } = operand.kind()
    else {
        panic!("eager binary")
    };
    let expected = match input.operator() {
        EagerBooleanOperator::And => CBinaryOperator::BitAnd,
        EagerBooleanOperator::Or => CBinaryOperator::BitOr,
        EagerBooleanOperator::Xor => CBinaryOperator::BitXor,
    };
    assert_eq!(*operator, expected);
    for value in [left, right] {
        assert_eq!(
            value.ty().kind(),
            &CObjectTypeKind::Scalar(CScalarType::Bool)
        );
    }
    eprintln!("EAGER_AST\tc");
}
