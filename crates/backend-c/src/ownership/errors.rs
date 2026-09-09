//! Closed safety diagnostics, without input-authored proof flags.
use crate::ast::{CContextError, COperatorError, CRegistryError};

/// Diagnostics do not expose checked numeric/layout fact constructors.
///
/// ```compile_fail
/// use portable_backend_c::ownership::layout::CLayout;
/// let forged = CLayout { size: 1, alignment: 1 };
/// ```
///
/// ```compile_fail
/// use portable_backend_c::ownership::constants::CInteger;
/// use portable_backend_c::ast::CScalarType;
/// let forged = CInteger::checked(CScalarType::I8, 1000);
/// ```
///
/// ```compile_fail
/// use portable_backend_c::ownership::ranges::integer::IntegerRange;
/// let forged = IntegerRange { ty: todo!(), min: 1, max: 0 };
/// ```
///
/// ```compile_fail
/// use portable_backend_c::ownership::ranges::floating::FloatRange;
/// let forged = FloatRange { kind: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSafetyError {
    Context(CContextError),
    Registry(CRegistryError),
    Operator(COperatorError),
    IncompleteLayout,
    RecursiveLayout,
    LayoutCapacity,
    ExpectedNumericConstant,
    ExpectedIntegerConstant,
    IntegerRange,
    DivisionByZero,
    SignedOverflow,
    InvalidShift,
    FalseAssertion,
    UnsequencedCall,
    InvalidNumericRange,
}

impl From<CContextError> for CSafetyError {
    fn from(value: CContextError) -> Self {
        Self::Context(value)
    }
}
impl From<CRegistryError> for CSafetyError {
    fn from(value: CRegistryError) -> Self {
        Self::Registry(value)
    }
}
impl From<COperatorError> for CSafetyError {
    fn from(value: COperatorError) -> Self {
        Self::Operator(value)
    }
}
impl std::fmt::Display for CSafetyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Context(value) => value.fmt(f),
            Self::Registry(value) => value.fmt(f),
            Self::Operator(value) => value.fmt(f),
            Self::IncompleteLayout => f.write_str("C object has no complete admitted layout"),
            Self::RecursiveLayout => f.write_str("C by-value layout graph is recursive"),
            Self::LayoutCapacity => f.write_str("C size/alignment calculation exceeds target Size"),
            Self::ExpectedNumericConstant => f.write_str("C expression is not a numeric constant"),
            Self::ExpectedIntegerConstant => f.write_str("C expression is not an integer constant"),
            Self::IntegerRange => {
                f.write_str("C numeric conversion is outside its proven integer range")
            }
            Self::DivisionByZero => f.write_str("C integer divisor is not proved nonzero"),
            Self::SignedOverflow => f.write_str("C signed arithmetic is not proved representable"),
            Self::InvalidShift => f.write_str("C shift operands and result are not proved valid"),
            Self::FalseAssertion => f.write_str("C static assertion evaluates to zero"),
            Self::UnsequencedCall => f.write_str(
                "C call must be a permitted full-expression root with call-free operands",
            ),
            Self::InvalidNumericRange => {
                f.write_str("C numeric range has unordered bounds or mismatched scalar types")
            }
        }
    }
}
impl std::error::Error for CSafetyError {}
