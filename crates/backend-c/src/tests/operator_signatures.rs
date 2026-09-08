//! Exhaustive operator-domain matrix over the measured scalar ABI model.

use crate::ast::{
    CBinaryOperator as B, COperatorError as E, CScalarType as T, CUnaryOperator as U,
};

#[test]
fn every_binary_operator_checks_every_scalar_pair() {
    for left in T::ALL {
        for right in T::ALL {
            for op in [B::Add, B::Subtract, B::Multiply, B::Divide] {
                assert_eq!(
                    op.result_type(left, right),
                    Ok(left.usual_arithmetic_conversion(right))
                );
            }
            for op in [
                B::Equal,
                B::NotEqual,
                B::Less,
                B::LessEqual,
                B::Greater,
                B::GreaterEqual,
            ] {
                assert_eq!(op.result_type(left, right), Ok(T::Int));
            }
            for op in [
                B::Remainder,
                B::BitAnd,
                B::BitOr,
                B::BitXor,
                B::ShiftLeft,
                B::ShiftRight,
            ] {
                let expected = if left == T::F64 || right == T::F64 {
                    Err(E::ExpectedInteger)
                } else if matches!(op, B::ShiftLeft | B::ShiftRight) {
                    Ok(left.integer_promotion().unwrap())
                } else {
                    Ok(left.usual_arithmetic_conversion(right))
                };
                assert_eq!(
                    op.result_type(left, right),
                    expected,
                    "{op:?} {left:?} {right:?}"
                );
            }
            for op in [B::LogicalAnd, B::LogicalOr] {
                let expected = if left == T::Bool && right == T::Bool {
                    Ok(T::Int)
                } else {
                    Err(E::ExpectedBool)
                };
                assert_eq!(op.result_type(left, right), expected);
            }
        }
    }
}

#[test]
fn every_unary_operator_observes_promotions_and_explicit_bool() {
    for ty in T::ALL {
        assert_eq!(
            U::Negate.result_type(ty),
            Ok(ty.integer_promotion().unwrap_or(T::F64))
        );
        assert_eq!(
            U::BitNot.result_type(ty),
            ty.integer_promotion().ok_or(E::ExpectedInteger)
        );
        assert_eq!(
            U::LogicalNot.result_type(ty),
            if ty == T::Bool {
                Ok(T::Int)
            } else {
                Err(E::ExpectedBool)
            }
        );
    }
    // Independent witnesses for shift-vs-arithmetic and C int-vs-Bool rules.
    assert_eq!(B::ShiftLeft.result_type(T::I8, T::U64), Ok(T::Int));
    assert_eq!(B::Add.result_type(T::I8, T::U64), Ok(T::U64));
    assert_eq!(B::Add.result_type(T::U32, T::I64), Ok(T::I64));
    assert_eq!(B::LogicalAnd.result_type(T::Bool, T::Bool), Ok(T::Int));
}
