//! Java AST: modifiers.

use super::declaration_model::JavaModifier;
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(super) enum JavaModifierSite {
    Type { top_level: bool },
    Field,
    Method,
    Constructor,
}

pub(super) fn verify_modifiers_for(
    modifiers: &[JavaModifier],
    site: JavaModifierSite,
) -> Vec<AstViolation> {
    let distinct = modifiers.iter().copied().collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    if distinct.len() != modifiers.len() {
        violations.push(AstViolation::new(
            DiagnosticCode::DuplicateDeclaration,
            "Java modifier is repeated",
        ));
    }
    for (left, right, message) in [
        (
            JavaModifier::Public,
            JavaModifier::Private,
            "Java declaration cannot be both public and private",
        ),
        (
            JavaModifier::Abstract,
            JavaModifier::Final,
            "Java declaration cannot be both abstract and final",
        ),
        (
            JavaModifier::Sealed,
            JavaModifier::NonSealed,
            "Java declaration cannot be both sealed and non-sealed",
        ),
    ] {
        if distinct.contains(&left) && distinct.contains(&right) {
            violations.push(AstViolation::new(DiagnosticCode::InvalidStructure, message));
        }
    }
    let allowed = |modifier| match site {
        JavaModifierSite::Type { top_level } => modifier == JavaModifier::Static && !top_level,
        JavaModifierSite::Field => matches!(
            modifier,
            JavaModifier::Public
                | JavaModifier::Private
                | JavaModifier::Static
                | JavaModifier::Final
                | JavaModifier::Transient
        ),
        JavaModifierSite::Method => matches!(
            modifier,
            JavaModifier::Public
                | JavaModifier::Private
                | JavaModifier::Static
                | JavaModifier::Final
                | JavaModifier::Abstract
        ),
        JavaModifierSite::Constructor => {
            matches!(modifier, JavaModifier::Public | JavaModifier::Private)
        }
    };
    for modifier in &distinct {
        if !allowed(*modifier) {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                format!("Java modifier {modifier:?} is illegal in this declaration context"),
            ));
        }
    }
    if matches!(site, JavaModifierSite::Method)
        && distinct.contains(&JavaModifier::Abstract)
        && (distinct.contains(&JavaModifier::Static)
            || distinct.contains(&JavaModifier::Private)
            || distinct.contains(&JavaModifier::Final))
    {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "abstract Java methods cannot be static, private, or final",
        ));
    }
    violations
}
