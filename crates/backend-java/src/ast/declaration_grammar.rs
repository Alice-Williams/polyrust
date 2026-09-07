//! Java AST: declaration grammar.

use super::declaration_model::{
    JavaDeclarationKind, JavaMember, JavaMethodDeclaration, JavaModifier, JavaTypeDeclaration,
    JavaVisibility,
};
use super::sealed_permits::java_visibility_rank;
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;

pub(super) fn verify_declaration_kind_grammar(
    declaration: &JavaTypeDeclaration,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    let interface = matches!(
        declaration.kind,
        JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface
    );
    let mut record_constructor_count = 0usize;
    let mut enum_constant_count = 0usize;
    for member in &declaration.members {
        match member {
            JavaMember::EnumConstant(_) if declaration.kind == JavaDeclarationKind::Enum => {
                enum_constant_count += 1;
            }
            JavaMember::EnumConstant(_) => {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "Java enum constants can only belong to enum declarations",
                ));
            }
            JavaMember::Field(_)
            | JavaMember::CompileFailField(_)
            | JavaMember::Method(_)
            | JavaMember::Constructor(_)
            | JavaMember::NestedType(_)
                if declaration.kind == JavaDeclarationKind::Enum =>
            {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "portable Java enums contain only payload-free constants",
                ));
            }
            JavaMember::Field(_) | JavaMember::CompileFailField(_) if interface => {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "portable Java interfaces cannot declare fields",
                ));
            }
            JavaMember::Constructor(_) if interface => {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "Java interfaces cannot declare constructors",
                ));
            }
            JavaMember::NestedType(_) if interface => {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "portable Java interfaces are flat and cannot declare nested types",
                ));
            }
            JavaMember::Method(method) if interface => {
                if !matches!(
                    method.declared,
                    JavaMethodDeclaration::Structural | JavaMethodDeclaration::Interface(_)
                ) || (declaration.declared.is_some()
                    && !matches!(method.declared, JavaMethodDeclaration::Interface(_)))
                    || method.body.is_some()
                    || !method.modifiers.contains(&JavaModifier::Abstract)
                    || method.modifiers.contains(&JavaModifier::Static)
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "generated Java interface members must use registered interface method declarations; all Java interface members must be abstract instance declarations",
                    ));
                }
            }
            JavaMember::Method(method) => {
                if matches!(method.declared, JavaMethodDeclaration::Interface(_)) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java interface method declarations must belong to an interface",
                    ));
                }
                if method.modifiers.contains(&JavaModifier::Abstract) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "records and final Java classes cannot declare abstract methods",
                    ));
                }
            }
            JavaMember::Field(field) if declaration.kind == JavaDeclarationKind::Record => {
                if !field.modifiers.contains(&JavaModifier::Static) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java records cannot declare additional instance fields",
                    ));
                }
            }
            JavaMember::CompileFailField(field)
                if declaration.kind == JavaDeclarationKind::Record =>
            {
                if !field.modifiers.contains(&JavaModifier::Static) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java records cannot declare additional instance fields",
                    ));
                }
            }
            JavaMember::Constructor(constructor)
                if declaration.kind == JavaDeclarationKind::Record =>
            {
                record_constructor_count += 1;
                let canonical = constructor.parameters.len() == declaration.record_components.len()
                    && constructor
                        .parameters
                        .iter()
                        .zip(&declaration.record_components)
                        .all(|(parameter, component)| {
                            parameter.name == component.name && parameter.ty == component.ty
                        });
                if !canonical {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "explicit Java record constructor must have the canonical component signature",
                    ));
                }
                let constructor_visibility =
                    if constructor.modifiers.contains(&JavaModifier::Public) {
                        JavaVisibility::Public
                    } else if constructor.modifiers.contains(&JavaModifier::Private) {
                        JavaVisibility::Private
                    } else {
                        JavaVisibility::Package
                    };
                if java_visibility_rank(constructor_visibility)
                    < java_visibility_rank(declaration.visibility)
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "canonical Java record constructor cannot be less accessible than its record",
                    ));
                }
            }
            JavaMember::Field(_)
            | JavaMember::CompileFailField(_)
            | JavaMember::Constructor(_)
            | JavaMember::NestedType(_) => {}
        }
    }
    if declaration.kind == JavaDeclarationKind::Enum && enum_constant_count == 0 {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "portable Java enum must declare at least one constant",
        ));
    }
    if declaration.kind == JavaDeclarationKind::Record && record_constructor_count > 1 {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "portable Java record may declare at most one canonical constructor",
        ));
    }
    violations
}
