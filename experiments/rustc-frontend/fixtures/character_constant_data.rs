//! Constant-only Unicode producer: original scalar identities, not UTF-16.
pub const ZERO: char = '\0';
pub const ASCII: char = 'A';
pub const SAME_INTEGER: i32 = 65;
pub const BEFORE_SURROGATES: char = '\u{d7ff}';
pub const AFTER_SURROGATES: char = '\u{e000}';
pub const NONCHARACTER: char = '\u{ffff}';
pub const SUPPLEMENTARY: char = '\u{10000}';
/// Maximum Unicode scalar 🦀; changes must invalidate dependent packages.
pub const MAXIMUM: char = '\u{10ffff}';
pub const COMPUTED: char = match char::from_u32(0x1f980) {
    Some(value) => value,
    None => panic!("valid constant scalar"),
};
pub const COPIED: char = MAXIMUM;
pub use MAXIMUM as PUBLIC_ALIAS;
