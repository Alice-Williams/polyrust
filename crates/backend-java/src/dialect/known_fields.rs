//! Java dialect: known fields.

use super::member_names::JavaMemberName;
use crate::ast::{JavaKnownType, JavaPrimitive, JavaType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaKnownField {
    IntegerMinValue,
    IntegerMaxValue,
    LongMinValue,
    LongMaxValue,
    StandardCharsetsUtf8,
    CodingErrorReport,
}

impl JavaKnownField {
    pub fn owner(self) -> JavaKnownType {
        match self {
            Self::IntegerMinValue | Self::IntegerMaxValue => JavaKnownType::Integer,
            Self::LongMinValue | Self::LongMaxValue => JavaKnownType::Long,
            Self::StandardCharsetsUtf8 => JavaKnownType::StandardCharsets,
            Self::CodingErrorReport => JavaKnownType::CodingErrorAction,
        }
    }

    pub fn member(self) -> JavaMemberName {
        match self {
            Self::IntegerMinValue | Self::LongMinValue => JavaMemberName::MinValue,
            Self::IntegerMaxValue | Self::LongMaxValue => JavaMemberName::MaxValue,
            Self::StandardCharsetsUtf8 => JavaMemberName::Utf8,
            Self::CodingErrorReport => JavaMemberName::Report,
        }
    }

    pub fn ty(self) -> JavaType {
        match self {
            Self::IntegerMinValue | Self::IntegerMaxValue => {
                JavaType::primitive(JavaPrimitive::Int)
            }
            Self::LongMinValue | Self::LongMaxValue => JavaType::primitive(JavaPrimitive::Long),
            Self::StandardCharsetsUtf8 => JavaType::known(JavaKnownType::Charset),
            Self::CodingErrorReport => JavaType::known(JavaKnownType::CodingErrorAction),
        }
    }
}
