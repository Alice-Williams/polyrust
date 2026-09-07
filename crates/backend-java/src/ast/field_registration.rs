//! Registered static values are declarations, not merely references to typed IDs.

use super::{JavaField, JavaModifier, JavaVisibility};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};
use portable_diagnostics::DiagnosticCode;

pub(super) fn verify(
    field: &JavaField,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let Some(id) = field.declared else {
        return vec![];
    };
    let valid = context.value(id).is_some_and(|registered| {
        let public = field.modifiers.contains(&JavaModifier::Public);
        let private = field.modifiers.contains(&JavaModifier::Private);
        let visibility = match registered.visibility {
            JavaVisibility::Public => public && !private,
            JavaVisibility::Private => private && !public,
            JavaVisibility::Package => !public && !private,
        };
        registered.name == field.name.as_str()
            && (!matches!(
                registered.origin,
                portable_codegen::GeneratedOrigin::CoreDeclaration(_)
            ) || matches!(
                registered.origin,
                portable_codegen::GeneratedOrigin::CoreDeclaration(
                    portable_core_ir::CoreDeclaration::Constant(_)
                )
            ))
            && registered.ty == JavaDialect.registered_type(&field.ty)
            && field.modifiers.contains(&JavaModifier::Static)
            && field.modifiers.contains(&JavaModifier::Final)
            && field.initializer.is_some()
            && visibility
    });
    if valid {
        vec![]
    } else {
        vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java registered value must retain its name, type, visibility, and initialized static-final declaration",
        )]
    }
}
