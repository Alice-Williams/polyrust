//! Local construction errors, never replacements for contextual diagnostics.

use super::{COperatorError, CRegistryError, CTypeError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CExpressionError {
    ExpectedFunctionPointer,
    ExpectedValueCall,
    ExpectedEffectCall,
    ArityMismatch { expected: usize, actual: usize },
    Registry(CRegistryError),
    Type(CTypeError),
    Operator(COperatorError),
    ExpectedArithmetic,
    ExpectedBool,
    ExpectedAggregate,
    ExpectedObjectPointer,
    ExpectedArray,
    ExpectedOwningSlotPointer,
    ArrayReadRequiresExplicitAddress,
    TypeMismatch,
    IncompleteEnum,
    InvalidPointerConversion,
}

impl From<CRegistryError> for CExpressionError {
    fn from(value: CRegistryError) -> Self {
        Self::Registry(value)
    }
}
impl From<CTypeError> for CExpressionError {
    fn from(value: CTypeError) -> Self {
        Self::Type(value)
    }
}
impl From<COperatorError> for CExpressionError {
    fn from(value: COperatorError) -> Self {
        Self::Operator(value)
    }
}
impl std::fmt::Display for CExpressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedFunctionPointer => {
                f.write_str("C indirect call requires a function pointer")
            }
            Self::ExpectedValueCall => f.write_str("C void call cannot produce a value"),
            Self::ExpectedEffectCall => {
                f.write_str("C nonvoid call requires a value or explicit Discard")
            }
            Self::ArityMismatch { expected, actual } => {
                write!(f, "C call expects {expected} arguments, received {actual}")
            }
            Self::Registry(value) => value.fmt(f),
            Self::Type(value) => value.fmt(f),
            Self::Operator(value) => value.fmt(f),
            Self::ExpectedArithmetic => f.write_str("C expression requires arithmetic operands"),
            Self::ExpectedBool => f.write_str("C condition requires explicit Bool conversion"),
            Self::ExpectedAggregate => {
                f.write_str("C member access requires its registered aggregate")
            }
            Self::ExpectedObjectPointer => f.write_str("C place requires an object pointer"),
            Self::ExpectedArray => f.write_str("C indexed array place requires an array"),
            Self::ExpectedOwningSlotPointer => {
                f.write_str("C SameSlot requires matching owning-slot address types")
            }
            Self::ArrayReadRequiresExplicitAddress => {
                f.write_str("C arrays require explicit element address formation")
            }
            Self::TypeMismatch => f.write_str("C expression types do not match"),
            Self::IncompleteEnum => {
                f.write_str("C enum arithmetic requires a registered definition")
            }
            Self::InvalidPointerConversion => {
                f.write_str("C pointer conversion would change category or lose qualifiers")
            }
        }
    }
}
impl std::error::Error for CExpressionError {}
