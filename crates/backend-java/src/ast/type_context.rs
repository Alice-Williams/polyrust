//! Java AST: type context.

use super::expression_model::JavaCallableRef;
use super::expression_nodes::{JavaConstructorRef, JavaExpr, JavaExprKind};
use super::field_metadata::find_type_declaration;
use super::identifiers::JavaIdentifier;
use super::statement_context::verify_block_type_context;
use super::types::{JavaKnownType, JavaPrimitive, JavaType, JavaTypeName, type_error};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum JavaErasedType {
    Primitive(JavaPrimitive),
    Reference(JavaTypeName),
    Array(Box<JavaErasedType>),
}

pub(super) fn erased_java_type(ty: &JavaType) -> JavaErasedType {
    match ty {
        JavaType::Primitive(value) => JavaErasedType::Primitive(*value),
        JavaType::Boxed(value) => JavaErasedType::Reference(JavaTypeName::Known(match value {
            JavaPrimitive::Boolean => JavaKnownType::Boolean,
            JavaPrimitive::Byte => JavaKnownType::Byte,
            JavaPrimitive::Char => JavaKnownType::Character,
            JavaPrimitive::Int => JavaKnownType::Integer,
            JavaPrimitive::Long => JavaKnownType::Long,
            JavaPrimitive::Double => JavaKnownType::Double,
            JavaPrimitive::Void => JavaKnownType::Object,
        })),
        JavaType::Reference(name) | JavaType::Generic { raw: name, .. } => {
            JavaErasedType::Reference(name.clone())
        }
        JavaType::Array { component, .. } => {
            JavaErasedType::Array(Box::new(erased_java_type(component)))
        }
        JavaType::Wildcard { .. } | JavaType::TypeVariable(_) => {
            JavaErasedType::Reference(JavaTypeName::Known(JavaKnownType::Object))
        }
    }
}

fn known_generic_arity(known: JavaKnownType) -> usize {
    match known {
        JavaKnownType::ArrayList
        | JavaKnownType::List
        | JavaKnownType::RuntimeResult
        | JavaKnownType::RuntimeOption => 1,
        JavaKnownType::LinkedHashMap | JavaKnownType::Map | JavaKnownType::RuntimeValueResult => 2,
        _ => 0,
    }
}

pub(super) fn verify_contextual_type(
    ty: &JavaType,
    variables: &BTreeSet<JavaIdentifier>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    match ty {
        JavaType::Reference(JavaTypeName::Known(known)) => {
            if known_generic_arity(*known) != 0 {
                violations.push(type_error(
                    "generic Java known type cannot be used as a raw reference",
                ));
            }
        }
        JavaType::Reference(JavaTypeName::Generated(id)) => {
            match find_type_declaration(ty, context) {
                Some(declaration) if declaration.type_parameters.is_empty() => {}
                Some(_) => violations.push(type_error(
                    "generic generated Java type cannot be used as a raw reference",
                )),
                None => violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    format!("generated Java type {id:?} has no AST declaration"),
                )),
            }
        }
        JavaType::Generic { raw, arguments } => {
            let expected = match raw {
                JavaTypeName::Known(known) => Some(known_generic_arity(*known)),
                JavaTypeName::Generated(_) => find_type_declaration(ty, context)
                    .map(|declaration| declaration.type_parameters.len()),
            };
            match expected {
                Some(arity) if arity == arguments.len() && arity != 0 => {}
                Some(arity) => violations.push(type_error(&format!(
                    "Java generic type requires exactly {arity} type arguments"
                ))),
                None => violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "generic generated Java type has no AST declaration",
                )),
            }
            for argument in arguments {
                violations.extend(verify_contextual_type(argument, variables, context));
            }
        }
        JavaType::Array { component, .. } => {
            violations.extend(verify_contextual_type(component, variables, context));
        }
        JavaType::Wildcard {
            bound: Some((_, bound)),
        } => {
            violations.extend(verify_contextual_type(bound, variables, context));
        }
        JavaType::TypeVariable(name) if !variables.contains(name) => {
            violations.push(AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                format!(
                    "Java type variable {:?} is not declared in this type/member scope",
                    name.as_str()
                ),
            ));
        }
        JavaType::Primitive(_)
        | JavaType::Boxed(_)
        | JavaType::Wildcard { bound: None }
        | JavaType::TypeVariable(_) => {}
    }
    violations
}

