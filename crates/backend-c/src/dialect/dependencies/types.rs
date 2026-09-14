//! Type spelling and completeness edges retain aliases and nominal owners.

use super::{CFileDependencies, CTagDependency, CTypeRequirement};
use crate::ast::{CFunctionType, CObjectType, CObjectTypeKind, CPointerTarget, CReturnType};

impl CFileDependencies {
    pub(super) fn object_type(&mut self, ty: &CObjectType, need: CTypeRequirement) {
        match ty.kind() {
            CObjectTypeKind::Scalar(scalar) => self.headers.extend(scalar.header()),
            CObjectTypeKind::Known(known) => {
                self.headers.insert(known.header());
            }
            CObjectTypeKind::Struct(value) => self.tag(CTagDependency::Struct(value.clone()), need),
            CObjectTypeKind::Union(value) => self.tag(CTagDependency::Union(value.clone()), need),
            // ISO C17 cannot forward-declare an enum tag.
            CObjectTypeKind::Enum(value) => self.tag(
                CTagDependency::Enum(value.clone()),
                CTypeRequirement::Complete,
            ),
            CObjectTypeKind::Typedef(alias) => {
                self.aliases.insert(alias.clone());
                self.object_type(alias.target(), need);
            }
            CObjectTypeKind::Pointer(CPointerTarget::Void(_)) => {}
            CObjectTypeKind::Pointer(CPointerTarget::Object(pointee)) => {
                self.object_type(pointee, CTypeRequirement::Declaration);
            }
            CObjectTypeKind::Pointer(CPointerTarget::Function(signature)) => {
                self.signature(signature, CTypeRequirement::Declaration);
            }
            // Even a pointer to an array needs the complete array element type.
            CObjectTypeKind::Array { element, .. } => {
                self.object_type(element, CTypeRequirement::Complete)
            }
        }
    }

    pub(super) fn signature(&mut self, signature: &CFunctionType, need: CTypeRequirement) {
        if let CReturnType::Value(result) = signature.return_type() {
            self.object_type(result.declared_type(), need);
        }
        for parameter in signature.parameters() {
            self.object_type(parameter.declared_type(), need);
        }
    }
}
