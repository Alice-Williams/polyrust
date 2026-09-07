//! Java AST: members.

use super::completion::block_guarantees_exit;
use super::constructor_flow::verify_constructor_final_assignments;
use super::declaration_model::{
    JavaDeclarationKind, JavaMember, JavaMethodDeclaration, JavaModifier, JavaTypeDeclaration,
};
use super::exceptions::block_checked_exceptions;
use super::generated_members::generated_value_matches;
use super::initializers::verify_field_initializer_context;
use super::lexical_blocks::verify_block_scope_in_context;
use super::lexical_scope::JavaLexicalScope;
use super::method_contracts::verify_method_annotations;
use super::modifiers::{JavaModifierSite, verify_modifiers_for};
use super::operator_signatures::invocation_types_match;
use super::types::{JavaPrimitive, JavaType, JavaTypeName, JavaTypeUse, type_error};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedSymbolId, TargetAstContext, TargetSymbolRef};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

impl JavaMember {
    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        match self {
            Self::Field(field) => {
                if let Some(value) = field.declared {
                    symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Value(value)));
                }
                field.ty.symbols(symbols);
                if let Some(value) = &field.initializer {
                    value.symbols(symbols);
                }
            }
            Self::CompileFailField(field) => {
                field.expected_type.symbols(symbols);
                field.initializer.symbols(symbols);
            }
            Self::EnumConstant(value) => {
                symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Value(
                    value.declared,
                )));
            }
            Self::Method(method) => {
                match method.declared {
                    JavaMethodDeclaration::Callable(value) => {
                        symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Callable(
                            value,
                        )));
                    }
                    JavaMethodDeclaration::Interface(value)
                    | JavaMethodDeclaration::UninhabitedImplementation(value)
                    | JavaMethodDeclaration::Implementation {
                        interface: value, ..
                    } => {
                        symbols.insert(TargetSymbolRef::Generated(
                            GeneratedSymbolId::InterfaceMethod(value),
                        ));
                    }
                    JavaMethodDeclaration::Structural => {}
                }
                method.return_type.symbols(symbols);
                for parameter in &method.parameters {
                    parameter.ty.symbols(symbols);
                }
                if let Some(body) = &method.body {
                    body.symbols(symbols);
                }
            }
            Self::Constructor(constructor) => {
                for parameter in &constructor.parameters {
                    parameter.ty.symbols(symbols);
                }
                constructor.body.symbols(symbols);
            }
            Self::NestedType(value) => value.symbols(symbols),
        }
    }

    pub fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        self.verify_with_owner(context, None, None)
    }

    pub(super) fn verify_with_owner(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
        owner: Option<&JavaType>,
        declaration: Option<&JavaTypeDeclaration>,
    ) -> Vec<AstViolation> {
        match self {
            Self::Field(field) => {
                let mut violations =
                    verify_modifiers_for(&field.modifiers, JavaModifierSite::Field);
                violations.extend(field.ty.verify(JavaTypeUse::Field));
                violations.extend(super::field_registration::verify(field, context));
                if field.initializer.is_none()
                    && field.modifiers.contains(&JavaModifier::Static)
                    && field.modifiers.contains(&JavaModifier::Final)
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        "blank static final Java fields require a static initializer, which the typed AST does not admit",
                    ));
                }
                if let Some(value) = &field.initializer {
                    violations.extend(value.verify(context));
                    violations.extend(verify_field_initializer_context(
                        field,
                        value,
                        owner,
                        declaration,
                        context,
                    ));
                    if value.ty != field.ty {
                        violations.push(type_error("field initializer type mismatch"));
                    }
                }
                violations
            }
            Self::CompileFailField(field) => {
                let mut violations =
                    verify_modifiers_for(&field.modifiers, JavaModifierSite::Field);
                violations.extend(field.expected_type.verify(JavaTypeUse::Field));
                violations.extend(field.initializer.verify(context));
                if invocation_types_match(&field.expected_type, &field.initializer.ty) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "compile-fail field must contain a deliberate type mismatch",
                    ));
                }
                violations
            }
            Self::EnumConstant(value) => {
                let Some(declaration) = declaration else {
                    return vec![AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java enum constant must belong to an enum declaration",
                    )];
                };
                let Some(owner) = declaration.declared else {
                    return vec![AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        "Java enum declaration must have a generated type identity",
                    )];
                };
                let ty = JavaType::Reference(JavaTypeName::Generated(owner));
                if declaration.kind == JavaDeclarationKind::Enum
                    && generated_value_matches(value.declared, &ty, context) == Some(true)
                    && context.value(value.declared).is_some_and(|registered| {
                        registered.name == value.name.as_str()
                            && registered.visibility == super::JavaVisibility::Public
                            && match registered.origin {
                                portable_codegen::GeneratedOrigin::CoreDeclaration(
                                    portable_core_ir::CoreDeclaration::Enum(id),
                                ) => context.generated_type(owner).is_some_and(|owner| {
                                    owner.origin
                                        == portable_codegen::GeneratedOrigin::CoreDeclaration(
                                            portable_core_ir::CoreDeclaration::Enum(id),
                                        )
                                }),
                                portable_codegen::GeneratedOrigin::CoreDeclaration(_) => false,
                                _ => true,
                            }
                    })
                {
                    vec![]
                } else {
                    vec![AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java enum constant must be registered with its declaring enum type",
                    )]
                }
            }
            Self::Method(method) => {
                let mut violations =
                    verify_modifiers_for(&method.modifiers, JavaModifierSite::Method);
                violations.extend(verify_method_annotations(method, declaration, context));
                violations.extend(method.return_type.verify(JavaTypeUse::Return));
                let distinct_type_parameters =
                    method.type_parameters.iter().collect::<BTreeSet<_>>();
                if distinct_type_parameters.len() != method.type_parameters.len() {
                    violations.push(AstViolation::new(
                        DiagnosticCode::DuplicateDeclaration,
                        "Java method type parameter is declared more than once",
                    ));
                }
                for parameter in &method.parameters {
                    violations.extend(parameter.ty.verify(JavaTypeUse::Parameter));
                }
                if method.body.is_none() != method.modifiers.contains(&JavaModifier::Abstract) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "abstract methods have no body and concrete methods have a body",
                    ));
                }
                let (mut scope, scope_violations) = JavaLexicalScope::for_method_in_declaration(
                    method,
                    owner.cloned(),
                    declaration,
                );
                violations.extend(scope_violations);
                violations.extend(scope.protect_qualifiers(context));
                if let Some(body) = &method.body {
                    violations.extend(body.verify(context));
                    violations.extend(verify_block_scope_in_context(
                        body,
                        &mut scope,
                        &method.return_type,
                        false,
                        Some(context),
                    ));
                    if method.return_type != JavaType::primitive(JavaPrimitive::Void)
                        && !block_guarantees_exit(body)
                    {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidControlFlow,
                            "concrete non-void Java method does not return or throw on every path",
                        ));
                    }
                    let unhandled = block_checked_exceptions(body, context);
                    if !unhandled.is_empty() {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidInvocation,
                            format!("Java method has unhandled checked exceptions: {unhandled:?}"),
                        ));
                    }
                }
                violations
            }
            Self::Constructor(constructor) => {
                let mut violations =
                    verify_modifiers_for(&constructor.modifiers, JavaModifierSite::Constructor);
                for parameter in &constructor.parameters {
                    violations.extend(parameter.ty.verify(JavaTypeUse::Parameter));
                }
                violations.extend(constructor.body.verify(context));
                let (mut scope, scope_violations) =
                    JavaLexicalScope::for_constructor_in_declaration(
                        constructor,
                        owner.cloned(),
                        declaration,
                    );
                violations.extend(scope_violations);
                violations.extend(verify_block_scope_in_context(
                    &constructor.body,
                    &mut scope,
                    &JavaType::primitive(JavaPrimitive::Void),
                    false,
                    Some(context),
                ));
                if let Some(declaration) = declaration {
                    violations.extend(verify_constructor_final_assignments(
                        constructor,
                        declaration,
                    ));
                }
                let unhandled = block_checked_exceptions(&constructor.body, context);
                if !unhandled.is_empty() {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidInvocation,
                        format!("Java constructor has unhandled checked exceptions: {unhandled:?}"),
                    ));
                }
                violations
            }
            Self::NestedType(value) => value.verify(context, false),
        }
    }
}
