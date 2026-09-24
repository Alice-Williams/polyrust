//! C field scopes derive from exact registered aggregate owners, never names.
use super::{CDialect, bindings::CValueBinding, violation};
use crate::ast::{CAggregateRef, CFileRole};
use portable_codegen::{AstViolation, GeneratedTypeId, GeneratedValueId, TargetAstPackage};

pub(super) fn owner(
    package: &TargetAstPackage<CDialect>,
    value: GeneratedValueId,
) -> Result<Option<GeneratedTypeId>, AstViolation> {
    let unit = package
        .files()
        .next()
        .and_then(|file| file.items().first())
        .ok_or_else(|| violation("C value scope requires original package authority"))?;
    let bindings = &unit.projection.bindings;
    let binding = bindings
        .reverse_values
        .get(&value)
        .ok_or_else(|| violation("C value scope lacks an exact registered binding"))?;
    if let CValueBinding::Member(member) = binding
        && let CAggregateRef::Struct(record) = member.owner()
        && record.file().key().role == CFileRole::GeneratedPublicHeader
    {
        return bindings
            .types
            .get(record)
            .copied()
            .map(Some)
            .ok_or_else(|| violation("C public member lacks its exact registered type owner"));
    }
    // Preserve allocation of existing source-private bindings in this increment.
    Ok(None)
}
