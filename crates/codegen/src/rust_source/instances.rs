//! Bounded source-instance descriptions. These are metadata, never certificates.

mod facts;
mod owner;

pub use facts::{RustCanonicalInstanceFacts, RustResultVariantFacts};
pub use owner::TargetPackageOwner;

use super::RustDeclarationId;

/// Closed normalized argument vocabulary; not general enum or generic support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RustCanonicalInstanceKind {
    /// Success is primitive I32; error is the zero-argument nominal declaration
    /// retained by the key. Only a compiler witness can authenticate that name.
    I32TryFromIntErrorResult,
}

/// Roles identify original compiler declarations, not target symbol spellings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RustInstanceDefinitionRole {
    CoreRoot,
    Result,
    Error,
    Ok,
    Err,
    OkPayload,
    ErrPayload,
}

/// A descriptive inconsistency, not a compiler diagnostic or validity proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RustInstanceIdentityError {
    DifferentCrates {
        first: RustInstanceDefinitionRole,
        second: RustInstanceDefinitionRole,
    },
    RepeatedDefinition {
        first: RustInstanceDefinitionRole,
        second: RustInstanceDefinitionRole,
    },
}

/// Fixed-size identity of a single selected normalized Rust instance.
///
/// Construction checks only same-crate/distinct-definition consistency. Callers
/// can describe fictitious definitions; no target or compiler authority follows.
/// The compiler adapter must independently authenticate every original role.
///
/// ```compile_fail,E0451
/// use portable_codegen::{RustCanonicalInstanceKey, RustDeclarationId};
/// let id = RustDeclarationId { crate_id: 1, definition_path_hash: 2 };
/// let key = RustCanonicalInstanceKey { result: id, error: id };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustCanonicalInstanceKey {
    result: RustDeclarationId,
    error: RustDeclarationId,
}

impl RustCanonicalInstanceKey {
    pub fn i32_try_from_int_error_result(
        result: RustDeclarationId,
        error: RustDeclarationId,
    ) -> Result<Self, RustInstanceIdentityError> {
        check_distinct(
            (RustInstanceDefinitionRole::Result, result),
            (RustInstanceDefinitionRole::Error, error),
        )?;
        Ok(Self { result, error })
    }

    pub fn kind(self) -> RustCanonicalInstanceKind {
        RustCanonicalInstanceKind::I32TryFromIntErrorResult
    }

    pub fn result_definition(self) -> RustDeclarationId {
        self.result
    }

    pub fn error_definition(self) -> RustDeclarationId {
        self.error
    }
}

fn check_distinct(
    (first, left): (RustInstanceDefinitionRole, RustDeclarationId),
    (second, right): (RustInstanceDefinitionRole, RustDeclarationId),
) -> Result<(), RustInstanceIdentityError> {
    if left.crate_id != right.crate_id {
        Err(RustInstanceIdentityError::DifferentCrates { first, second })
    } else if left == right {
        Err(RustInstanceIdentityError::RepeatedDefinition { first, second })
    } else {
        Ok(())
    }
}
