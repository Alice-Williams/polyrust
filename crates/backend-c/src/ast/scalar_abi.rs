//! The measured Linux x86_64 C17 scalar model, not expression safety proof.
use super::CScalarType;

/// Only these five arithmetic ranks survive promotion on the pinned ABI.
/// Keeping this separate prevents a new scalar from reaching a default result.
#[derive(Clone, Copy)]
enum Promoted {
    Int,
    U32,
    I64,
    U64,
    F64,
}

impl Promoted {
    const fn scalar(self) -> CScalarType {
        match self {
            Self::Int => CScalarType::Int,
            Self::U32 => CScalarType::U32,
            Self::I64 => CScalarType::I64,
            Self::U64 => CScalarType::U64,
            Self::F64 => CScalarType::F64,
        }
    }
}

impl CScalarType {
    /// Collapse typedef-compatible scalar spellings without changing origins.
    pub const fn abi_identity(self) -> Self {
        match self {
            Self::I32 => Self::Int,
            Self::Size => Self::U64,
            Self::Bool
            | Self::PlainChar
            | Self::Int
            | Self::I8
            | Self::U8
            | Self::I16
            | Self::U16
            | Self::U32
            | Self::I64
            | Self::U64
            | Self::F64 => self,
        }
    }

    pub fn is_compatible_with(self, other: Self) -> bool {
        self.abi_identity() == other.abi_identity()
    }

    /// C integer promotion; floating values have no integer promotion.
    pub const fn integer_promotion(self) -> Option<Self> {
        match self.promoted() {
            Promoted::F64 => None,
            integer => Some(integer.scalar()),
        }
    }

    const fn promoted(self) -> Promoted {
        match self {
            Self::Bool
            | Self::PlainChar
            | Self::I8
            | Self::U8
            | Self::I16
            | Self::U16
            | Self::Int
            | Self::I32 => Promoted::Int,
            Self::U32 => Promoted::U32,
            Self::I64 => Promoted::I64,
            Self::U64 | Self::Size => Promoted::U64,
            Self::F64 => Promoted::F64,
        }
    }

    /// Actual C result type; range/UB/portable-semantic proof remains separate.
    pub const fn usual_arithmetic_conversion(self, other: Self) -> Self {
        use Promoted::{F64, I64, Int, U32, U64};
        match (self.promoted(), other.promoted()) {
            (F64, _) | (_, F64) => Self::F64,
            (U64, _) | (_, U64) => Self::U64,
            (I64, _) | (_, I64) => Self::I64,
            (U32, _) | (_, U32) => Self::U32,
            (Int, Int) => Self::Int,
        }
    }
}
