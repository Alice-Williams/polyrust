//! Test-only corruption of expected compiler values at the target join.
use super::ScalarConstantValue;

#[cfg(constant_import_wrong_type)]
pub(crate) fn value(original: ScalarConstantValue) -> ScalarConstantValue {
    match original {
        ScalarConstantValue::Bool(v) => ScalarConstantValue::I32(i32::from(v)),
        ScalarConstantValue::Char(v) => ScalarConstantValue::I32(u32::from(v) as i32),
        ScalarConstantValue::I32(_)
        | ScalarConstantValue::I64(_)
        | ScalarConstantValue::F64(_)
        | ScalarConstantValue::Infinity(_) => ScalarConstantValue::Bool(false),
    }
}
#[cfg(constant_import_wrong_value)]
pub(crate) fn value(original: ScalarConstantValue) -> ScalarConstantValue {
    match original {
        ScalarConstantValue::Bool(v) => ScalarConstantValue::Bool(!v),
        ScalarConstantValue::Char(v) => {
            ScalarConstantValue::Char(char::from_u32(u32::from(v) ^ 1).unwrap())
        }
        ScalarConstantValue::I32(v) => ScalarConstantValue::I32(v.wrapping_add(1)),
        ScalarConstantValue::I64(v) => ScalarConstantValue::I64(v.wrapping_add(1)),
        ScalarConstantValue::F64(v) => ScalarConstantValue::F64(
            portable_binary64::FiniteBinary64::from_bits(v.to_bits() ^ (1_u64 << 63)).unwrap(),
        ),
        ScalarConstantValue::Infinity(sign) => ScalarConstantValue::Infinity(match sign {
            portable_binary64::Binary64Sign::Positive => portable_binary64::Binary64Sign::Negative,
            portable_binary64::Binary64Sign::Negative => portable_binary64::Binary64Sign::Positive,
        }),
    }
}
