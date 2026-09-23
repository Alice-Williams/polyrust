//! Mixed constant owner, private helper and evaluated constant contexts.
pub use middle::MAXIMUM as RENAMED;
pub use middle::{ASCII, MAXIMUM};
/// Root-owned scalar, not an alias.
pub const OWN: char = '\u{1f980}';
const PRIVATE: char = '\u{e000}';
struct Holder;
impl Holder {
    const VALUE: char = '\u{ffff}';
}
fn private_read() -> char {
    PRIVATE
}
pub fn read_zero() -> char {
    middle::ZERO
}
pub fn read_ascii() -> char {
    middle::ASCII
}
pub fn read_same_integer() -> i32 {
    middle::SAME_INTEGER
}
pub fn read_before_surrogates() -> char {
    middle::BEFORE_SURROGATES
}
pub fn read_after_surrogates() -> char {
    middle::AFTER_SURROGATES
}
pub fn read_noncharacter() -> char {
    middle::NONCHARACTER
}
pub fn read_supplementary() -> char {
    middle::SUPPLEMENTARY
}
pub fn read_maximum() -> char {
    middle::MAXIMUM
}
pub fn read_computed() -> char {
    middle::COMPUTED
}
pub fn read_copied() -> char {
    middle::COPIED
}
pub fn read_other_same() -> char {
    middle::OTHER_SAME
}
pub fn read_other_maximum() -> char {
    middle::OTHER_MAXIMUM
}
pub fn read_alias() -> char {
    middle::nested::AGAIN
}
pub fn read_own() -> char {
    OWN
}
pub fn read_private() -> char {
    private_read()
}
pub fn read_local() -> char {
    const VALUE: char = match char::from_u32(0x10000) {
        Some(value) => value,
        None => panic!("valid constant scalar"),
    };
    VALUE
}
pub fn read_inherent() -> char {
    Holder::VALUE
}
pub fn read_comparison() -> bool {
    middle::ASCII == middle::OTHER_SAME
}