pub(super) fn verify_expression_type_context(
    expression: &JavaExpr,
    variables: &BTreeSet<JavaIdentifier>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = verify_contextual_type(&expression.ty, variables, context);
    match &expression.kind {
        JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {}
        JavaExprKind::Unary { operand, .. } => {
            violations.extend(verify_expression_type_context(operand, variables, context));
        }
        JavaExprKind::Binary { left, right, .. } => {
            violations.extend(verify_expression_type_context(left, variables, context));
            violations.extend(verify_expression_type_context(right, variables, context));
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            violations.extend(verify_expression_type_context(
                condition, variables, context,
            ));
            violations.extend(verify_expression_type_context(
                when_true, variables, context,
            ));
            violations.extend(verify_expression_type_context(
                when_false, variables, context,
            ));
        }
        JavaExprKind::Call {
            callable,
            receiver,
            arguments,
        } => {
            let signature = match callable {
                JavaCallableRef::Known { signature, .. }
                | JavaCallableRef::Runtime { signature, .. }
                | JavaCallableRef::Generated { signature, .. }
                | JavaCallableRef::Interface { signature, .. }
                | JavaCallableRef::Member { signature, .. } => signature,
            };
            if let JavaCallableRef::Member { owner, .. } = callable {
                violations.extend(verify_contextual_type(owner, variables, context));
            }
            if let Some(receiver_type) = &signature.receiver {
                violations.extend(verify_contextual_type(receiver_type, variables, context));
            }
            for parameter in &signature.parameters {
                violations.extend(verify_contextual_type(parameter, variables, context));
            }
            violations.extend(verify_contextual_type(
                &signature.result,
                variables,
                context,
            ));
            if let Some(receiver) = receiver {
                violations.extend(verify_expression_type_context(receiver, variables, context));
            }
            for argument in arguments {
                violations.extend(verify_expression_type_context(argument, variables, context));
            }
        }
        JavaExprKind::New {
            constructor,
            arguments,
        } => {
            match constructor {
                JavaConstructorRef::Known {
                    owner, parameters, ..
                } => {
                    violations.extend(verify_contextual_type(owner, variables, context));
                    for parameter in parameters {
                        violations.extend(verify_contextual_type(parameter, variables, context));
                    }
                }
                JavaConstructorRef::Generated { owner, parameters } => {
                    violations.extend(verify_contextual_type(
                        &JavaType::Reference(JavaTypeName::Generated(*owner)),
                        variables,
                        context,
                    ));
                    for parameter in parameters {
                        violations.extend(verify_contextual_type(parameter, variables, context));
                    }
                }
            }
            for argument in arguments {
                violations.extend(verify_expression_type_context(argument, variables, context));
            }
        }
        JavaExprKind::NewArray { component, length } => {
            violations.extend(verify_contextual_type(component, variables, context));
            violations.extend(verify_expression_type_context(length, variables, context));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            violations.extend(verify_expression_type_context(array, variables, context));
            violations.extend(verify_expression_type_context(index, variables, context));
        }
        JavaExprKind::Field { receiver, field } => {
            violations.extend(verify_expression_type_context(receiver, variables, context));
            violations.extend(verify_contextual_type(&field.ty(), variables, context));
        }
        JavaExprKind::Cast { target, value }
        | JavaExprKind::InterfaceCoercion {
            target,
            value,
            implementation: _,
        }
        | JavaExprKind::InstanceOf {
            target,
            value,
            binding: _,
        } => {
            violations.extend(verify_contextual_type(target, variables, context));
            violations.extend(verify_expression_type_context(value, variables, context));
        }
        JavaExprKind::ArrayOwnershipTransition { value, .. } => {
            violations.extend(verify_expression_type_context(value, variables, context));
        }
        JavaExprKind::Lambda { parameters, body } => {
            for parameter in parameters {
                violations.extend(verify_contextual_type(&parameter.ty, variables, context));
            }
            violations.extend(verify_block_type_context(body, variables, context));
        }
    }
    violations
}
