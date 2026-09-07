//! Java AST: initializers.

use super::declaration_model::{JavaField, JavaHeritage, JavaMember, JavaTypeDeclaration};
use super::exceptions::expr_checked_exceptions;
use super::expression_nodes::JavaExpr;
use super::final_assignment::{collect_blank_final_reads, declaration_blank_instance_finals};
use super::lexical_expressions::verify_expr_scope;
use super::lexical_scope::JavaLexicalScope;
use super::statement_context::verify_member_type_context;
use super::type_context::verify_contextual_type;
use super::types::JavaType;
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedSymbolId, TargetAstContext, TargetSymbolRef};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn verify_declaration_type_context(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let variables = declaration
        .type_parameters
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for component in &declaration.record_components {
        violations.extend(verify_contextual_type(&component.ty, &variables, context));
    }
    if let JavaHeritage::Interfaces(interfaces) = &declaration.heritage {
        for interface in interfaces {
            violations.extend(verify_contextual_type(interface, &variables, context));
        }
    }
    for permitted in &declaration.permits {
        violations.extend(verify_contextual_type(permitted, &variables, context));
    }
    for member in &declaration.members {
        violations.extend(verify_member_type_context(member, &variables, context));
    }
    violations
}

pub(super) fn verify_field_initializer_context(
    field: &JavaField,
    value: &JavaExpr,
    owner: Option<&JavaType>,
    declaration: Option<&JavaTypeDeclaration>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let scope = JavaLexicalScope::for_field_initializer(field, owner.cloned(), declaration);
    let mut violations = verify_expr_scope(value, &scope);
    if let Some(declaration) = declaration {
        let blank_finals = declaration_blank_instance_finals(declaration);
        let mut reads = BTreeSet::new();
        collect_blank_final_reads(value, &blank_finals, &mut reads);
        for field in reads {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                format!(
                    "Java field initializer reads blank final field `{}` before constructor assignment",
                    field.as_str()
                ),
            ));
        }
    }
    let unhandled = expr_checked_exceptions(value, context);
    if !unhandled.is_empty() {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidInvocation,
            format!("Java field initializer has unhandled checked exceptions: {unhandled:?}"),
        ));
    }
    violations
}

pub(super) fn verify_field_initializer_declaration_order(
    declaration: &JavaTypeDeclaration,
) -> Vec<AstViolation> {
    let local_values = declaration
        .members
        .iter()
        .filter_map(|member| match member {
            JavaMember::Field(JavaField {
                declared: Some(value),
                ..
            }) => Some(*value),
            JavaMember::Field(JavaField { declared: None, .. })
            | JavaMember::CompileFailField(_)
            | JavaMember::EnumConstant(_)
            | JavaMember::Method(_)
            | JavaMember::Constructor(_)
            | JavaMember::NestedType(_) => None,
        })
        .collect::<BTreeSet<_>>();
    let mut available = BTreeSet::new();
    let mut violations = Vec::new();
    for member in &declaration.members {
        let JavaMember::Field(field) = member else {
            continue;
        };
        if let Some(initializer) = &field.initializer {
            let mut symbols = BTreeSet::new();
            initializer.symbols(&mut symbols);
            for value in symbols.into_iter().filter_map(|symbol| match symbol {
                TargetSymbolRef::Generated(GeneratedSymbolId::Value(value)) => Some(value),
                _ => None,
            }) {
                if local_values.contains(&value) && !available.contains(&value) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        "Java generated field initializer references itself or a later field in the same declaration",
                    ));
                }
            }
        }
        if let Some(value) = field.declared {
            available.insert(value);
        }
    }
    violations
}
