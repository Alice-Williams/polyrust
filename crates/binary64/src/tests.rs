//! Independent bit classification, explicit value vectors and decomposition.
use super::{Binary64Sign as Sign, FiniteBinary64, FiniteClass, NonFiniteBinary64};

fn check(bits: u64) {
    let reference = f64::from_bits(bits);
    let result = FiniteBinary64::from_bits(bits);
    assert_eq!(result.is_ok(), reference.is_finite(), "{bits:016x}");
    let expected_sign = if reference.is_sign_negative() {
        Sign::Negative
    } else {
        Sign::Positive
    };
    match result {
        Ok(value) => {
            assert_eq!(value.to_bits(), bits);
            assert_eq!(value.sign(), expected_sign);
            let expected_class = if reference == 0.0 {
                FiniteClass::Zero
            } else if reference.is_normal() {
                FiniteClass::Normal
            } else {
                FiniteClass::Subnormal
            };
            assert_eq!(value.class(), expected_class);
            let parts = value.parts();
            assert_eq!(parts.sign(), expected_sign);
            assert!(parts.significand() < (1 << 53));
            assert!((-1074..=971).contains(&parts.exponent()));
            // Rebuild the representation independently from mathematical parts:
            // the highest significand bit fixes its normalized power of two.
            let magnitude = if parts.significand() == 0 {
                assert_eq!(parts.exponent(), 0);
                0
            } else if parts.significand() < (1 << 52) {
                assert_eq!(parts.exponent(), -1074);
                parts.significand()
            } else {
                let leading_power = 63 - parts.significand().leading_zeros();
                assert_eq!(leading_power, 52);
                let power = i32::from(parts.exponent()) + leading_power as i32;
                let encoded_power = u64::try_from(power + 1023).unwrap();
                assert!((1..2047).contains(&encoded_power));
                (encoded_power << 52) | (parts.significand() ^ (1 << leading_power))
            };
            let recomposed = magnitude
                | if parts.sign() == Sign::Negative {
                    1 << 63
                } else {
                    0
                };
            assert_eq!(recomposed, bits);
        }
        Err(error) => {
            let expected = if reference.is_infinite() {
                NonFiniteBinary64::Infinity(expected_sign)
            } else {
                assert!(reference.is_nan());
                NonFiniteBinary64::NaN
            };
            assert_eq!(error, expected);
        }
    }
}

#[test]
fn all_exponents_both_signs_and_fraction_boundaries() {
    let fractions = [
        0,
        1,
        2,
        (1 << 51) - 1,
        1 << 51,
        (1 << 52) - 2,
        (1 << 52) - 1,
    ];
    for sign in [0, 1_u64 << 63] {
        for exponent in 0..2048_u64 {
            for fraction in fractions {
                check(sign | (exponent << 52) | fraction);
            }
        }
    }
}

#[test]
fn deterministic_full_width_bit_corpus() {
    let mut state = 0x6a09_e667_f3bc_c909_u64;
    for _ in 0..100_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        check(state);
    }
}

#[test]
fn hand_derived_decomposition_boundaries() {
    // Hex values are independently specified IEEE encodings, not outputs from
    // the implementation under test. Negative cases reuse only the sign bit.
    let cases = [
        (0x0000_0000_0000_0000, FiniteClass::Zero, 0, 0),
        (0x0000_0000_0000_0001, FiniteClass::Subnormal, 1, -1074),
        (
            0x000f_ffff_ffff_ffff,
            FiniteClass::Subnormal,
            4_503_599_627_370_495,
            -1074,
        ),
        (
            0x0010_0000_0000_0000,
            FiniteClass::Normal,
            4_503_599_627_370_496,
            -1074,
        ),
        (
            0x3fef_ffff_ffff_ffff,
            FiniteClass::Normal,
            9_007_199_254_740_991,
            -53,
        ),
        (
            0x3ff0_0000_0000_0000,
            FiniteClass::Normal,
            4_503_599_627_370_496,
            -52,
        ),
        (
            0x3ff0_0000_0000_0001,
            FiniteClass::Normal,
            4_503_599_627_370_497,
            -52,
        ),
        (
            0x7fef_ffff_ffff_ffff,
            FiniteClass::Normal,
            9_007_199_254_740_991,
            971,
        ),
    ];
    for sign in [0, 1_u64 << 63] {
        for (bits, class, significand, exponent) in cases {
            let value = FiniteBinary64::from_bits(sign | bits).unwrap();
            assert_eq!(value.class(), class);
            assert_eq!(value.parts().significand(), significand);
            assert_eq!(value.parts().exponent(), exponent);
        }
    }
}

#[test]
fn rejects_infinities_and_quiet_or_signaling_nan_payloads() {
    for sign in [0, 1_u64 << 63] {
        for payload in [0, 1, 0x1234, 1 << 51, (1 << 51) | 0x1234, (1 << 52) - 1] {
            assert!(FiniteBinary64::from_bits(sign | 0x7ff0_0000_0000_0000 | payload).is_err());
        }
    }
}

#[test]
fn representation_equality_keeps_signed_zero_distinct() {
    const POSITIVE: FiniteBinary64 = match FiniteBinary64::from_bits(0) {
        Ok(value) => value,
        Err(_) => panic!("finite zero"),
    };
    const NEGATIVE: FiniteBinary64 = match FiniteBinary64::from_bits(1 << 63) {
        Ok(value) => value,
        Err(_) => panic!("finite negative zero"),
    };
    assert_ne!(POSITIVE, NEGATIVE);
    assert_eq!(POSITIVE, FiniteBinary64::from_bits(0).unwrap());
    assert_eq!(NEGATIVE.parts().sign(), Sign::Negative);
}
