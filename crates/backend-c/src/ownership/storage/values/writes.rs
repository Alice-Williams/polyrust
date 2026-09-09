//! Exact writes initialize; interval writes weakly update without invented coverage.
use super::{Cell, E};
use crate::ast::{CAggregateRef, CObjectType, CObjectTypeKind, CRegistry};
use crate::ownership::paths::Selector;
use std::collections::BTreeMap;

impl Cell {
    pub(in crate::ownership::storage) fn write(
        &mut self,
        ty: &CObjectType,
        path: &[Selector],
        value: &Self,
        registry: &CRegistry,
    ) -> Result<(), E> {
        let Some((next, tail)) = path.split_first() else {
            *self = value.clone();
            return Ok(());
        };
        match next {
            Selector::Element(_) => return Err(E::UnprovedStorage),
            Selector::Member(member) => match member.owner() {
                CAggregateRef::Union(_) => {
                    let mut child = if self.active(ty, registry)?.as_ref() == Some(member) {
                        self.member(member, registry)?
                    } else {
                        Self::Uninitialized
                    };
                    child.write(member.ty(), tail, value, registry)?;
                    *self = Self::Union {
                        member: (**member).clone(),
                        value: Box::new(child),
                    };
                }
                CAggregateRef::Struct(owner) => {
                    let mut fields = BTreeMap::new();
                    for field in registry
                        .members(&CAggregateRef::Struct(owner.clone()))?
                        .ok_or(E::IncompleteLayout)?
                    {
                        fields.insert(field.clone(), self.member(field, registry)?);
                    }
                    fields
                        .get_mut(member.as_ref())
                        .ok_or(E::UnprovedStorage)?
                        .write(member.ty(), tail, value, registry)?;
                    *self = Self::Record(fields);
                }
            },
            Selector::Index { first, last } => {
                let canonical = ty.canonical();
                let CObjectTypeKind::Array { element, length } = canonical.kind() else {
                    return Err(E::UnprovedStorage);
                };
                if first > last || *last >= length.get() {
                    return Err(E::IndexOutOfBounds);
                }
                let (mut default, mut elements) = self.array();
                if first == last {
                    elements
                        .entry(*first)
                        .or_insert_with(|| default.clone())
                        .write(element, tail, value, registry)?;
                } else {
                    // A non-singleton write initializes no particular element.
                    // Weakly updating the compressed default also outside the
                    // selected interval is deliberately conservative.
                    let mut changed = default.clone();
                    changed.write(element, tail, value, registry)?;
                    default = default.join(&changed, element, registry)?;
                    for (_, cell) in elements.range_mut(*first..=*last) {
                        let mut changed = cell.clone();
                        changed.write(element, tail, value, registry)?;
                        *cell = cell.join(&changed, element, registry)?;
                    }
                }
                *self = Self::Array {
                    default: Box::new(default),
                    elements,
                };
            }
        }
        Ok(())
    }
}
