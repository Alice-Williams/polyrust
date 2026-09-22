//! Exact scalar metadata uses the same reservation and encoding traversal.
use crate::json::Sink;
use portable_backend_java::ast::{JavaLiteral, JavaScalarConstantValue};

pub(crate) fn certified_value(
    out: &mut impl Sink,
    constant: &JavaScalarConstantValue,
) -> Result<(), String> {
    let literal = constant
        .literal()
        .ok_or("nonfinite Java source constants are not admitted yet")?;
    value(out, &literal)
}

pub(crate) fn value(out: &mut impl Sink, value: &JavaLiteral) -> Result<(), String> {
    match value {
        JavaLiteral::Boolean(value) => out.fixed(if *value { "true" } else { "false" }),
        JavaLiteral::I32(value) => out.string(&value.to_string()),
        JavaLiteral::I64(value) => out.string(&value.to_string()),
        JavaLiteral::F64(value) => out.string(&format!("0x{:016x}", value.to_bits())),
        _ => Err("unsupported Java constant metadata literal".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::{Encoder, Reservation};

    #[test]
    fn infinity_inventory_cannot_cross_the_finite_source_manifest_boundary() {
        for sign in [
            portable_binary64::Binary64Sign::Positive,
            portable_binary64::Binary64Sign::Negative,
        ] {
            let constant = JavaScalarConstantValue::Infinity(sign);
            let mut reservation = Reservation::default();
            assert!(certified_value(&mut reservation, &constant).is_err());
            assert_eq!(reservation.0.0, 0);
            let mut encoder = Encoder::new(100);
            assert!(certified_value(&mut encoder, &constant).is_err());
            assert_eq!(encoder.finish(), "");
        }
    }

    #[test]
    fn exact_values_are_lossless_and_fully_reserved() {
        for (literal, text) in [
            (JavaLiteral::Boolean(false), "false"),
            (JavaLiteral::Boolean(true), "true"),
            (JavaLiteral::I32(i32::MIN), "\"-2147483648\""),
            (JavaLiteral::I32(i32::MAX), "\"2147483647\""),
            (JavaLiteral::I64(i64::MIN), "\"-9223372036854775808\""),
            (JavaLiteral::I64(i64::MAX), "\"9223372036854775807\""),
            (
                JavaLiteral::I64(9_007_199_254_740_993),
                "\"9007199254740993\"",
            ),
            (
                JavaLiteral::I64(-9_007_199_254_740_993),
                "\"-9007199254740993\"",
            ),
        ] {
            let mut reservation = Reservation::default();
            value(&mut reservation, &literal).unwrap();
            assert!(reservation.0.0 >= text.len() as u64);
            let mut encoder = Encoder::new(reservation.0.0);
            value(&mut encoder, &literal).unwrap();
            assert_eq!(encoder.finish(), text);
            let mut too_small = Encoder::new(text.len() as u64 - 1);
            assert!(value(&mut too_small, &literal).is_err());
        }
    }

    #[test]
    fn finite_values_preserve_bits_and_sufficient_reservation() {
        for bits in [
            0,
            0x8000_0000_0000_0000,
            1,
            0x0010_0000_0000_0000,
            0x3ff0_0000_0000_0001,
            0x7fef_ffff_ffff_ffff,
            0xffef_ffff_ffff_ffff,
        ] {
            let literal =
                JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(bits).unwrap());
            let expected = format!("\"0x{bits:016x}\"");
            let mut reservation = Reservation::default();
            value(&mut reservation, &literal).unwrap();
            assert!(reservation.0.0 >= expected.len() as u64);
            let mut encoder = Encoder::new(reservation.0.0);
            value(&mut encoder, &literal).unwrap();
            assert_eq!(encoder.finish(), expected);
            assert!(value(&mut Encoder::new(expected.len() as u64 - 1), &literal).is_err());
        }
    }

    #[test]
    fn unsupported_values_reject_before_encoding() {
        assert!(
            value(
                &mut Reservation::default(),
                &JavaLiteral::String("not a scalar".into())
            )
            .is_err()
        );
        assert!(
            value(
                &mut Encoder::new(100),
                &JavaLiteral::String("not a scalar".into())
            )
            .is_err()
        );
    }
}
