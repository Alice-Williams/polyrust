//! Complete original field/variant facts, kept separate from placement identity.

use super::{
    RustCanonicalInstanceKey, RustDeclarationId, RustInstanceDefinitionRole as Role,
    RustInstanceIdentityError, check_distinct,
};

/// Descriptive variant and its sole original payload field; not member authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustResultVariantFacts {
    pub variant: RustDeclarationId,
    pub payload: RustDeclarationId,
}

/// Fixed-size facts to reconcile on every encounter of the same instance key.
///
/// The core root is a source anchor, never the generated type package owner.
/// Constructors check consistency only, not that the IDs actually denote core,
/// an enum, a variant, a field or a no-drop type. That requires a rustc witness.
/// A different but internally consistent field ID remains descriptive data and
/// must be rejected by the graph when it conflicts with authenticated facts.
///
/// ```compile_fail,E0451
/// use portable_codegen::{RustCanonicalInstanceFacts, RustCanonicalInstanceKey,
///     RustDeclarationId, RustResultVariantFacts};
/// let id = |definition_path_hash| RustDeclarationId { crate_id: 1, definition_path_hash };
/// let key = RustCanonicalInstanceKey::i32_try_from_int_error_result(id(1), id(2)).unwrap();
/// let facts = RustCanonicalInstanceFacts {
///     key, core_root: id(0),
///     ok: RustResultVariantFacts { variant: id(3), payload: id(4) },
///     err: RustResultVariantFacts { variant: id(5), payload: id(6) },
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustCanonicalInstanceFacts {
    key: RustCanonicalInstanceKey,
    core_root: RustDeclarationId,
    ok: RustResultVariantFacts,
    err: RustResultVariantFacts,
}

impl RustCanonicalInstanceFacts {
    pub fn new(
        key: RustCanonicalInstanceKey,
        core_root: RustDeclarationId,
        ok: RustResultVariantFacts,
        err: RustResultVariantFacts,
    ) -> Result<Self, RustInstanceIdentityError> {
        let definitions = [
            (Role::CoreRoot, core_root),
            (Role::Result, key.result_definition()),
            (Role::Error, key.error_definition()),
            (Role::Ok, ok.variant),
            (Role::Err, err.variant),
            (Role::OkPayload, ok.payload),
            (Role::ErrPayload, err.payload),
        ];
        for (index, &left) in definitions.iter().enumerate() {
            for &right in &definitions[index + 1..] {
                check_distinct(left, right)?;
            }
        }
        Ok(Self {
            key,
            core_root,
            ok,
            err,
        })
    }

    pub fn key(self) -> RustCanonicalInstanceKey {
        self.key
    }

    pub fn core_root(self) -> RustDeclarationId {
        self.core_root
    }

    pub fn ok(self) -> RustResultVariantFacts {
        self.ok
    }

    pub fn err(self) -> RustResultVariantFacts {
        self.err
    }
}
