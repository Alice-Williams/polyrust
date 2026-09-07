//! Names that Java may reclassify as expression/type qualifiers.

use super::{
    JavaAnnotation, JavaFileItem, JavaIdentifier, JavaKnownType, JavaMember, JavaTypeDeclaration,
};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(crate) fn reserved_type_names() -> impl Iterator<Item = &'static str> {
    [
        "Generated",
        "Runtime",
        "GeneratedTest",
        "ConformanceTest",
        "InvalidTypes",
    ]
    .into_iter()
    .chain(JavaKnownType::ALL.into_iter().flat_map(|ty| {
        [
            ty.simple_name(),
            ty.qualified_name()
                .split('.')
                .next()
                .expect("qualified known type"),
        ]
    }))
    .chain(
        JavaAnnotation::ALL
            .into_iter()
            .map(JavaAnnotation::simple_name),
    )
}

pub(super) fn known_names() -> BTreeSet<JavaIdentifier> {
    reserved_type_names()
        .map(JavaIdentifier::from_portable)
        .collect()
}

pub(super) fn in_context(context: &TargetAstContext<'_, JavaDialect>) -> BTreeSet<JavaIdentifier> {
    fn collect(node: &JavaTypeDeclaration, names: &mut BTreeSet<JavaIdentifier>) {
        names.insert(node.name.clone());
        for member in &node.members {
            if let JavaMember::NestedType(child) = member {
                collect(child, names);
            }
        }
    }
    let mut names = known_names();
    for file in context.files() {
        for item in file.items() {
            if let JavaFileItem::Type { declaration, .. } = item {
                collect(declaration, &mut names);
            }
        }
    }
    names
}

pub(super) fn violation() -> AstViolation {
    AstViolation::new(
        DiagnosticCode::InvalidStructure,
        "Java binding cannot shadow a registered type or expression qualifier",
    )
}

pub(super) fn verify_declaration(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
    enclosing_names: &[JavaIdentifier],
) -> Vec<AstViolation> {
    let forbidden = in_context(context);
    let mut violations = Vec::new();
    let reserved_nominal = reserved_type_names().any(|name| declaration.name.as_str() == name);
    if reserved_nominal
        && !exact_runtime_nested(declaration, enclosing_names)
        && !registered_shell(declaration, context, enclosing_names)
    {
        violations.push(violation());
    }
    for name in declaration
        .type_parameters
        .iter()
        .chain(
            declaration
                .record_components
                .iter()
                .map(|value| &value.name),
        )
        .chain(
            declaration
                .members
                .iter()
                .filter_map(|member| match member {
                    JavaMember::Field(value) => Some(&value.name),
                    JavaMember::EnumConstant(value) => Some(&value.name),
                    JavaMember::CompileFailField(value) => Some(&value.name),
                    _ => None,
                }),
        )
        .chain(
            declaration
                .members
                .iter()
                .filter_map(|member| match member {
                    JavaMember::Method(value) => Some(value.type_parameters.iter()),
                    _ => None,
                })
                .flatten(),
        )
    {
        if forbidden.contains(name) {
            violations.push(violation());
        }
    }
    violations
}

fn registered_shell(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
    enclosing_names: &[JavaIdentifier],
) -> bool {
    use portable_codegen::{GeneratedOrigin, SourceRole, SynthesisReason};
    if !enclosing_names.is_empty() {
        return false;
    }
    match declaration.name.as_str() {
        "Generated" => declaration
            .declared
            .and_then(|id| context.generated_type(id))
            .is_some_and(|value| {
                value.origin == GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint)
            }),
        // Exact role/path/shell authentication is performed by the runtime
        // composed-file verifier, including its later combined declaration.
        "Runtime" => true,
        "GeneratedTest" | "ConformanceTest" | "InvalidTypes" => {
            let role = match declaration.name.as_str() {
                "GeneratedTest" => SourceRole::NativeTest,
                "ConformanceTest" => SourceRole::Conformance,
                _ => SourceRole::NegativeTest,
            };
            context.files().any(|file| {
                file.role() == role && file.items().iter().any(|item| {
                    matches!(item, JavaFileItem::Type { declaration: root, .. } if std::ptr::eq(root, declaration))
                })
            })
        }
        _ => false,
    }
}

// A reserved runtime name is only admitted for its exact registered helper
// declaration directly inside Runtime. Matching a spelling alone is not proof.
fn exact_runtime_nested(
    declaration: &JavaTypeDeclaration,
    enclosing_names: &[JavaIdentifier],
) -> bool {
    if enclosing_names.len() != 1 || enclosing_names[0].as_str() != "Runtime" {
        return false;
    }
    JavaKnownType::ALL
        .into_iter()
        .filter(|ty| ty.simple_name() == declaration.name.as_str())
        .filter_map(JavaKnownType::runtime_helper)
        .flat_map(crate::runtime::helper_items)
        .any(|item| {
            match item {
            JavaFileItem::RuntimeMembers { members, .. } => members.iter().any(|member| {
                matches!(member, JavaMember::NestedType(expected) if expected == declaration)
            }),
            JavaFileItem::Type { .. } => false,
        }
        })
}
