//! Exact backend type matching, not C's implicit assignment conversions.

use super::{
    CFunctionType, CObjectType, CObjectTypeKind, CPointerTarget, CRegistry, CRegistryError,
    CReturnType,
};

impl CRegistry {
    /// Compare authenticated pointees while ignoring only their immediate
    /// qualification. Array qualification belongs to the element; qualification
    /// behind a nested pointer remains significant.
    pub(crate) fn pointee_types_match(
        &self,
        left: &CObjectType,
        right: &CObjectType,
    ) -> Result<bool, CRegistryError> {
        self.check_type(left)?;
        self.check_type(right)?;
        let left = left.canonical();
        let right = right.canonical();
        let (mut left, mut right) = (&left, &right);
        while let (
            CObjectTypeKind::Array {
                element: a,
                length: x,
            },
            CObjectTypeKind::Array {
                element: b,
                length: y,
            },
        ) = (left.kind(), right.kind())
        {
            if x != y {
                return Ok(false);
            }
            left = a;
            right = b;
        }
        self.types_match(
            &left.clone().without_top_level_const(),
            &right.clone().without_top_level_const(),
        )
    }

    /// Authenticate source aliases first, then compare their concrete types.
    /// ABI-compatible scalar spellings match; distinct nominal registrations do
    /// not. No qualifier addition/loss, decay or pointer conversion is implicit.
    pub fn types_match(
        &self,
        left: &CObjectType,
        right: &CObjectType,
    ) -> Result<bool, CRegistryError> {
        self.check_type(left)?;
        self.check_type(right)?;
        Ok(canonical_types_match(&left.canonical(), &right.canonical()))
    }

    pub fn signatures_match(
        &self,
        left: &CFunctionType,
        right: &CFunctionType,
    ) -> Result<bool, CRegistryError> {
        self.check_signature(left)?;
        self.check_signature(right)?;
        Ok(signatures_match(left, right))
    }
}

fn canonical_types_match(left: &CObjectType, right: &CObjectType) -> bool {
    if left.constness() != right.constness() {
        return false;
    }
    use CObjectTypeKind as T;
    match (left.kind(), right.kind()) {
        (T::Scalar(a), T::Scalar(b)) => a.is_compatible_with(*b),
        (T::Known(a), T::Known(b)) => a == b,
        (T::Struct(a), T::Struct(b)) => a == b,
        (T::Union(a), T::Union(b)) => a == b,
        (T::Enum(a), T::Enum(b)) => a == b,
        (
            T::Array {
                element: a,
                length: x,
            },
            T::Array {
                element: b,
                length: y,
            },
        ) => x == y && canonical_types_match(a, b),
        (T::Pointer(a), T::Pointer(b)) => match (a, b) {
            (CPointerTarget::Void(a), CPointerTarget::Void(b)) => a == b,
            (CPointerTarget::Object(a), CPointerTarget::Object(b)) => canonical_types_match(a, b),
            (CPointerTarget::Function(a), CPointerTarget::Function(b)) => signatures_match(a, b),
            _ => false,
        },
        _ => false,
    }
}

fn signatures_match(left: &CFunctionType, right: &CFunctionType) -> bool {
    let returns_match = match (left.return_type(), right.return_type()) {
        (CReturnType::Void, CReturnType::Void) => true,
        (CReturnType::Value(a), CReturnType::Value(b)) => canonical_types_match(a.ty(), b.ty()),
        _ => false,
    };
    returns_match
        && left.parameters().len() == right.parameters().len()
        && left
            .parameters()
            .iter()
            .zip(right.parameters())
            .all(|(a, b)| canonical_types_match(a.ty(), b.ty()))
}
