//! Alias expansion is derived from immutable registered targets, never flags.

use super::{CConstness, CObjectType, CObjectTypeKind, CPointerTarget};

impl CObjectType {
    /// Expand structural aliases without losing nominal identity. Callback
    /// signature wrappers retain source provenance alongside canonical types;
    /// Rust equality authenticates that provenance, not merely C compatibility.
    /// Nominal tags remain nominal: expanding an alias never unfolds a record.
    pub fn canonical(&self) -> Self {
        let mut root = self;
        let mut is_const = root.constness() == CConstness::Const;
        while let CObjectTypeKind::Typedef(alias) = root.kind() {
            root = alias.target();
            is_const |= root.constness() == CConstness::Const;
        }
        let value = match root.kind() {
            CObjectTypeKind::Scalar(value) => Self::scalar(*value),
            CObjectTypeKind::Struct(value) => Self::structure(value.clone()),
            CObjectTypeKind::Union(value) => Self::union(value.clone()),
            CObjectTypeKind::Enum(value) => Self::enumeration(value.clone()),
            CObjectTypeKind::Array { element, length } => Self::array(element.canonical(), *length),
            CObjectTypeKind::Pointer(target) => Self::pointer(match target {
                CPointerTarget::Void(qualifier) => CPointerTarget::Void(*qualifier),
                CPointerTarget::Object(value) => {
                    CPointerTarget::Object(Box::new(value.canonical()))
                }
                // Signature wrappers already contain canonical types.
                CPointerTarget::Function(value) => CPointerTarget::Function(value.clone()),
            }),
            CObjectTypeKind::Typedef(_) => unreachable!("the root alias chain was expanded"),
        };
        // with_constness forbids array qualification, including through aliases.
        value
            .with_constness(if is_const {
                CConstness::Const
            } else {
                CConstness::Unqualified
            })
            .expect("registered aliases cannot introduce an array qualifier")
    }
}
