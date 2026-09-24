//! Source record component identity; metadata itself is not compiler authority.
use super::{
    JavaDeclarationKind, JavaFileItem, JavaIdentifier, JavaMember, JavaRecordComponentOrigin,
    JavaType, JavaTypeDeclaration,
};
use crate::dialect::JavaDialect;
use portable_codegen::{
    AstViolation, GeneratedOrigin, GeneratedTypeId, RustDeclarationId, RustSourceOrigin,
    TargetAstContext,
};
use portable_diagnostics::DiagnosticCode;
use std::sync::Arc;

#[cfg(test)]
#[path = "../tests/source_records.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaSourceFieldOrigin {
    pub owner: RustDeclarationId,
    pub origin: Arc<RustSourceOrigin>,
}

pub(super) fn verify(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let owner = declaration
        .declared
        .and_then(|id| context.generated_type(id))
        .and_then(|value| {
            if let GeneratedOrigin::RustSource(origin) = &value.origin {
                Some(origin)
            } else {
                None
            }
        });
    let mut violations = Vec::new();
    for component in &declaration.record_components {
        let valid = match (&component.origin, owner) {
            (JavaRecordComponentOrigin::Synthesized(field), _) => {
                super::synthesized_fields::valid_component(*field, component, declaration, context)
            }
            (JavaRecordComponentOrigin::RustSource(field), Some(owner)) => {
                declaration.kind == JavaDeclarationKind::Record
                    && field.owner == owner.declaration
                    && field.origin.declaration != owner.declaration
                    && field.origin.module == owner.module
                    && field.origin.crate_exports.root == owner.crate_exports.root
            }
            (JavaRecordComponentOrigin::RustSource(_), None) => false,
            (
                JavaRecordComponentOrigin::Core(_) | JavaRecordComponentOrigin::Runtime(_),
                Some(_),
            ) => false,
            (JavaRecordComponentOrigin::Core(_) | JavaRecordComponentOrigin::Runtime(_), None) => {
                true
            }
        };
        if !valid {
            violations.push(AstViolation::new(DiagnosticCode::InvalidStructure,
                "Java record component must retain its registered source nominal owner and module/export identity, or exact synthesized owner/role/type"));
        }
    }
    violations
}

pub(super) fn matches(
    owner: GeneratedTypeId,
    field: RustDeclarationId,
    name: &JavaIdentifier,
    ty: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| file.items().iter().any(|item| {
        let JavaFileItem::Type { declaration, .. } = item else { return false; };
        let mut pending = vec![declaration.as_ref()];
        while let Some(declaration) = pending.pop() {
            if declaration.declared == Some(owner) {
                return declaration.record_components.iter().any(|component| {
                    matches!(&component.origin, JavaRecordComponentOrigin::RustSource(origin) if origin.origin.declaration == field)
                        && &component.name == name && &component.ty == ty
                });
            }
            pending.extend(declaration.members.iter().filter_map(|member| {
                if let JavaMember::NestedType(child) = member { Some(child) } else { None }
            }));
        }
        false
    }))
}

mod traversal;
pub(crate) use traversal::origins;
