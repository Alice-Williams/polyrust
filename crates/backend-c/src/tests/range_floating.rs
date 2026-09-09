//! NaN and both zero signs are part of numeric soundness, not formatting details.
use super::integer_tests::{TYPES, interval, number};
use super::{CScalarRange as Range, floating::FloatRange};
use crate::ast::{CBinaryOperator as B, CScalarType as T};
use crate::ownership::{
    CSafetyError as E,
    constants::{CNumber, limits},
};

#[test]
fn float_joins_preserve_nan_infinity_and_zero_sign_uncertainty() {
    let positive = Range::exact(CNumber::Double(0.0));
    let negative = Range::exact(CNumber::Double(-0.0));
    let zeros = positive.join(negative).unwrap();
    assert!(zeros.exact_value().is_none());
    assert_eq!(zeros.truth(), Some(false));
    assert!(zeros.contains(CNumber::Double(0.0)));
    assert!(zeros.contains(CNumber::Double(-0.0)));
    let reciprocal = Range::exact(CNumber::Double(1.0))
        .binary(B::Divide, zeros)
        .unwrap()
        .range();
    assert!(reciprocal.contains(CNumber::Double(f64::INFINITY)));
    assert!(reciprocal.contains(CNumber::Double(f64::NEG_INFINITY)));
    let nan = zeros.join(Range::exact(CNumber::Double(f64::NAN))).unwrap();
    assert_eq!(nan.truth(), None);
    assert!(nan.contains(CNumber::Double(f64::NAN)));
    assert_eq!(
        nan.convert(T::Bool).unwrap().range(),
        interval(T::Bool, 0, 1)
    );
    for value in [
        f64::NEG_INFINITY,
        -f64::MAX,
        -0.0,
        0.0,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
    ] {
        assert!(
            Range::full(T::F64)
                .unwrap()
                .contains(CNumber::Double(value))
        );
    }
    assert_eq!(
        FloatRange::checked(f64::NAN, 1.0, false),
        Err(E::InvalidNumericRange)
    );
    assert_eq!(
        FloatRange::checked(2.0, 1.0, false),
        Err(E::InvalidNumericRange)
    );
}

#[test]
fn float_interval_casts_use_finite_truncation_and_exact_exclusive_integer_limits() {
    for ty in TYPES {
        if matches!(ty, T::Bool | T::F64) {
            continue;
        }
        let (min, max) = limits(ty).unwrap();
        let upper = (max + 1) as f64;
        let safe_upper = upper.next_down();
        let range = Range::Float(FloatRange::checked(min as f64, safe_upper, false).unwrap());
        let result = range.convert(ty).unwrap().range();
        assert!(result.contains(number(ty, min)));
        assert!(result.contains(CNumber::Double(safe_upper).convert(ty).unwrap()));
        assert_eq!(
            Range::Float(FloatRange::checked(min as f64, upper, false).unwrap()).convert(ty),
            Err(E::IntegerRange)
        );
        assert_eq!(
            Range::Float(FloatRange::checked(min as f64, safe_upper, true).unwrap()).convert(ty),
            Err(E::IntegerRange)
        );
    }
    let fraction = Range::Float(FloatRange::checked(-0.9, 0.9, false).unwrap());
    assert_eq!(
        fraction.convert(T::U64).unwrap().range(),
        interval(T::U64, 0, 0)
    );
    assert_eq!(
        Range::Float(FloatRange::checked(f64::NEG_INFINITY, 0.0, false).unwrap()).convert(T::I64),
        Err(E::IntegerRange)
    );
    assert_eq!(
        Range::Float(FloatRange::checked(0.0, f64::INFINITY, false).unwrap()).convert(T::U64),
        Err(E::IntegerRange)
    );
}

#[test]
fn integer_to_double_intervals_include_rounded_extrema_without_inventing_roundtrip_safety() {
    for ty in TYPES {
        if ty == T::F64 {
            continue;
        }
        let range = Range::full(ty).unwrap().convert(T::F64).unwrap().range();
        let (min, max) = limits(ty).unwrap();
        for value in [min, max, 0] {
            assert!(range.contains(number(ty, value).convert(T::F64).unwrap()));
        }
    }
    assert_eq!(
        Range::full(T::U64)
            .unwrap()
            .convert(T::F64)
            .unwrap()
            .range()
            .convert(T::U64),
        Err(E::IntegerRange)
    );
    assert_eq!(
        Range::exact(number(T::U64, 1_i128 << 53))
            .convert(T::F64)
            .unwrap()
            .range(),
        Range::exact(number(T::U64, (1_i128 << 53) + 1))
            .convert(T::F64)
            .unwrap()
            .range()
    );
}
