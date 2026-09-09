//! Independent value boundaries alongside the existing native 169-type matrix.
use super::{CInteger, CNumber};
use crate::ast::{
    CBinaryOperator as B, CScalarRepresentation as R, CScalarType as T, CUnaryOperator as U,
};
use crate::ownership::CSafetyError as E;

fn n(ty: T, value: i128) -> CNumber {
    CNumber::Integer(CInteger::checked(ty, value).unwrap())
}
fn integer(value: CNumber) -> i128 {
    value.integer().unwrap().value()
}

#[test]
fn each_integer_width_checks_storage_and_normalizes_only_explicit_conversions() {
    for ty in T::ALL {
        let bounds = match ty.representation() {
            R::Bool => (0, 1),
            R::Signed(width) => {
                let limit = 1_i128 << (width.bits() - 1);
                (-limit, limit - 1)
            }
            R::Unsigned(width) => (0, (1_i128 << width.bits()) - 1),
            R::Binary64 => {
                assert_eq!(CInteger::checked(ty, 0), Err(E::ExpectedIntegerConstant));
                continue;
            }
        };
        for value in [bounds.0, bounds.1, 0, 1] {
            let actual = CInteger::checked(ty, value).unwrap();
            assert_eq!(actual.ty(), ty);
            assert_eq!(actual.value(), value);
        }
        assert_eq!(CInteger::checked(ty, bounds.0 - 1), Err(E::IntegerRange));
        assert_eq!(CInteger::checked(ty, bounds.1 + 1), Err(E::IntegerRange));
    }
}

#[test]
fn every_scalar_pair_and_operator_keeps_the_constructor_relation_and_numeric_value() {
    let operations = [
        B::Add,
        B::Subtract,
        B::Multiply,
        B::Divide,
        B::Remainder,
        B::ShiftLeft,
        B::ShiftRight,
        B::BitAnd,
        B::BitOr,
        B::BitXor,
        B::Equal,
        B::NotEqual,
        B::Less,
        B::LessEqual,
        B::Greater,
        B::GreaterEqual,
        B::LogicalAnd,
        B::LogicalOr,
    ];
    for left in T::ALL {
        for right in T::ALL {
            let a = if left == T::F64 {
                CNumber::Double(1.0)
            } else {
                n(left, 1)
            };
            let b = if right == T::F64 {
                CNumber::Double(1.0)
            } else {
                n(right, 1)
            };
            for operation in operations {
                let actual = a.binary(operation, b);
                let ty = match operation.result_type(left, right) {
                    Ok(ty) => ty,
                    Err(error) => {
                        assert_eq!(actual, Err(E::Operator(error)));
                        continue;
                    }
                };
                let expected = match operation {
                    B::Add | B::ShiftLeft => 2,
                    B::Subtract
                    | B::Remainder
                    | B::ShiftRight
                    | B::BitXor
                    | B::NotEqual
                    | B::Less
                    | B::Greater => 0,
                    B::Multiply
                    | B::Divide
                    | B::BitAnd
                    | B::BitOr
                    | B::Equal
                    | B::LessEqual
                    | B::GreaterEqual
                    | B::LogicalAnd
                    | B::LogicalOr => 1,
                };
                let actual = actual.unwrap();
                assert!(actual.ty().is_compatible_with(ty));
                match actual {
                    CNumber::Integer(value) => assert_eq!(value.value(), expected),
                    CNumber::Double(value) => assert_eq!(value, expected as f64),
                }
            }
        }
    }
}

