//! Distinct source-level nesting paths must remain distinct JVM class names.

use super::{JavaFileItem, JavaMember, JavaTypeDeclaration};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn verify(context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for file in context.files() {
        for item in file.items() {
            if let JavaFileItem::Type { declaration, .. } = item {
                let root = format!("{}/{}", file.module().name(), declaration.name.as_str());
                declaration_names(declaration, &root, &mut seen, &mut errors);
                // File verification independently confines fragments to the
                // unique Runtime shell. Include that physical class's members.
                for fragment in file.items() {
                    if let JavaFileItem::RuntimeMembers { members, .. } = fragment {
                        nested_names(members, &root, &mut seen, &mut errors);
                    }
                }
            }
        }
    }
    errors
}

fn declaration_names(
    declaration: &JavaTypeDeclaration,
    binary: &str,
    seen: &mut BTreeSet<String>,
    errors: &mut Vec<AstViolation>,
) {
    if !seen.insert(binary.to_owned()) {
        errors.push(AstViolation::new(
            DiagnosticCode::DuplicateDeclaration,
            "Java source declarations resolve to the same JVM binary class name",
        ));
    }
    nested_names(&declaration.members, binary, seen, errors);
}

fn nested_names(
    members: &[JavaMember],
    owner: &str,
    seen: &mut BTreeSet<String>,
    errors: &mut Vec<AstViolation>,
) {
    for member in members {
        if let JavaMember::NestedType(child) = member {
            declaration_names(
                child,
                &format!("{owner}${}", child.name.as_str()),
                seen,
                errors,
            );
        }
    }
}
