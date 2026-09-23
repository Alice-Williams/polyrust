//! Original constant facts are descriptive, never inferred target authority.
use super::RustScalarKind;

/// A source constant's kind and exact value cannot disagree structurally.
///
/// F64Bits preserves representation, including signed zero. This descriptive
/// data does not itself authenticate rustc or admit a target/source feature.
/// Char uses Rust's scalar invariant, not an unchecked integer/code unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RustConstantValue {
    Bool(bool),
    I32(i32),
    I64(i64),
    F64Bits(u64),
    Char(char),
}

impl RustConstantValue {
    pub fn kind(self) -> RustScalarKind {
        match self {
            Self::Bool(_) => RustScalarKind::Bool,
            Self::I32(_) => RustScalarKind::I32,
            Self::I64(_) => RustScalarKind::I64,
            Self::F64Bits(_) => RustScalarKind::F64,
            Self::Char(_) => RustScalarKind::Char,
        }
    }
}
