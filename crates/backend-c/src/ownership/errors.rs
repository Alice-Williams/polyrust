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
    InvalidCountedLoop,
    SharedLoopCounter,
    InvalidLoopMutation,
    LoopAddressEscape,
    MissingLoopStep,
    RepeatedLoopStep,
    ExpectedNumericValue,
    UnprovedSizeArithmetic,
    InvalidNumericSite,
    IndexOutOfBounds,
    UnprovedPointerExtent,
    UnprovedStorage,
    UninitializedStorage,
    InactiveUnionMember,
    ExpiredStorage,
    NullStorage,
    StorageTypeMismatch,
    AutomaticAddressEscape,
    UnprovedStorageCall,
    UnprovedAllocation,
    UnprovedAllocationSize,
    UnreleasedAllocation,
    InvalidAllocationRelease,
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
            Self::UnprovedAllocationSize => {
                f.write_str("C allocation needs positive checked bytes")
            }
            Self::UnreleasedAllocation => {
                f.write_str("C path retains an unaccounted live allocation")
            }
            Self::InvalidAllocationRelease => {
                f.write_str("C release lacks the exact live allocator base")
            }
            Self::UnprovedStorage => f.write_str("C storage provenance is not established"),
            Self::UninitializedStorage => f.write_str("C selected storage is not initialized"),
            Self::InactiveUnionMember => f.write_str("C union read lacks its actual active member"),
            Self::ExpiredStorage => {
                f.write_str("C automatic storage or pointer lifetime has expired")
            }
            Self::NullStorage => f.write_str("C null pointer cannot designate accessed storage"),
            Self::StorageTypeMismatch => {
                f.write_str("C pointer target differs from its backing storage type")
            }
            Self::AutomaticAddressEscape => {
                f.write_str("C automatic address may escape its lifetime")
            }
            Self::UnprovedStorageCall => {
                f.write_str("C call storage effects require body-derived evidence")
            }
            Self::UnprovedAllocation => {
                f.write_str("C allocation restoration lacks producing-call evidence")
            }
            Self::Context(value) => value.fmt(f),
            Self::Registry(value) => value.fmt(f),
            Self::Operator(value) => value.fmt(f),
            Self::IncompleteLayout => f.write_str("C object has no complete admitted layout"),
            Self::RecursiveLayout => f.write_str("C by-value layout graph is recursive"),
            Self::LayoutCapacity => f.write_str("C size/alignment calculation exceeds target Size"),
            Self::ExpectedNumericConstant => f.write_str("C expression is not a numeric constant"),
            Self::ExpectedNumericValue => f.write_str("C expression does not have a numeric value"),
            Self::IndexOutOfBounds => {
                f.write_str("C index is not proved inside its actual array extent")
            }
            Self::UnprovedPointerExtent => {
                f.write_str("C pointer index lacks authenticated storage extent")
            }
            Self::InvalidNumericSite => {
                f.write_str("C numeric fact does not match its actual graph site")
            }
            Self::UnprovedSizeArithmetic => {
                f.write_str("C size calculation is not proved nonwrapping")
            }
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
            Self::InvalidCountedLoop => {
                f.write_str("C loop does not match the checked counted grammar")
            }
            Self::SharedLoopCounter => f.write_str("C counted loops cannot share a counter"),
            Self::InvalidLoopMutation => {
                f.write_str("C loop counter has an unowned or noncanonical write")
            }
            Self::LoopAddressEscape => {
                f.write_str("C counted-loop counter or bound address is exposed")
            }
            Self::MissingLoopStep => {
                f.write_str("C continuing path does not execute its counted step")
            }
            Self::RepeatedLoopStep => {
                f.write_str("C iteration path executes its counted step more than once")
            }
        }
    }
}
impl std::error::Error for CSafetyError {}
