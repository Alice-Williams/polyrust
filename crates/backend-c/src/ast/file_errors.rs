//! Authenticated source grouping and declaration-context errors.

use super::{CExpressionError, CInitializerError, CRegistryError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CFileError {
    ExpectedStaticInitializer,
    Registry(CRegistryError),
    Expression(CExpressionError),
    Initializer(CInitializerError),
    WrongFile,
    WrongFileRole,
    InvalidLinkage,
    IncompleteDefinition,
    ParameterInventory,
    InvalidFunctionRoot,
    ExpectedIntegerConstantExpression,
}
impl From<CRegistryError> for CFileError {
    fn from(value: CRegistryError) -> Self {
        Self::Registry(value)
    }
}
impl From<CExpressionError> for CFileError {
    fn from(value: CExpressionError) -> Self {
        Self::Expression(value)
    }
}
impl From<CInitializerError> for CFileError {
    fn from(value: CInitializerError) -> Self {
        Self::Initializer(value)
    }
}
impl std::fmt::Display for CFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedStaticInitializer => {
                f.write_str("C file storage requires a constant initializer")
            }
            Self::Registry(value) => value.fmt(f),
            Self::Expression(value) => value.fmt(f),
            Self::Initializer(value) => value.fmt(f),
            Self::WrongFile => f.write_str("C item does not belong to this registered file"),
            Self::WrongFileRole => f.write_str(
                "C file role cannot contain this definition or supply production symbols",
            ),
            Self::InvalidLinkage => {
                f.write_str("C linkage is invalid for this declaration context")
            }
            Self::IncompleteDefinition => f.write_str(
                "C nominal definition requires its complete nonempty registry inventory",
            ),
            Self::ParameterInventory => f.write_str(
                "C function definition requires every registered parameter in prototype order",
            ),
            Self::InvalidFunctionRoot => {
                f.write_str("C function body must bind its own parentless root scope")
            }
            Self::ExpectedIntegerConstantExpression => {
                f.write_str("C static assertion requires an integer constant-expression tree")
            }
        }
    }
}
impl std::error::Error for CFileError {}
