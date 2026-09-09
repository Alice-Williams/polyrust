//! Reads require every selected initialized subobject and the actual union arm.
use super::{Cell, E};
use crate::ast::{CAggregateRef, CMemberRef, CObjectType, CObjectTypeKind, CRegistry};
use crate::ownership::paths::Selector;
use std::collections::BTreeMap;

impl Cell {
    pub(in crate::ownership::storage) fn member(
        &self,
        member: &CMemberRef,
        registry: &CRegistry,
    ) -> Result<Self, E> {
        if let CAggregateRef::Union(owner) = member.owner() {
            let ty = CObjectType::union(owner.clone());
            if self.active(&ty, registry)?.as_ref() != Some(member) {
                return Err(E::InactiveUnionMember);
            }
        }
        Ok(match self {
            Self::Record(fields) => fields.get(member).cloned().unwrap_or(Self::Uninitialized),
            Self::Union { value, .. } => (**value).clone(),
            Self::Zero => Self::Zero,
            Self::Initialized => Self::Initialized,
            _ => Self::Uninitialized,
        })
    }
    pub(in crate::ownership::storage) fn active(
        &self,
        ty: &CObjectType,
        registry: &CRegistry,
    ) -> Result<Option<CMemberRef>, E> {
        Ok(match self {
            Self::Union { member, .. } => Some(member.clone()),
            Self::Zero => {
                let CObjectTypeKind::Union(owner) = ty.canonical().kind().clone() else {
                    return Ok(None);
                };
                registry
                    .members(&CAggregateRef::Union(owner))?
                    .ok_or(E::IncompleteLayout)?
                    .first()
                    .cloned()
            }
            _ => None,
        })
    }
    pub(in crate::ownership::storage) fn array(&self) -> (Self, BTreeMap<u64, Self>) {
        match self {
            Self::Array { default, elements } => ((**default).clone(), elements.clone()),
            Self::Zero => (Self::Zero, BTreeMap::new()),
            Self::Initialized => (Self::Initialized, BTreeMap::new()),
            _ => (Self::Uninitialized, BTreeMap::new()),
        }
    }
    pub(in crate::ownership::storage) fn read(
        &self,
        ty: &CObjectType,
        path: &[Selector],
        registry: &CRegistry,
    ) -> Result<Self, E> {
        let Some((next, tail)) = path.split_first() else {
            self.complete(ty, registry)?;
            return Ok(self.clone());
        };
        match next {
            Selector::Element(_) => Err(E::UnprovedStorage),
            Selector::Member(member) => {
                self.member(member, registry)?
                    .read(member.ty(), tail, registry)
            }
            Selector::Index { first, last } => {
                let canonical = ty.canonical();
                let CObjectTypeKind::Array { element, length } = canonical.kind() else {
                    return Err(E::UnprovedStorage);
                };
                if first > last || *last >= length.get() {
                    return Err(E::IndexOutOfBounds);
                }
                let (default, elements) = self.array();
                let selected: Vec<_> = elements
                    .range(*first..=*last)
                    .map(|(_, cell)| cell)
                    .collect();
                let mut values = Vec::new();
                if (selected.len() as u64) < last - first + 1 {
                    values.push(default.read(element, tail, registry)?);
                }
                for cell in selected {
                    values.push(cell.read(element, tail, registry)?);
                }
                let mut values = values.into_iter();
                let mut result = values.next().ok_or(E::UnprovedStorage)?;
                let result_type = selected_type(element, tail);
                for cell in values {
                    result = result.join(&cell, &result_type, registry)?;
                }
                Ok(result)
            }
        }
    }
}

fn selected_type(ty: &CObjectType, tail: &[Selector]) -> CObjectType {
    let mut ty = ty.canonical();
    for selector in tail {
        ty = match selector {
            Selector::Element(_) => {
                unreachable!("buffer selectors are consumed by root storage")
            }
            Selector::Member(member) => member.ty().canonical(),
            Selector::Index { .. } => match ty.kind() {
                CObjectTypeKind::Array { element, .. } => element.canonical(),
                _ => unreachable!("checked storage path"),
            },
        };
    }
    ty
}
