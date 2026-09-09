//! Typed obligations over actual members, never child-liveness evidence.
#[cfg(test)]
#[path = "../../tests/member_role_reconstruction.rs"]
mod tests;

use super::{CAggregateRef, CMemberRef, CRegistry, CRegistryError};
use crate::ast::{CConstness, CObjectTypeKind, CPointerTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CMemberOwnership {
    Required,
    Optional,
    BorrowedMetadata,
}

/// A role over an existing member and its fixed-array terminal slots.
/// It contains no runtime allocation, completeness or transfer-success claim.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CMemberRef, CMemberOwnership, CMemberOwnershipRef};
/// fn forge(member: CMemberRef) -> CMemberOwnershipRef {
///     CMemberOwnershipRef { member, role: CMemberOwnership::Required }
/// }
/// ```
///
/// ```compile_fail
/// use portable_backend_c::ast::{CMemberRef, CMemberOwnershipRef};
/// fn retarget(mut role: CMemberOwnershipRef, member: CMemberRef) {
///     role.member = member;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMemberOwnershipRef {
    member: CMemberRef,
    role: CMemberOwnership,
}
impl CMemberOwnershipRef {
    pub fn member(&self) -> &CMemberRef {
        &self.member
    }
    pub const fn role(&self) -> CMemberOwnership {
        self.role
    }
}

impl CRegistry {
    pub fn register_member_ownership(
        &mut self,
        member: &CMemberRef,
        role: CMemberOwnership,
    ) -> Result<CMemberOwnershipRef, CRegistryError> {
        self.validate_member_ownership(member, role)?;
        if self.member_ownership.contains_key(member) {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CMemberOwnershipRef {
            member: member.clone(),
            role,
        };
        self.member_ownership.insert(member.clone(), value.clone());
        Ok(value)
    }

    pub fn member_ownerships(&self) -> impl Iterator<Item = &CMemberOwnershipRef> {
        self.member_ownership.values()
    }

    pub fn check_member_ownership(
        &self,
        owner: &CAggregateRef,
        value: &CMemberOwnershipRef,
    ) -> Result<(), CRegistryError> {
        self.check_member(owner, value.member())?;
        self.validate_member_ownership(value.member(), value.role())?;
        if self.member_ownership.get(value.member()) == Some(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    fn validate_member_ownership(
        &self,
        member: &CMemberRef,
        role: CMemberOwnership,
    ) -> Result<(), CRegistryError> {
        self.check_member(member.owner(), member)?;
        self.check_type(member.ty())?;
        let canonical = member.ty().canonical();
        let mut terminal = &canonical;
        while let CObjectTypeKind::Array { element, .. } = terminal.kind() {
            terminal = element;
        }
        let valid = match role {
            CMemberOwnership::Required | CMemberOwnership::Optional => {
                super::owner_slots::owning_pointer(terminal)
            }
            CMemberOwnership::BorrowedMetadata => match terminal.kind() {
                CObjectTypeKind::Pointer(CPointerTarget::Function(_)) => true,
                CObjectTypeKind::Pointer(CPointerTarget::Object(target)) => {
                    let mut base = target.as_ref();
                    while let CObjectTypeKind::Array { element, .. } = base.kind() {
                        base = element;
                    }
                    target.require_storable().is_ok() && base.constness() == CConstness::Const
                }
                _ => false,
            },
        };
        if valid {
            Ok(())
        } else {
            Err(CRegistryError::InvalidMemberOwnership)
        }
    }
}
