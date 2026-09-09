//! Derived ranges and modulo classification do not carry target AST authority.
use super::{floating::FloatRange, integer::IntegerRange};
use crate::ast::CScalarType;
use crate::ownership::{CSafetyError as E, constants::CNumber};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ownership) enum CScalarRange {
    Integer(IntegerRange),
    Float(FloatRange),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ownership) enum CTransfer {
    // This primitive operation has no integer modulo loss. It does not
    // certify operand provenance, an entire expression or floating finiteness.
    NonWrapping(CScalarRange),
    MayWrap(CScalarRange),
}
impl CTransfer {
    pub(in crate::ownership) const fn range(self) -> CScalarRange {
        match self {
            Self::NonWrapping(range) | Self::MayWrap(range) => range,
        }
    }
    pub(super) fn with_range(self, range: CScalarRange) -> Self {
        match self {
            Self::NonWrapping(_) => Self::NonWrapping(range),
            Self::MayWrap(_) => Self::MayWrap(range),
        }
    }
}

impl CScalarRange {
    pub(in crate::ownership) fn exact(value: CNumber) -> Self {
        match value {
            CNumber::Integer(value) => Self::Integer(IntegerRange::exact(value)),
            CNumber::Double(value) => Self::Float(FloatRange::exact(value)),
        }
    }
    pub(in crate::ownership) fn full(ty: CScalarType) -> Result<Self, E> {
        if ty == CScalarType::F64 {
            Ok(Self::Float(FloatRange::unknown()))
        } else {
            IntegerRange::full(ty).map(Self::Integer)
        }
    }
    pub(in crate::ownership) fn ty(self) -> CScalarType {
        match self {
            Self::Integer(value) => value.ty(),
            Self::Float(_) => CScalarType::F64,
        }
    }
    pub(in crate::ownership) fn exact_value(self) -> Option<CNumber> {
        match self {
            Self::Integer(value) => value.exact_value().map(CNumber::Integer),
            Self::Float(value) => value.exact_value().map(CNumber::Double),
        }
    }
    pub(super) fn integer(self) -> Result<IntegerRange, E> {
        match self {
            Self::Integer(value) => Ok(value),
            Self::Float(_) => Err(E::ExpectedIntegerConstant),
        }
    }
    pub(in crate::ownership) fn join(self, right: Self) -> Result<Self, E> {
        match (self, right) {
            (Self::Integer(left), Self::Integer(right)) => left.join(right).map(Self::Integer),
            (Self::Float(left), Self::Float(right)) => Ok(Self::Float(left.join(right))),
            _ => Err(E::InvalidNumericRange),
        }
    }
    pub(in crate::ownership) fn truth(self) -> Option<bool> {
        match self {
            Self::Integer(value) => value.truth(),
            Self::Float(value) => value.truth(),
        }
    }
    #[cfg(test)]
    pub(in crate::ownership) fn contains(self, value: CNumber) -> bool {
        match (self, value) {
            (Self::Integer(range), CNumber::Integer(value)) => {
                range.ty() == value.ty() && range.contains(value.value())
            }
            (Self::Float(range), CNumber::Double(value)) => range.contains(value),
            _ => false,
        }
    }
}
