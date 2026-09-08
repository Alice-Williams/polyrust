//! Authenticate every nominal reference, including nested callback signatures.

use std::collections::BTreeSet;

use super::super::{CFunctionType, CObjectType, CObjectTypeKind, CPointerTarget, CReturnType};
use super::{CAggregateRef, CRegistry, CRegistryError};

impl CRegistry {
    pub fn check_signature(&self, signature: &CFunctionType) -> Result<(), CRegistryError> {
        if let CReturnType::Value(value) = signature.return_type() {
            self.check_type(value.declared_type())?;
        }
        for parameter in signature.parameters() {
            self.check_type(parameter.declared_type())?;
        }
        Ok(())
    }

    pub fn check_type(&self, ty: &CObjectType) -> Result<(), CRegistryError> {
        let mut pending = vec![ty];
        let mut aliases = BTreeSet::new();
        while let Some(ty) = pending.pop() {
            match ty.kind() {
                CObjectTypeKind::Scalar(_) | CObjectTypeKind::Known(_) => {}
                CObjectTypeKind::Struct(value) => {
                    self.check_aggregate(&CAggregateRef::Struct(value.clone()))?
                }
                CObjectTypeKind::Union(value) => {
                    self.check_aggregate(&CAggregateRef::Union(value.clone()))?
                }
                CObjectTypeKind::Enum(value) => self.check_enum(value)?,
                CObjectTypeKind::Typedef(value) => {
                    self.check_scope(&value.identity.scope)?;
                    if !self.typedefs.contains(value) {
                        return Err(CRegistryError::UnregisteredReference);
                    }
                    if aliases.insert(value) {
                        pending.push(value.target());
                    }
                }
                CObjectTypeKind::Array { element, .. } => pending.push(element),
                CObjectTypeKind::Pointer(target) => match target {
                    CPointerTarget::Void(_) => {}
                    CPointerTarget::Object(value) => pending.push(value),
                    CPointerTarget::Function(signature) => {
                        if let CReturnType::Value(value) = signature.return_type() {
                            pending.push(value.declared_type());
                        }
                        pending.extend(
                            signature
                                .parameters()
                                .iter()
                                .map(|value| value.declared_type()),
                        );
                    }
                },
            }
        }
        Ok(())
    }
}
