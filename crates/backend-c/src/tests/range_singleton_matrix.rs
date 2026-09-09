//! Exact endpoints cover every actual scalar/category pair and operation.
use super::integer_tests::{OPS, TYPES, number};
use super::{CScalarRange as Range, integer::IntegerRange};
use crate::ast::{CScalarType as T, CUnaryOperator as U};
use crate::ownership::{
    CSafetyError as E,
    constants::{CNumber, limits},
};

fn samples(ty: T) -> Vec<CNumber> {
    if ty == T::F64 {
        return [
            f64::NEG_INFINITY,
            -1.5,
            -0.0,
            0.0,
            1.5,
            f64::INFINITY,
            f64::NAN,
        ]
        .into_iter()
        .map(CNumber::Double)
        .collect();
    }
    let (min, max) = limits(ty).unwrap();
    let mut values = vec![min, min + 1, -1, 0, 1, max - 1, max];
    values.retain(|value| min <= *value && *value <= max);
    values.sort_unstable();
    values.dedup();
    values.into_iter().map(|value| number(ty, value)).collect()
}

#[test]
fn all_scalar_pairs_and_binary_operators_match_the_independent_exact_model_at_boundaries() {
    for left_type in TYPES {
        for right_type in TYPES {
            for left in samples(left_type) {
                for right in samples(right_type) {
                    for op in OPS {
                        let expected = left.binary(op, right);
                        let actual = Range::exact(left).binary(op, Range::exact(right));
                        match (expected, actual) {
                            (Ok(expected), Ok(actual)) => {
                                assert!(
                                    actual.range().contains(expected),
                                    "{left:?} {op:?} {right:?}: {actual:?}"
                                );
                                assert!(actual.range().exact_value().is_some());
                            }
                            (Err(expected), Err(actual)) => assert_eq!(actual, expected),
                            pair => panic!("{left:?} {op:?} {right:?}: {pair:?}"),
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn all_scalar_unary_and_conversion_boundaries_match_exact_evaluation() {
    for ty in TYPES {
        for value in samples(ty) {
            for op in [U::Negate, U::BitNot, U::LogicalNot] {
                match (value.unary(op), Range::exact(value).unary(op)) {
                    (Ok(expected), Ok(actual)) => assert!(actual.range().contains(expected)),
                    (Err(expected), Err(actual)) => assert_eq!(actual, expected),
                    pair => panic!("{value:?} {op:?}: {pair:?}"),
                }
            }
            for destination in TYPES {
                match (
                    value.convert(destination),
                    Range::exact(value).convert(destination),
                ) {
                    (Ok(expected), Ok(actual)) => assert!(actual.range().contains(expected)),
                    (Err(expected), Err(actual)) => assert_eq!(actual, expected),
                    pair => panic!("{value:?} -> {destination:?}: {pair:?}"),
                }
            }
        }
    }
}

#[test]
fn invalid_bounds_and_mismatched_join_types_cannot_construct_valid_integer_intervals() {
    assert_eq!(
        IntegerRange::checked(T::I8, 2, 1),
        Err(E::InvalidNumericRange)
    );
    assert_eq!(
        IntegerRange::checked(T::I8, -129, 127),
        Err(E::IntegerRange)
    );
    assert_eq!(IntegerRange::checked(T::U8, 0, 256), Err(E::IntegerRange));
    assert_eq!(IntegerRange::full(T::F64), Err(E::ExpectedIntegerConstant));
    assert_eq!(
        Range::exact(number(T::Int, 0)).join(Range::exact(number(T::U64, 0))),
        Err(E::InvalidNumericRange)
    );
}
