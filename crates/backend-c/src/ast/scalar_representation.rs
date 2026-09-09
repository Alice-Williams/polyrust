//! Closed storage widths for the measured C ABI, independent of host usize.
use super::CScalarType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CIntegerWidth {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl CIntegerWidth {
    pub const fn bits(self) -> u32 {
        match self {
            Self::Eight => 8,
            Self::Sixteen => 16,
            Self::ThirtyTwo => 32,
            Self::SixtyFour => 64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CScalarRepresentation {
    Bool,
    Signed(CIntegerWidth),
    Unsigned(CIntegerWidth),
    Binary64,
}

impl CScalarType {
    pub const fn representation(self) -> CScalarRepresentation {
        use CIntegerWidth::{Eight, Sixteen, SixtyFour, ThirtyTwo};
        use CScalarRepresentation::{Binary64, Bool, Signed, Unsigned};
        match self {
            Self::Bool => Bool,
            Self::PlainChar | Self::I8 => Signed(Eight),
            Self::U8 => Unsigned(Eight),
            Self::I16 => Signed(Sixteen),
            Self::U16 => Unsigned(Sixteen),
            Self::Int | Self::I32 => Signed(ThirtyTwo),
            Self::U32 => Unsigned(ThirtyTwo),
            Self::I64 => Signed(SixtyFour),
            Self::U64 | Self::Size => Unsigned(SixtyFour),
            Self::F64 => Binary64,
        }
    }
}
