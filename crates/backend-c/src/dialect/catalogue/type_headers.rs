//! Dependencies of atomic C spellings, shared by lowering and linking.

use super::CHeader;
use crate::ast::{CKnownConstant, CKnownObject, CScalarType};

impl CScalarType {
    /// Bool is spelled `_Bool`; boolean literals are numeric 0/1, not macros.
    pub const fn header(self) -> Option<CHeader> {
        match self {
            Self::I8
            | Self::U8
            | Self::I16
            | Self::U16
            | Self::I32
            | Self::U32
            | Self::I64
            | Self::U64 => Some(CHeader::Stdint),
            Self::Size => Some(CHeader::Stddef),
            Self::Bool | Self::PlainChar | Self::Int | Self::F64 => None,
        }
    }
}

impl CKnownObject {
    pub const fn header(self) -> CHeader {
        match self {
            Self::File => CHeader::Stdio,
            Self::MaxAlign => CHeader::Stddef,
        }
    }
}

impl CKnownConstant {
    pub const fn header(self) -> CHeader {
        match self {
            Self::CharBit | Self::IntMin | Self::IntMax => CHeader::Limits,
            Self::I32Min
            | Self::I32Max
            | Self::U32Max
            | Self::I64Min
            | Self::I64Max
            | Self::U64Max
            | Self::SizeMax => CHeader::Stdint,
            Self::FloatRadix
            | Self::DoubleMantissaDigits
            | Self::DoubleMinExponent
            | Self::DoubleMaxExponent
            | Self::DoubleHasSubnormals
            | Self::FloatEvaluationMethod => CHeader::Float,
            Self::EndOfFile | Self::StandardInput | Self::StandardOutput | Self::StandardError => {
                CHeader::Stdio
            }
        }
    }
}
