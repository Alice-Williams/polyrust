//! Must-joins keep initialization and exact pointer/active-member agreement.
use super::{Cell, E, Pointer};
use crate::ast::{CAggregateRef, CObjectType, CObjectTypeKind, CRegistry};
use std::collections::{BTreeMap, BTreeSet};

impl Cell {
    pub(in crate::ownership::storage) fn join(
        &self,
        other: &Self,
        ty: &CObjectType,
        registry: &CRegistry,
    ) -> Result<Self, E> {
        if self == other {
            return Ok(self.clone());
        }
        if matches!(self, Self::Uninitialized) || matches!(other, Self::Uninitialized) {
            return Ok(Self::Uninitialized);
        }
        Ok(match ty.canonical().kind() {
            CObjectTypeKind::Pointer(_) => match (self.pointer(), other.pointer()) {
                (Ok(a), Ok(b)) if a == b => Self::Pointer(a),
                _ => Self::Pointer(Pointer::Unknown),
            },
            CObjectTypeKind::Struct(owner) => {
                let mut fields = BTreeMap::new();
                for member in registry
                    .members(&CAggregateRef::Struct(owner.clone()))?
                    .ok_or(E::IncompleteLayout)?
                {
                    fields.insert(
                        member.clone(),
                        self.member(member, registry)?.join(
                            &other.member(member, registry)?,
                            member.ty(),
                            registry,
                        )?,
                    );
                }
                Self::Record(fields)
            }
            CObjectTypeKind::Array { element, .. } => {
                let (a, left) = self.array();
                let (b, right) = other.array();
                let keys: BTreeSet<_> = left.keys().chain(right.keys()).copied().collect();
                let mut elements = BTreeMap::new();
                for key in keys {
                    elements.insert(
                        key,
                        left.get(&key).unwrap_or(&a).join(
                            right.get(&key).unwrap_or(&b),
                            element,
                            registry,
                        )?,
                    );
                }
                Self::Array {
                    default: Box::new(a.join(&b, element, registry)?),
                    elements,
                }
            }
            CObjectTypeKind::Union(_) => {
                match (self.active(ty, registry)?, other.active(ty, registry)?) {
                    (Some(a), Some(b)) if a == b => {
                        let value = self.member(&a, registry)?.join(
                            &other.member(&a, registry)?,
                            a.ty(),
                            registry,
                        )?;
                        Self::Union {
                            member: a,
                            value: Box::new(value),
                        }
                    }
                    _ => Self::Initialized,
                }
            }
            _ => Self::Initialized,
        })
    }
}
