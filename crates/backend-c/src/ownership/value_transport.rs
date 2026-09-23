//! Closed pointer-free value shape, not source variant or effect authority.
use crate::ast::{CAggregateRef, CObjectType, CObjectTypeKind, CRegistry, CScalarType};

pub(crate) fn scalar_result(registry: Option<&CRegistry>, ty: &CObjectType) -> bool {
    let Some(registry) = registry else {
        return false;
    };
    let CObjectTypeKind::Struct(record) = ty.kind() else {
        return false;
    };
    if ty != &CObjectType::structure(record.clone()) {
        return false;
    }
    let owner = CAggregateRef::Struct(record.clone());
    let Ok(Some(members)) = registry.members(&owner) else {
        return false;
    };
    let [tag, payload] = members else {
        return false;
    };
    tag.owner() == &owner
        && payload.owner() == &owner
        && registry.check_member(&owner, tag).is_ok()
        && registry.check_member(&owner, payload).is_ok()
        && tag.ty() == &CObjectType::scalar(CScalarType::Bool)
        && payload.ty() == &CObjectType::scalar(CScalarType::I32)
}
