//! Source metadata coherence, not proof that a Rust compiler accepted a program.
use super::JavaDialect;
use crate::ast::JavaPackage;
use portable_codegen::{
    AstViolation, CheckedRustDocumentation, GeneratedOrigin, RustSourceNode, RustSourceOrigin,
    RustVisibility, TargetAstPackage,
};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn verify(package: &TargetAstPackage<JavaDialect>) -> Vec<AstViolation> {
    let module = package.files().next().map(|file| *file.module());
    let origins = origins(package);
    let mut seen = BTreeSet::new();
    let mut violations = Vec::new();
    let checked = CheckedRustDocumentation::check(origins.inspect(|origin| {
        let crate_id = origin.declaration.crate_id;
        if module != Some(JavaPackage::RustCrate(crate_id)) {
            violations.push(error(
                "Java RustSource registration requires its own RustCrate namespace",
            ));
        }
        if origin.node != RustSourceNode::Declaration {
            violations.push(error(
                "Java RustSource registration requires a declaration origin, not a body-local node",
            ));
        }
        if origin.module.crate_id != crate_id
            || origin.crate_exports.root.crate_id != crate_id
            || matches!(origin.visibility, RustVisibility::RestrictedTo(owner) if owner.crate_id != crate_id)
        {
            violations.push(error(
                "Java RustSource module, export root and visibility owner must belong to its crate",
            ));
        }
        if !seen.insert(origin.declaration) {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                "one RustSource declaration cannot have multiple Java registrations",
            ));
        }
    }));
    match checked {
        Err(problem) => violations.push(error(&problem.to_string())),
        Ok(metadata) => violations.extend(super::documentation::verify_presentation_owner(
            package, &metadata,
        )),
    }
    violations
}

pub(super) fn origins(
    package: &TargetAstPackage<JavaDialect>,
) -> impl Iterator<Item = &RustSourceOrigin> {
    package
        .generated_types()
        .map(|value| &value.origin)
        .chain(package.callables().map(|value| &value.origin))
        .chain(package.interface_methods().map(|value| &value.origin))
        .chain(package.values().map(|value| &value.origin))
        .filter_map(|origin| match origin {
            GeneratedOrigin::RustSource(origin) => Some(origin.as_ref()),
            _ => None,
        })
        .chain(crate::ast::source_fields::origins(package))
}

fn error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::InvalidStructure, message)
}

#[cfg(test)]
#[path = "../tests/source_registration.rs"]
mod tests;
