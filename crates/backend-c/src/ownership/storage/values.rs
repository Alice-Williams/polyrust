//! Compressed representation initialization, separate from owning-handle state.
mod join;
mod projections;
mod writes;

use crate::ast::{CFunctionRef, CMemberRef, CObjectType, CObjectTypeKind, CRegistry};
use crate::ownership::{
    CSafetyError as E,
    paths::{Key, Root},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Pointer {
    Null,
    Target(Box<Key>),
    Function(CFunctionRef),
    Unknown,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Cell {
    Uninitialized,
    Initialized,
    Zero,
    Pointer(Pointer),
    Record(BTreeMap<CMemberRef, Cell>),
    Array {
        default: Box<Cell>,
        elements: BTreeMap<u64, Cell>,
    },
    Union {
        member: CMemberRef,
        value: Box<Cell>,
    },
}

impl Cell {
    pub(super) fn pointer(&self) -> Result<Pointer, E> {
        match self {
            Self::Pointer(Pointer::Expired) => Err(E::ExpiredStorage),
            Self::Pointer(Pointer::Unknown) | Self::Initialized => Err(E::UnprovedStorage),
            Self::Pointer(value) => Ok(value.clone()),
            Self::Zero => Ok(Pointer::Null),
            _ => Err(E::UninitializedStorage),
        }
    }

    pub(super) fn expire(&mut self, root: &Root) {
        match self {
            Self::Pointer(Pointer::Target(path)) if path.root() == root => {
                *self = Self::Pointer(Pointer::Expired);
            }
            Self::Record(fields) => {
                for field in fields.values_mut() {
                    field.expire(root);
                }
            }
            Self::Array { default, elements } => {
                default.expire(root);
                for element in elements.values_mut() {
                    element.expire(root);
                }
            }
            Self::Union { value, .. } => value.expire(root),
            _ => {}
        }
    }

    pub(super) fn complete(&self, ty: &CObjectType, registry: &CRegistry) -> Result<(), E> {
        if matches!(self, Self::Uninitialized) {
            return Err(E::UninitializedStorage);
        }
        match ty.canonical().kind() {
            CObjectTypeKind::Struct(owner) => {
                let owner = crate::ast::CAggregateRef::Struct(owner.clone());
                for member in registry.members(&owner)?.ok_or(E::IncompleteLayout)? {
                    self.member(member, registry)?
                        .complete(member.ty(), registry)?;
                }
            }
            CObjectTypeKind::Union(_) => {
                let member = self.active(ty, registry)?.ok_or(E::InactiveUnionMember)?;
                self.member(&member, registry)?
                    .complete(member.ty(), registry)?;
            }
            CObjectTypeKind::Array { element, length } => {
                let (default, elements) = self.array();
                if (elements.len() as u64) < length.get() {
                    default.complete(element, registry)?;
                }
                for cell in elements.values() {
                    cell.complete(element, registry)?;
                }
            }
            CObjectTypeKind::Pointer(_) => {
                self.pointer()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn automatic_address(
        &self,
        ty: &CObjectType,
        registry: &CRegistry,
    ) -> Result<bool, E> {
        Ok(match ty.canonical().kind() {
            CObjectTypeKind::Pointer(_) => match self.pointer()? {
                Pointer::Target(path) => !matches!(path.root(), Root::Global(_)),
                Pointer::Unknown | Pointer::Expired => true,
                Pointer::Null | Pointer::Function(_) => false,
            },
            CObjectTypeKind::Struct(owner) => {
                let owner = crate::ast::CAggregateRef::Struct(owner.clone());
                let mut escapes = false;
                for member in registry.members(&owner)?.ok_or(E::IncompleteLayout)? {
                    escapes |= self
                        .member(member, registry)?
                        .automatic_address(member.ty(), registry)?;
                }
                escapes
            }
            CObjectTypeKind::Union(_) => {
                let member = self.active(ty, registry)?.ok_or(E::InactiveUnionMember)?;
                self.member(&member, registry)?
                    .automatic_address(member.ty(), registry)?
            }
            CObjectTypeKind::Array { element, length } => {
                let (default, elements) = self.array();
                let mut escapes = (elements.len() as u64) < length.get()
                    && default.automatic_address(element, registry)?;
                for cell in elements.values() {
                    escapes |= cell.automatic_address(element, registry)?;
                }
                escapes
            }
            _ => false,
        })
    }
}
