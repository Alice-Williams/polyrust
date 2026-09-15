//! Separate package: constant reads stay local to their compiler definition.
const FALLBACK: i64 = -(1_i64 << 53) - 1;
pub fn identity(value: i64, flag: bool) -> i64 {
    if flag { value } else { FALLBACK }
}
