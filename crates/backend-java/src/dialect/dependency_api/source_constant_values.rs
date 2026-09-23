//! Reconcile original source values with exact primitive target constant facts.
use crate::ast::JavaScalarConstantValue as Target;
use portable_binary64::Binary64Sign;
use portable_codegen::RustConstantValue as Original;

pub(super) fn matches(original: Original, target: &Target) -> bool {
    match (original, target) {
        (Original::Bool(left), Target::Boolean(right)) => left == *right,
        (Original::I32(left), Target::I32(right)) => left == *right,
        (Original::I64(left), Target::I64(right)) => left == *right,
        (Original::Char(left), Target::I32(right)) => u32::from(left) as i32 == *right,
        (Original::F64Bits(left), Target::F64(right)) => left == right.to_bits(),
        (Original::F64Bits(left), Target::Infinity(sign)) => {
            left == match sign {
                Binary64Sign::Positive => 0x7ff0_0000_0000_0000,
                Binary64Sign::Negative => 0xfff0_0000_0000_0000,
            }
        }
        _ => false,
    }
}
