//! Test-only corruption of expected compiler values at the target join.
use super::ScalarConstantValue;

#[cfg(constant_import_wrong_type)]
pub(crate) fn value(original: ScalarConstantValue) -> ScalarConstantValue {
    match original {
        ScalarConstantValue::Bool(v) => ScalarConstantValue::I32(i32::from(v)),
        ScalarConstantValue::I32(_) | ScalarConstantValue::I64(_) => {
            ScalarConstantValue::Bool(false)
        }
    }
}
#[cfg(constant_import_wrong_value)]
pub(crate) fn value(original: ScalarConstantValue) -> ScalarConstantValue {
    match original {
        ScalarConstantValue::Bool(v) => ScalarConstantValue::Bool(!v),
        ScalarConstantValue::I32(v) => ScalarConstantValue::I32(v.wrapping_add(1)),
        ScalarConstantValue::I64(v) => ScalarConstantValue::I64(v.wrapping_add(1)),
    }
}
