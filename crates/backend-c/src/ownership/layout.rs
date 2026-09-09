//! Checked LP64 layouts derived from actual registered member inventories.
use std::collections::{BTreeMap, BTreeSet};

use super::CSafetyError as E;
use crate::ast::{
    CAggregateRef, CKnownObject, CMemberRef, CObjectType, CObjectTypeKind, CPointerTarget,
    CRegistry, CReturnType, CScalarRepresentation,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CLayout {
    size: u64,
    alignment: u64,
}

impl CLayout {
    pub(super) const fn size(self) -> u64 {
        self.size
    }
    pub(super) const fn alignment(self) -> u64 {
        self.alignment
    }
}

pub(super) struct Layouts<'a> {
    registry: &'a CRegistry,
    layouts: BTreeMap<CObjectType, CLayout>,
    offsets: BTreeMap<CMemberRef, u64>,
}

enum Pending {
    Enter(CObjectType),
    Array {
        key: CObjectType,
        element: CObjectType,
        count: u64,
    },
    Aggregate {
        key: CObjectType,
        owner: CAggregateRef,
        members: Vec<CMemberRef>,
    },
}

impl<'a> Layouts<'a> {
    pub(super) fn new(registry: &'a CRegistry) -> Self {
        Self {
            registry,
            layouts: BTreeMap::new(),
            offsets: BTreeMap::new(),
        }
    }

    pub(super) fn object(&mut self, ty: &CObjectType) -> Result<CLayout, E> {
        self.registry.check_type(ty)?;
        let root = ty.canonical();
        let mut pending = vec![Pending::Enter(root.clone())];
        let mut active = BTreeSet::new();
        while let Some(item) = pending.pop() {
            match item {
                Pending::Enter(ty) => {
                    if self.layouts.contains_key(&ty) {
                        continue;
                    }
                    if !active.insert(ty.clone()) {
                        return Err(E::RecursiveLayout);
                    }
                    match ty.kind() {
                        CObjectTypeKind::Array { element, length } => {
                            let element = element.canonical();
                            pending.push(Pending::Array {
                                key: ty.clone(),
                                element: element.clone(),
                                count: length.get(),
                            });
                            pending.push(Pending::Enter(element));
                            continue;
                        }
                        CObjectTypeKind::Struct(_) | CObjectTypeKind::Union(_) => {
                            let owner = match ty.kind() {
                                CObjectTypeKind::Struct(value) => {
                                    CAggregateRef::Struct(value.clone())
                                }
                                CObjectTypeKind::Union(value) => {
                                    CAggregateRef::Union(value.clone())
                                }
                                _ => unreachable!(),
                            };
                            let members = self
                                .registry
                                .members(&owner)?
                                .ok_or(E::IncompleteLayout)?
                                .to_vec();
                            pending.push(Pending::Aggregate {
                                key: ty.clone(),
                                owner,
                                members: members.clone(),
                            });
                            pending.extend(
                                members
                                    .iter()
                                    .rev()
                                    .map(|member| Pending::Enter(member.ty().canonical())),
                            );
                            continue;
                        }
                        _ => {}
                    }
                    let layout = self.leaf(&ty)?;
                    active.remove(&ty);
                    self.layouts.insert(ty, layout);
                }
                Pending::Array {
                    key,
                    element,
                    count,
                } => {
                    let element = self.layouts[&element];
                    let size = element.size.checked_mul(count).ok_or(E::LayoutCapacity)?;
                    self.layouts.insert(
                        key.clone(),
                        CLayout {
                            size,
                            alignment: element.alignment,
                        },
                    );
                    active.remove(&key);
                }
                Pending::Aggregate {
                    key,
                    owner,
                    members,
                } => {
                    let mut size = 0_u64;
                    let mut alignment = 1_u64;
                    for member in members {
                        let layout = self.layouts[&member.ty().canonical()];
                        alignment = alignment.max(layout.alignment);
                        let offset = match owner {
                            CAggregateRef::Struct(_) => {
                                let offset = align(size, layout.alignment)?;
                                size = offset.checked_add(layout.size).ok_or(E::LayoutCapacity)?;
                                offset
                            }
                            CAggregateRef::Union(_) => {
                                size = size.max(layout.size);
                                0
                            }
                        };
                        self.offsets.insert(member, offset);
                    }
                    let size = align(size, alignment)?;
                    self.layouts
                        .insert(key.clone(), CLayout { size, alignment });
                    active.remove(&key);
                }
            }
        }
        Ok(self.layouts[&root])
    }

    fn leaf(&self, ty: &CObjectType) -> Result<CLayout, E> {
        let (size, alignment) = match ty.kind() {
            CObjectTypeKind::Scalar(value) => {
                let size = match value.representation() {
                    CScalarRepresentation::Bool => 1,
                    CScalarRepresentation::Signed(width)
                    | CScalarRepresentation::Unsigned(width) => u64::from(width.bits() / 8),
                    CScalarRepresentation::Binary64 => 8,
                };
                (size, size)
            }
            CObjectTypeKind::Pointer(_) => (8, 8),
            CObjectTypeKind::Known(CKnownObject::MaxAlign) => (32, 16),
            CObjectTypeKind::Known(CKnownObject::File) => return Err(E::IncompleteLayout),
            CObjectTypeKind::Enum(value) => {
                self.registry
                    .enumerators(value)?
                    .ok_or(E::IncompleteLayout)?;
                (4, 4)
            }
            CObjectTypeKind::Struct(_)
            | CObjectTypeKind::Union(_)
            | CObjectTypeKind::Array { .. }
            | CObjectTypeKind::Typedef(_) => unreachable!("layout work item or expanded alias"),
        };
        Ok(CLayout { size, alignment })
    }

    /// A pointer may name an incomplete tag, but every array type needs layout.
    pub(super) fn form(&mut self, ty: &CObjectType) -> Result<(), E> {
        self.registry.check_type(ty)?;
        let mut pending = vec![ty.canonical()];
        while let Some(ty) = pending.pop() {
            match ty.kind() {
                CObjectTypeKind::Array { .. } => {
                    self.object(&ty)?;
                }
                CObjectTypeKind::Pointer(CPointerTarget::Object(target)) => {
                    pending.push(target.canonical())
                }
                CObjectTypeKind::Pointer(CPointerTarget::Function(signature)) => {
                    if let CReturnType::Value(result) = signature.return_type() {
                        pending.push(result.declared_type().canonical());
                    }
                    pending.extend(
                        signature
                            .parameters()
                            .iter()
                            .map(|value| value.declared_type().canonical()),
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn align(value: u64, alignment: u64) -> Result<u64, E> {
    let remainder = value % alignment;
    if remainder == 0 {
        Ok(value)
    } else {
        value
            .checked_add(alignment - remainder)
            .ok_or(E::LayoutCapacity)
    }
}

#[cfg(test)]
#[path = "../tests/constant_layout.rs"]
mod tests;