#[test]
fn promotion_wrapping_and_signed_failures_are_separate() {
    for (ty, max, expected) in [
        (T::I8, 127, 254),
        (T::U8, 255, 510),
        (T::I16, 32767, 65534),
        (T::U16, 65535, 131070),
    ] {
        let value = n(ty, max).binary(B::Add, n(ty, max)).unwrap();
        assert_eq!(value.ty(), T::Int);
        assert_eq!(integer(value), expected);
    }
    for (ty, max) in [
        (T::U32, i128::from(u32::MAX)),
        (T::U64, i128::from(u64::MAX)),
        (T::Size, i128::from(u64::MAX)),
    ] {
        assert_eq!(integer(n(ty, max).binary(B::Add, n(ty, 1)).unwrap()), 0);
        assert_eq!(
            integer(n(ty, 0).binary(B::Subtract, n(ty, 1)).unwrap()),
            max
        );
        assert_eq!(
            integer(n(ty, max).binary(B::Multiply, n(ty, max)).unwrap()),
            1
        );
        assert_eq!(integer(n(ty, max).unary(U::Negate).unwrap()), 1);
        assert_eq!(integer(n(ty, 0).unary(U::BitNot).unwrap()), max);
    }
    for (ty, min, max) in [
        (T::Int, i128::from(i32::MIN), i128::from(i32::MAX)),
        (T::I32, i128::from(i32::MIN), i128::from(i32::MAX)),
        (T::I64, i128::from(i64::MIN), i128::from(i64::MAX)),
    ] {
        for (op, a, b) in [
            (B::Add, max, 1),
            (B::Subtract, min, 1),
            (B::Multiply, max, 2),
            (B::Divide, min, -1),
            (B::Remainder, min, -1),
        ] {
            assert_eq!(n(ty, a).binary(op, n(ty, b)), Err(E::SignedOverflow));
        }
        assert_eq!(n(ty, min).unary(U::Negate), Err(E::SignedOverflow));
        assert_eq!(integer(n(ty, -1).unary(U::BitNot).unwrap()), 0);
    }
    assert_eq!(
        integer(
            n(T::U32, u32::MAX.into())
                .binary(B::Add, n(T::I64, -1))
                .unwrap()
        ),
        i128::from(u32::MAX) - 1
    );
    assert_eq!(
        integer(
            n(T::U64, u64::MAX.into())
                .binary(B::Equal, n(T::I64, -1))
                .unwrap()
        ),
        1
    );
    assert_eq!(
        integer(n(T::Int, -7).binary(B::Divide, n(T::Int, 3)).unwrap()),
        -2
    );
    assert_eq!(
        integer(n(T::Int, -7).binary(B::Remainder, n(T::Int, 3)).unwrap()),
        -1
    );
}

#[test]
fn zero_division_and_shift_boundaries_never_use_host_masked_counts() {
    for ty in T::ALL.into_iter().filter(|ty| *ty != T::F64) {
        for op in [B::Divide, B::Remainder] {
            assert_eq!(n(ty, 1).binary(op, n(ty, 0)), Err(E::DivisionByZero));
        }
        let promoted = ty.integer_promotion().unwrap();
        let (width, signed) = match promoted.representation() {
            R::Signed(width) => (width.bits(), true),
            R::Unsigned(width) => (width.bits(), false),
            _ => unreachable!(),
        };
        for count in [-1, i128::from(width), i128::from(width) + 1] {
            for op in [B::ShiftLeft, B::ShiftRight] {
                assert_eq!(n(ty, 1).binary(op, n(T::I64, count)), Err(E::InvalidShift));
            }
        }
        let last = n(T::Size, i128::from(width - 1));
        let left = n(ty, 1).binary(B::ShiftLeft, last);
        if signed {
            assert_eq!(left, Err(E::InvalidShift));
        } else {
            assert_eq!(integer(left.unwrap()), 1_i128 << (width - 1));
        }
        assert_eq!(integer(n(ty, 1).binary(B::ShiftRight, last).unwrap()), 0);
    }
    assert_eq!(
        n(T::I64, -1).binary(B::ShiftLeft, n(T::Int, 0)),
        Err(E::InvalidShift)
    );
    assert_eq!(
        integer(n(T::I64, -3).binary(B::ShiftRight, n(T::Int, 1)).unwrap()),
        -2
    );
    assert_eq!(
        n(T::U64, 1).binary(B::ShiftLeft, n(T::U64, u64::MAX.into())),
        Err(E::InvalidShift)
    );
}

#[test]
fn all_unary_operators_cover_all_scalar_categories() {
    for ty in T::ALL {
        let input = if ty == T::F64 {
            CNumber::Double(1.0)
        } else {
            n(ty, 1)
        };
        for op in [U::LogicalNot, U::Negate, U::BitNot] {
            let expected_type = match op.result_type(ty) {
                Ok(ty) => ty,
                Err(error) => {
                    assert_eq!(input.unary(op), Err(E::Operator(error)));
                    continue;
                }
            };
            let actual = input.unary(op).unwrap();
            assert!(actual.ty().is_compatible_with(expected_type));
            if let CNumber::Double(value) = actual {
                assert_eq!(value, -1.0);
                continue;
            }
            let expected = match (op, expected_type.representation()) {
                (U::LogicalNot, _) => 0,
                (U::Negate, R::Unsigned(width)) => (1_i128 << width.bits()) - 1,
                (U::BitNot, R::Unsigned(width)) => (1_i128 << width.bits()) - 2,
                (U::Negate, _) => -1,
                (U::BitNot, _) => -2,
            };
            assert_eq!(integer(actual), expected);
        }
    }
}
