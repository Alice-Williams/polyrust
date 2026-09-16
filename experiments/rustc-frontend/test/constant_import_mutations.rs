//! Test-only corruption of expected compiler values at the target join.
use super::LiteralValue;

#[cfg(constant_import_wrong_type)]
pub(crate) fn value(original: LiteralValue) -> LiteralValue {
    match original {
        LiteralValue::Bool(v) => LiteralValue::I32(i32::from(v)),
        LiteralValue::I32(_) | LiteralValue::I64(_) => LiteralValue::Bool(false),
    }
}
#[cfg(constant_import_wrong_value)]
pub(crate) fn value(original: LiteralValue) -> LiteralValue {
    match original {
        LiteralValue::Bool(v) => LiteralValue::Bool(!v),
        LiteralValue::I32(v) => LiteralValue::I32(v.wrapping_add(1)),
        LiteralValue::I64(v) => LiteralValue::I64(v.wrapping_add(1)),
    }
}
