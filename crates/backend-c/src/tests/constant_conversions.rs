//! Integer and binary64 conversion boundaries use target, not Rust cast rules.
use super::{CInteger, CNumber, known_values::known};
use crate::ast::{
    CBinaryOperator as B, CKnownConstant as K, CScalarRepresentation as R, CScalarType as T,
};
use crate::ownership::CSafetyError as E;

fn n(ty: T, value: i128) -> CNumber {
    CNumber::Integer(CInteger::checked(ty, value).unwrap())
}

#[test]
fn every_signed_narrowing_and_unsigned_conversion_has_exact_boundary_controls() {
    for ty in T::ALL {
        match ty.representation() {
            R::Signed(width) => {
                let high = 1_i128 << (width.bits() - 1);
                for value in [-high, high - 1] {
                    assert_eq!(
                        n(T::I64, value)
                            .convert(ty)
                            .unwrap()
                            .integer()
                            .unwrap()
                            .value(),
                        value
                    );
                }
                assert_eq!(n(T::U64, high).convert(ty), Err(E::IntegerRange));
                if width.bits() < 64 {
                    assert_eq!(n(T::I64, -high - 1).convert(ty), Err(E::IntegerRange));
                }
            }
            R::Unsigned(width) => {
                let high = 1_i128 << width.bits();
                assert_eq!(
                    n(T::I64, -1)
                        .convert(ty)
                        .unwrap()
                        .integer()
                        .unwrap()
                        .value(),
                    high - 1
                );
                if width.bits() < 64 {
                    assert_eq!(
                        n(T::U64, high)
                            .convert(ty)
                            .unwrap()
                            .integer()
                            .unwrap()
                            .value(),
                        0
                    );
                }
            }
            R::Bool => {
                for (value, expected) in [(-1, 1), (0, 0), (2, 1)] {
                    assert_eq!(
                        n(T::I64, value)
                            .convert(ty)
                            .unwrap()
                            .integer()
                            .unwrap()
                            .value(),
                        expected
                    );
                }
            }
            R::Binary64 => assert_eq!(
                n(T::U64, u64::MAX.into()).convert(ty).unwrap(),
                CNumber::Double(18446744073709551616.0)
            ),
        }
    }
}

#[test]
fn floating_conversion_checks_exclusive_exact_bounds_before_casting() {
    for ty in T::ALL
        .into_iter()
        .filter(|ty| !matches!(ty, T::Bool | T::F64))
    {
        let (min, upper) = match ty.representation() {
            R::Signed(width) => {
                let high = 1_u64 << (width.bits() - 1);
                (-(high as f64), high as f64)
            }
            R::Unsigned(width) => (0.0, (1_u128 << width.bits()) as f64),
            _ => unreachable!(),
        };
        for value in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            upper,
            min - (min.abs() * f64::EPSILON).max(1.0),
        ] {
            assert_eq!(
                CNumber::Double(value).convert(ty),
                Err(E::IntegerRange),
                "{ty:?} {value}"
            );
        }
        for value in [min, 0.0, 1.75] {
            assert_eq!(
                CNumber::Double(value)
                    .convert(ty)
                    .unwrap()
                    .integer()
                    .unwrap()
                    .value(),
                value.trunc() as i128
            );
        }
    }
    assert_eq!(
        CNumber::Double(f64::from_bits(0x43df_ffff_ffff_ffff))
            .convert(T::I64)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        i128::from(i64::MAX) - 1023
    );
    assert_eq!(
        CNumber::Double(f64::from_bits(0x43ef_ffff_ffff_ffff))
            .convert(T::U64)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        i128::from(u64::MAX) - 2047
    );
    assert_eq!(
        CNumber::Double(-0.75)
            .convert(T::U64)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        0
    );
    assert_eq!(CNumber::Double(-1.0).convert(T::U64), Err(E::IntegerRange));
    for (value, expected) in [(f64::NAN, 1), (f64::INFINITY, 1), (-0.0, 0)] {
        assert_eq!(
            CNumber::Double(value)
                .convert(T::Bool)
                .unwrap()
                .integer()
                .unwrap()
                .value(),
            expected
        );
    }
    assert_eq!(
        n(T::U64, 9007199254740993).convert(T::F64).unwrap(),
        CNumber::Double(9007199254740992.0)
    );
}

#[test]
fn floating_facts_keep_ieee_categories_without_claiming_nan_payloads() {
    assert_eq!(
        CNumber::Double(1.0)
            .binary(B::Divide, CNumber::Double(0.0))
            .unwrap(),
        CNumber::Double(f64::INFINITY)
    );
    let CNumber::Double(nan) = CNumber::Double(0.0)
        .binary(B::Divide, CNumber::Double(0.0))
        .unwrap()
    else {
        panic!("double")
    };
    assert!(nan.is_nan());
    assert!(
        !CNumber::Double(nan)
            .binary(B::Equal, CNumber::Double(nan))
            .unwrap()
            .truth()
    );
    assert!(
        CNumber::Double(nan)
            .binary(B::NotEqual, CNumber::Double(nan))
            .unwrap()
            .truth()
    );
    let CNumber::Double(zero) = CNumber::Double(-0.0)
        .binary(B::Multiply, CNumber::Double(2.0))
        .unwrap()
    else {
        panic!("double")
    };
    assert_eq!(zero.to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn every_integer_known_constant_has_its_pinned_value_and_declared_type() {
    for (constant, expected) in [
        (K::CharBit, 8),
        (K::IntMin, -2147483648),
        (K::IntMax, 2147483647),
        (K::I32Min, -2147483648),
        (K::I32Max, 2147483647),
        (K::U32Max, 4294967295),
        (K::I64Min, i128::from(i64::MIN)),
        (K::I64Max, i128::from(i64::MAX)),
        (K::U64Max, i128::from(u64::MAX)),
        (K::SizeMax, i128::from(u64::MAX)),
        (K::FloatRadix, 2),
        (K::DoubleMantissaDigits, 53),
        (K::DoubleMinExponent, -1021),
        (K::DoubleMaxExponent, 1024),
        (K::FloatEvaluationMethod, 0),
        (K::EndOfFile, -1),
    ] {
        let value = known(constant).unwrap();
        assert_eq!(value.value(), expected);
        assert_eq!(crate::ast::CObjectType::scalar(value.ty()), constant.ty());
    }
    for constant in [K::StandardInput, K::StandardOutput, K::StandardError] {
        assert_eq!(known(constant), Err(E::ExpectedNumericConstant));
    }
}
