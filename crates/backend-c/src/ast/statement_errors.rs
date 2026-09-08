//! Local statement categories and owner relationships.

use super::{CExpressionError, CInitializerError, CRegistryError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CStatementError {
    Expression(CExpressionError),
    Initializer(CInitializerError),
    Registry(CRegistryError),
    WrongScope,
    ConstRequiresInitializer,
    NotModifiable,
    IncompleteAggregate,
    TypeMismatch,
    ReturnCategory,
    ExpectedIntegerSwitch,
    EmptyCaseList,
    InvalidCountedProgress,
    LabelBeforeDeclaration,
}

impl From<CExpressionError> for CStatementError {
    fn from(value: CExpressionError) -> Self {
        Self::Expression(value)
    }
}
impl From<CInitializerError> for CStatementError {
    fn from(value: CInitializerError) -> Self {
        Self::Initializer(value)
    }
}
impl From<CRegistryError> for CStatementError {
    fn from(value: CRegistryError) -> Self {
        Self::Registry(value)
    }
}
impl std::fmt::Display for CStatementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expression(value) => value.fmt(f),
            Self::Initializer(value) => value.fmt(f),
            Self::Registry(value) => value.fmt(f),
            Self::WrongScope => {
                f.write_str("C statement/block does not belong to this function or lexical scope")
            }
            Self::ConstRequiresInitializer => {
                f.write_str("C const automatic storage requires an initializer")
            }
            Self::NotModifiable => {
                f.write_str("C assignment requires a non-array place without const subobjects")
            }
            Self::IncompleteAggregate => {
                f.write_str("C aggregate mutation requires its complete definition")
            }
            Self::TypeMismatch => f.write_str("C assignment or return type does not match"),
            Self::ReturnCategory => {
                f.write_str("C return value presence must match the exact prototype")
            }
            Self::ExpectedIntegerSwitch => f.write_str("C switch requires an integer expression"),
            Self::EmptyCaseList => f.write_str("C switch arm requires at least one case constant"),
            Self::InvalidCountedProgress => {
                f.write_str("C counted loop requires same-scope Size counter and const Size bound")
            }
            Self::LabelBeforeDeclaration => {
                f.write_str("C17 labels cannot precede a bare declaration")
            }
        }
    }
}
impl std::error::Error for CStatementError {}
