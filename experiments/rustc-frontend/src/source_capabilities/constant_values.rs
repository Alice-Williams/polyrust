//! Closed compiler-evaluated constant domain, independent of source literals.
//! Adding a literal kind cannot expand any constant capability implicitly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalarConstantValue {
    I32(i32),
    I64(i64),
    Bool(bool),
}
