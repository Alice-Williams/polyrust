//! Exact source value identity stays distinct from its C integer representation.
use portable_backend_c::ast::CScalarConstantValue as Target;
use portable_binary64::Binary64Sign;
use portable_codegen::RustConstantValue as Original;

pub(super) fn matches(original: Original, target: &Target) -> bool {
    match (original, target) {
        (Original::Bool(left), Target::Bool(right)) => left == *right,
        (Original::I32(left), Target::I32(right)) => left == *right,
        (Original::I64(left), Target::I64(right)) => left == *right,
        (Original::Char(left), Target::U32(right)) => u32::from(left) == *right,
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

pub(super) fn encoded_value(value: Original) -> String {
    use super::serialization::quote;
    match value {
        Original::Bool(value) => value.to_string(),
        Original::I32(value) => quote(&value.to_string()),
        Original::I64(value) => quote(&value.to_string()),
        Original::F64Bits(value) => quote(&format!("0x{value:016x}")),
        Original::Char(value) => quote(&u32::from(value).to_string()),
    }
}
