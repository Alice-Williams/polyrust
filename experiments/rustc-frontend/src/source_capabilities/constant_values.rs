//! Closed compiler-evaluated constant domain, independent of source literals.
//! Adding a literal kind cannot expand any constant capability implicitly.
use portable_binary64::FiniteBinary64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalarConstantValue {
    I32(i32),
    I64(i64),
    Bool(bool),
    F64(FiniteBinary64),
}
