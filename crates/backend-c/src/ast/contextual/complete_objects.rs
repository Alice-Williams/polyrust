//! Complete-by-value dependencies; pointer recursion is not a layout cycle.

use super::super::{
    CAggregateRef, CObjectType, CObjectTypeKind, CPointerTarget, CRegistry, CReturnType,
    registry::CRegistered,
};
use super::CContextError as E;
use std::collections::BTreeSet;

pub(super) fn check(registry: &CRegistry) -> Result<(), E> {
    let mut checker = CompleteObjects {
        registry,
        active: BTreeSet::new(),
        done: BTreeSet::new(),
    };
    for node in registry.contextual_inventory() {
        match node {
            CRegistered::Struct(value) => {
                let owner = CAggregateRef::Struct(value.clone());
                if registry.members(&owner)?.is_some() {
                    checker.aggregate(owner)?;
                }
            }
            CRegistered::Union(value) => {
                let owner = CAggregateRef::Union(value.clone());
                if registry.members(&owner)?.is_some() {
                    checker.aggregate(owner)?;
                }
            }
            CRegistered::Typedef(value) => checker.form(value.target())?,
            CRegistered::Member(value) => checker.require(value.ty())?,
            CRegistered::Object(value) => checker.require(value.ty())?,
            CRegistered::Local(value) => checker.require(value.ty())?,
            CRegistered::Parameter(value) => checker.require(value.ty())?,
            CRegistered::Function(value) => {
                if let CReturnType::Value(value) = value.signature().return_type() {
                    checker.require(value.declared_type())?;
                }
                for parameter in value.signature().parameters() {
                    checker.require(parameter.declared_type())?;
                }
            }
            CRegistered::Enum(value) => {
                if registry.enumerators(value)?.is_none() {
                    return Err(E::IncompleteObject);
                }
            }
            CRegistered::Enumerator(_)
            | CRegistered::Scope(_)
            | CRegistered::Loop(_)
            | CRegistered::Switch(_)
            | CRegistered::CleanupExit(_)
            | CRegistered::Allocation(_)
            | CRegistered::Witness(_)
            | CRegistered::Table(_)
            | CRegistered::Adapter(_) => {}
        }
    }
    Ok(())
}

struct CompleteObjects<'a> {
    registry: &'a CRegistry,
    active: BTreeSet<CAggregateRef>,
    done: BTreeSet<CAggregateRef>,
}

impl CompleteObjects<'_> {
    fn require(&mut self, ty: &CObjectType) -> Result<(), E> {
        self.registry.check_type(ty)?;
        let canonical = ty.canonical();
        match canonical.kind() {
            CObjectTypeKind::Struct(value) => {
                self.aggregate(CAggregateRef::Struct(value.clone()))?
            }
            CObjectTypeKind::Union(value) => self.aggregate(CAggregateRef::Union(value.clone()))?,
            CObjectTypeKind::Array { element, .. } => self.require(element)?,
            CObjectTypeKind::Enum(value) => {
                if self.registry.enumerators(value)?.is_none() {
                    return Err(E::IncompleteObject);
                }
            }
            CObjectTypeKind::Known(_) => canonical
                .require_storable()
                .map_err(super::super::CExpressionError::from)?,
            CObjectTypeKind::Pointer(_) => self.form(&canonical)?,
            CObjectTypeKind::Scalar(_) => {}
            CObjectTypeKind::Typedef(_) => unreachable!("canonical type expands aliases"),
        }
        Ok(())
    }

    // Pointer targets may be incomplete, but a pointer-to-array still names
    // an array type whose element must be complete. Check nested prototypes
    // without incorrectly requiring pointed-to struct layout completion.
    fn form(&mut self, ty: &CObjectType) -> Result<(), E> {
        self.registry.check_type(ty)?;
        let canonical = ty.canonical();
        match canonical.kind() {
            CObjectTypeKind::Array { element, .. } => self.require(element)?,
            CObjectTypeKind::Pointer(CPointerTarget::Object(target)) => self.form(target)?,
            CObjectTypeKind::Pointer(CPointerTarget::Function(signature)) => {
                if let CReturnType::Value(value) = signature.return_type() {
                    self.form(value.declared_type())?;
                }
                for value in signature.parameters() {
                    self.form(value.declared_type())?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn aggregate(&mut self, owner: CAggregateRef) -> Result<(), E> {
        if self.done.contains(&owner) {
            return Ok(());
        }
        if !self.active.insert(owner.clone()) {
            return Err(E::RecursiveObject);
        }
        let members = self.registry.members(&owner)?.ok_or(E::IncompleteObject)?;
        for member in members {
            self.require(member.ty())?;
        }
        self.active.remove(&owner);
        self.done.insert(owner);
        Ok(())
    }
}
