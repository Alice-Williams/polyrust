//! Java AST: sealed permits.

use super::declaration_model::{
    JavaDeclarationKind, JavaHeritage, JavaMember, JavaTypeDeclaration, JavaVisibility,
};
use super::field_metadata::find_type_declaration;
use super::file_model::JavaFileItem;
use super::types::{JavaType, JavaTypeName};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedTypeId, TargetAstContext};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn java_visibility_rank(visibility: JavaVisibility) -> u8 {
    match visibility {
        JavaVisibility::Private => 0,
        JavaVisibility::Package => 1,
        JavaVisibility::Public => 2,
    }
}

pub(super) fn verify_sealed_permits(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    if declaration.kind != JavaDeclarationKind::SealedInterface {
        return vec![];
    }
    let Some(interface) = declaration.declared else {
        return vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "sealed Java interface must have a generated declaration identity",
        )];
    };
    let actual = generated_implementors(interface, context);
    let mut permitted = BTreeSet::new();
    let mut violations = Vec::new();
    for value in &declaration.permits {
        let JavaType::Reference(JavaTypeName::Generated(id)) = value else {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "sealed Java permits entries must be non-generic generated declaration types",
            ));
            continue;
        };
        if !permitted.insert(*id) {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                "sealed Java permits entry is repeated",
            ));
        }
        let valid_declaration = find_type_declaration(value, context).is_some_and(|candidate| {
            matches!(
                candidate.kind,
                JavaDeclarationKind::FinalClass | JavaDeclarationKind::Record
            ) && matches!(
                candidate.heritage,
                JavaHeritage::Interfaces(ref interfaces)
                    if interfaces.contains(&JavaType::Reference(JavaTypeName::Generated(interface)))
            )
        });
        if !valid_declaration {
            violations.push(AstViolation::new(
                DiagnosticCode::InterfaceNonconformance,
                "sealed Java permits entry does not name an actual final implementing declaration",
            ));
        }
    }
    if actual.is_empty() || permitted != actual {
        violations.push(AstViolation::new(
            DiagnosticCode::InterfaceNonconformance,
            "sealed Java permits set must exactly name every implementing declaration once",
        ));
    }
    violations
}

fn generated_implementors(
    interface: GeneratedTypeId,
    context: &TargetAstContext<'_, JavaDialect>,
) -> BTreeSet<GeneratedTypeId> {
    let mut output = BTreeSet::new();
    for file in context.files() {
        for item in file.items() {
            if let JavaFileItem::Type { declaration, .. } = item {
                collect_generated_implementors(declaration, interface, &mut output);
            }
        }
    }
    output
}

fn collect_generated_implementors(
    declaration: &JavaTypeDeclaration,
    interface: GeneratedTypeId,
    output: &mut BTreeSet<GeneratedTypeId>,
) {
    if let Some(id) = declaration.declared
        && matches!(
            declaration.kind,
            JavaDeclarationKind::FinalClass | JavaDeclarationKind::Record
        )
        && matches!(
            declaration.heritage,
            JavaHeritage::Interfaces(ref interfaces)
                if interfaces.contains(&JavaType::Reference(JavaTypeName::Generated(interface)))
        )
    {
        output.insert(id);
    }
    for member in &declaration.members {
        if let JavaMember::NestedType(nested) = member {
            collect_generated_implementors(nested, interface, output);
        }
    }
}
