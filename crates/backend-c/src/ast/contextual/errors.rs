//! Closed diagnostics, not caller-provided verification facts.

use super::super::{
    CExpressionError, CFileError, CInitializerError, CRegistryError, CStatementError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CContextError {
    Registry(CRegistryError),
    Expression(CExpressionError),
    Initializer(CInitializerError),
    Statement(CStatementError),
    File(CFileError),
    StoredStructureMismatch,
    IncompleteObject,
    RecursiveObject,
    MissingRegistrationOccurrence,
    DuplicateOccurrence,
    LinkageMismatch,
    OriginRoleMismatch,
    InvisibleBinding,
    WrongLexicalOwner,
    WrongControlTarget,
    InvalidCleanupExit,
    TraversalCapacity,
    UninitializedRead,
    MissingReturn,
    SwitchFallthrough,
    DuplicateCase,
}

macro_rules! context_error {
    ($source:ty, $variant:ident) => {
        impl From<$source> for CContextError {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    };
}
context_error!(CRegistryError, Registry);
context_error!(CExpressionError, Expression);
context_error!(CInitializerError, Initializer);
context_error!(CStatementError, Statement);
context_error!(CFileError, File);

impl std::fmt::Display for CContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registry(value) => value.fmt(f),
            Self::Expression(value) => value.fmt(f),
            Self::Initializer(value) => value.fmt(f),
            Self::Statement(value) => value.fmt(f),
            Self::File(value) => value.fmt(f),
            Self::StoredStructureMismatch => f.write_str(
                "stored C node differs from its independently reconstructed local structure",
            ),
            Self::IncompleteObject => {
                f.write_str("C by-value object requires a complete definition")
            }
            Self::RecursiveObject => f.write_str("C object has a recursive by-value layout"),
            Self::MissingRegistrationOccurrence => {
                f.write_str("C declaration tree does not cover its authoritative registrations")
            }
            Self::DuplicateOccurrence => {
                f.write_str("C identity occurs more than once in its defining role")
            }
            Self::LinkageMismatch => f.write_str("C prototype and definition linkage disagree"),
            Self::OriginRoleMismatch => {
                f.write_str("C definition origin is incompatible with its owning file role")
            }
            Self::InvisibleBinding => f.write_str("C binding is not visible at this use"),
            Self::WrongLexicalOwner => {
                f.write_str("C binding or block has a different lexical owner")
            }
            Self::WrongControlTarget => {
                f.write_str("C break/continue does not target the innermost control")
            }
            Self::InvalidCleanupExit => {
                f.write_str("C cleanup exit is not forward to a declaration-safe enclosing scope")
            }
            Self::TraversalCapacity => {
                f.write_str("C contextual traversal exceeds addressable capacity")
            }
            Self::UninitializedRead => {
                f.write_str("C local storage is not initialized on every incoming path")
            }
            Self::MissingReturn => {
                f.write_str("C nonvoid function has a reachable end without a return")
            }
            Self::SwitchFallthrough => {
                f.write_str("C switch arm has a reachable implicit fallthrough")
            }
            Self::DuplicateCase => {
                f.write_str("C switch constants coincide after integer promotion")
            }
        }
    }
}
impl std::error::Error for CContextError {}
