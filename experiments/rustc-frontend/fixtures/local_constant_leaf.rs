//! Local definitions remain owned by the separately compiled source crate.
pub fn identity(value: i64, flag: bool) -> i64 {
    const FALLBACK: i64 = -(1_i64 << 53) - 1;
    if flag { value } else { FALLBACK }
}
