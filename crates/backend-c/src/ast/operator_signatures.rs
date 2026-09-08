//! C operator typing, separate from range safety and portable semantics.

use super::CScalarType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CUnaryOperator {
    LogicalNot,
    BitNot,
    Negate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CBinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitOr,
    BitXor,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum COperatorError {
    ExpectedInteger,
    ExpectedBool,
}

impl std::fmt::Display for COperatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ExpectedInteger => "C operator requires integer operands",
            Self::ExpectedBool => "C logical operands require explicit Bool conversion",
        })
    }
}

impl std::error::Error for COperatorError {}

impl CUnaryOperator {
    pub fn result_type(self, operand: CScalarType) -> Result<CScalarType, COperatorError> {
        match self {
            Self::LogicalNot => {
                require_bool(operand)?;
                Ok(CScalarType::Int)
            }
            Self::BitNot => require_integer(operand),
            Self::Negate => Ok(operand.integer_promotion().unwrap_or(CScalarType::F64)),
        }
    }
}

impl CBinaryOperator {
    pub fn result_type(
        self,
        left: CScalarType,
        right: CScalarType,
    ) -> Result<CScalarType, COperatorError> {
        match self {
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide => {
                Ok(left.usual_arithmetic_conversion(right))
            }
            Self::Remainder | Self::BitAnd | Self::BitOr | Self::BitXor => {
                require_integer(left)?;
                require_integer(right)?;
                Ok(left.usual_arithmetic_conversion(right))
            }
            Self::ShiftLeft | Self::ShiftRight => {
                let result = require_integer(left)?;
                require_integer(right)?;
                Ok(result)
            }
            Self::Equal
            | Self::NotEqual
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual => Ok(CScalarType::Int),
            Self::LogicalAnd | Self::LogicalOr => {
                require_bool(left)?;
                require_bool(right)?;
                Ok(CScalarType::Int)
            }
        }
    }
}

fn require_integer(value: CScalarType) -> Result<CScalarType, COperatorError> {
    value
        .integer_promotion()
        .ok_or(COperatorError::ExpectedInteger)
}

fn require_bool(value: CScalarType) -> Result<(), COperatorError> {
    if value == CScalarType::Bool {
        Ok(())
    } else {
        Err(COperatorError::ExpectedBool)
    }
}
