//! Library-owned object identities cannot be synthesized from generated names.

use super::{CObjectType, CObjectTypeKind, CTypeError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CKnownObject {
    /// Borrowed streams only; the backend never stores or copies a FILE value.
    File,
    /// Complete library-owned alignment type on the pinned C ABI.
    MaxAlign,
}

impl CObjectType {
    /// A local storage-category check, not a nominal-completeness certificate.
    /// Typedef declarations may name FILE so callers can form borrowed pointers.
    /// Generated aggregates still require contextual completeness verification.
    pub fn require_storable(&self) -> Result<(), CTypeError> {
        let mut root = self;
        loop {
            match root.kind() {
                CObjectTypeKind::Typedef(alias) => root = alias.target(),
                CObjectTypeKind::Array { element, .. } => root = element,
                CObjectTypeKind::Known(CKnownObject::File) => {
                    return Err(CTypeError::KnownObjectRequiresPointer(CKnownObject::File));
                }
                CObjectTypeKind::Known(CKnownObject::MaxAlign)
                | CObjectTypeKind::Scalar(_)
                | CObjectTypeKind::Struct(_)
                | CObjectTypeKind::Union(_)
                | CObjectTypeKind::Enum(_)
                | CObjectTypeKind::Pointer(_) => return Ok(()),
            }
        }
    }
}
