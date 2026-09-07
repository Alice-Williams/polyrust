//! Java dialect: file checks.

use super::JavaDialect;
use crate::ast::{JavaFileItem, JavaFilePlacement, JavaPackage};
use portable_codegen::{AstViolation, TargetAstContext};
use portable_diagnostics::DiagnosticCode;

pub(super) fn verify_java_file_identity(
    role: portable_codegen::SourceRole,
    path: &str,
    module: &JavaPackage,
    placement: &JavaFilePlacement,
    declares_runtime_type: bool,
) -> Vec<AstViolation> {
    const RUNTIME_PATH: &str = "src/main/java/org/polyrust/generated/Runtime.java";
    const MAIN_ROOT: &str = "src/main/java/org/polyrust/generated/";
    const TEST_ROOT: &str = "src/test/java/org/polyrust/generated/";
    let canonical = role == portable_codegen::SourceRole::Runtime
        && module == &JavaPackage::Generated
        && path == RUNTIME_PATH
        && *placement == JavaFilePlacement::Runtime;
    let uses_reserved_identity = role == portable_codegen::SourceRole::Runtime
        || path == RUNTIME_PATH
        || *placement == JavaFilePlacement::Runtime
        || (module == &JavaPackage::Generated && declares_runtime_type);
    let mut violations = Vec::new();
    if uses_reserved_identity && !canonical {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java Runtime identity requires the exact runtime role, placement, generated package, and canonical path",
        ));
    }
    let expected_root = match (role, placement) {
        (
            portable_codegen::SourceRole::PublicApi | portable_codegen::SourceRole::Implementation,
            JavaFilePlacement::Main,
        )
        | (portable_codegen::SourceRole::Runtime, JavaFilePlacement::Runtime) => Some(MAIN_ROOT),
        (portable_codegen::SourceRole::NativeTest, JavaFilePlacement::NativeTest)
        | (portable_codegen::SourceRole::Conformance, JavaFilePlacement::Conformance)
        | (portable_codegen::SourceRole::NegativeTest, JavaFilePlacement::NegativeTest) => {
            Some(TEST_ROOT)
        }
        _ => None,
    };
    match expected_root {
        Some(root)
            if path
                .strip_prefix(root)
                .is_some_and(|filename| !filename.is_empty() && !filename.contains('/')) => {}
        Some(_) => violations.push(AstViolation::new(
            DiagnosticCode::UnsafeOutputPath,
            "Java source path must use the canonical package directory for its typed placement",
        )),
        None => violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java source role and file placement are incompatible",
        )),
    }
    violations
}

pub(super) fn declares_reserved_runtime_type(item: &JavaFileItem) -> bool {
    matches!(
        item,
        JavaFileItem::Type { declaration, .. } if declaration.name.as_str() == "Runtime"
    )
}

pub(super) fn verify_composed_java_file(
    placement: &JavaFilePlacement,
    items: Vec<&JavaFileItem>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let runtime_fragments = items
        .iter()
        .filter(|item| matches!(item, JavaFileItem::RuntimeMembers { .. }))
        .count();
    if *placement != JavaFilePlacement::Runtime {
        return (runtime_fragments > 0)
            .then(|| {
                AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "Java runtime member fragments are confined to the runtime file",
                )
            })
            .into_iter()
            .collect();
    }

    let shells = items
        .iter()
        .filter_map(|item| match *item {
            JavaFileItem::Type { .. } => Some(*item),
            JavaFileItem::RuntimeMembers { .. } => None,
        })
        .collect::<Vec<_>>();
    if shells.len() != 1 {
        return vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java runtime file must contain exactly one typed class shell",
        )];
    }

    let expected_shell = crate::runtime::shell_item();
    if shells[0] != &expected_shell {
        return vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java runtime file does not contain the exact registered Runtime class shell",
        )];
    }

    let JavaFileItem::Type { declaration, .. } = shells[0] else {
        unreachable!("runtime shell collection contains only type items")
    };
    let mut combined = declaration.clone();
    for item in items {
        if let JavaFileItem::RuntimeMembers { members, .. } = item {
            combined.members.extend(members.iter().cloned());
        }
    }
    let mut violations = combined.verify(context, true);
    if combined.contains_compile_fail_member() {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java compile-fail members are confined to negative-test files",
        ));
    }
    violations
}
