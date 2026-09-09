//! Requested owning-local roles; no registration establishes lifecycle evidence.
#[cfg(test)]
#[path = "../../tests/owner_contract_reconstruction.rs"]
mod tests;
use super::{CFunctionRef, CLocalRef, CRegistry, CRegistryError};
use crate::ast::{CConstness, CObjectType, CObjectTypeKind, CPointerTarget};

/// An ownership obligation over an existing authenticated local declaration.
/// This is not a live value, allocation, transfer or rendering certificate.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CLocalRef, COwnerSlotRef};
/// fn forge(local: CLocalRef) -> COwnerSlotRef { COwnerSlotRef { local } }
/// ```
///
/// ```compile_fail
/// use portable_backend_c::ast::{CLocalRef, COwnerSlotRef};
/// fn retarget(mut slot: COwnerSlotRef, local: CLocalRef) { slot.local = local; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct COwnerSlotRef {
    local: CLocalRef,
}

impl COwnerSlotRef {
    pub fn local(&self) -> &CLocalRef {
        &self.local
    }
}

impl CRegistry {
    pub fn register_local_owner(
        &mut self,
        local: &CLocalRef,
    ) -> Result<COwnerSlotRef, CRegistryError> {
        self.validate_owner_local(local)?;
        let value = COwnerSlotRef {
            local: local.clone(),
        };
        if !self.owner_slots.insert(value.clone()) {
            return Err(CRegistryError::DuplicateRegistration);
        }
        Ok(value)
    }

    pub fn owner_slots(&self) -> impl Iterator<Item = &COwnerSlotRef> {
        self.owner_slots.iter()
    }

    pub fn check_owner_slot(
        &self,
        function: &CFunctionRef,
        slot: &COwnerSlotRef,
    ) -> Result<(), CRegistryError> {
        self.check_local(function, slot.local())?;
        self.validate_owner_local(slot.local())?;
        if self.owner_slots.contains(slot) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    fn validate_owner_local(&self, local: &CLocalRef) -> Result<(), CRegistryError> {
        self.check_local(local.scope().function(), local)?;
        self.check_type(local.ty())?;
        if !owning_pointer(local.ty()) {
            return Err(CRegistryError::InvalidOwnerSlot);
        }
        Ok(())
    }
}

pub(super) fn owning_pointer(ty: &CObjectType) -> bool {
    let ty = ty.canonical();
    if ty.constness() != CConstness::Unqualified {
        return false;
    }
    let CObjectTypeKind::Pointer(CPointerTarget::Object(target)) = ty.kind() else {
        return false;
    };
    if target.require_storable().is_err() {
        return false;
    }
    let mut target = target.as_ref();
    while let CObjectTypeKind::Array { element, .. } = target.kind() {
        target = element;
    }
    target.constness() == CConstness::Unqualified
}
