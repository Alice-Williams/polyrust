//! Java AST: expressions.

use super::expression_model::{
    JavaCallableRef, JavaLiteral, JavaMemberOrigin, JavaPrecedence, JavaValueRef,
};
use super::expression_nodes::{JavaConstructorRef, JavaExpr, JavaExprKind, JavaFieldRef};
use super::field_metadata::structural_field_metadata;
use super::generated_members::{
    generated_enum_variant_matches, generated_field_matches, generated_value_matches,
};
use super::identifiers::JavaIdentifier;
use super::invocations::{
    java_cast_is_legal, java_instanceof_is_legal, java_type_is_reifiable, verify_call,
};
use super::operator_signatures::{
    binary_signature_matches, invocation_types_match, literal_matches_type, unary_signature_matches,
};
use super::types::{
    JavaArrayOwnership, JavaArrayOwnershipTransition, JavaPrimitive, JavaType, JavaTypeName,
    JavaTypeUse, type_error,
};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedSymbolId, TargetAstContext, TargetSymbolRef};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

impl JavaExpr {
    pub fn literal(ty: JavaType, literal: JavaLiteral) -> Self {
        Self {
            ty,
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Literal(literal),
        }
    }

    pub fn local(ty: JavaType, name: JavaIdentifier) -> Self {
        Self {
            ty,
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::Local(name)),
        }
    }

    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        self.ty.symbols(symbols);
        match &self.kind {
            JavaExprKind::Literal(_) => {}
            JavaExprKind::Value(value) => match value {
                JavaValueRef::Generated(value) => {
                    symbols.insert(TargetSymbolRef::Generated(*value));
                }
                JavaValueRef::EnumVariant {
                    enumeration,
                    variant,
                } => {
                    symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(
                        *enumeration,
                    )));
                    symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Value(
                        *variant,
                    )));
                }
                JavaValueRef::KnownField(value) => {
                    symbols.insert(TargetSymbolRef::KnownField(*value));
                    symbols.insert(TargetSymbolRef::KnownType(value.owner()));
                }
                JavaValueRef::Local(_) | JavaValueRef::This => {}
            },
            JavaExprKind::Unary { operand, .. } => operand.symbols(symbols),
            JavaExprKind::Binary { left, right, .. } => {
                left.symbols(symbols);
                right.symbols(symbols);
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } => {
                condition.symbols(symbols);
                when_true.symbols(symbols);
                when_false.symbols(symbols);
            }
            JavaExprKind::Call {
                callable,
                receiver,
                arguments,
            } => {
                match callable {
                    JavaCallableRef::Known { callable, .. } => {
                        symbols.insert(TargetSymbolRef::KnownCallable(*callable));
                        symbols.insert(TargetSymbolRef::KnownType(callable.owner()));
                    }
                    JavaCallableRef::Runtime { callable, .. } => {
                        symbols.insert(TargetSymbolRef::RuntimeCallable(*callable));
                        symbols.insert(TargetSymbolRef::RuntimeHelper(callable.helper()));
                    }
                    JavaCallableRef::Generated { symbol, .. } => {
                        symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Callable(
                            *symbol,
                        )));
                    }
                    JavaCallableRef::Interface { symbol, .. } => {
                        symbols.insert(TargetSymbolRef::Generated(
                            GeneratedSymbolId::InterfaceMethod(*symbol),
                        ));
                    }
                    JavaCallableRef::Member { owner, origin, .. } => {
                        owner.symbols(symbols);
                        if let JavaMemberOrigin::Known(value) = origin {
                            symbols.insert(TargetSymbolRef::KnownMethod(*value));
                        }
                    }
                }
                if let Some(receiver) = receiver {
                    receiver.symbols(symbols);
                }
                for argument in arguments {
                    argument.symbols(symbols);
                }
            }
            JavaExprKind::New {
                constructor,
                arguments,
            } => {
                match constructor {
                    JavaConstructorRef::Known {
                        constructor,
                        owner,
                        parameters,
                    } => {
                        symbols.insert(TargetSymbolRef::KnownConstructor(*constructor));
                        owner.symbols(symbols);
                        for parameter in parameters {
                            parameter.symbols(symbols);
                        }
                    }
                    JavaConstructorRef::Generated { owner, .. } => {
                        symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(*owner)));
                    }
                }
                for argument in arguments {
                    argument.symbols(symbols);
                }
            }
            JavaExprKind::NewArray { component, length } => {
                component.symbols(symbols);
                length.symbols(symbols);
            }
            JavaExprKind::ArrayIndex { array, index } => {
                array.symbols(symbols);
                index.symbols(symbols);
            }
            JavaExprKind::Field { receiver, field } => {
                receiver.symbols(symbols);
                match field {
                    JavaFieldRef::Known(value) => {
                        symbols.insert(TargetSymbolRef::KnownField(*value));
                        symbols.insert(TargetSymbolRef::KnownType(value.owner()));
                    }
                    JavaFieldRef::Structural { ty, .. } => ty.symbols(symbols),
                    JavaFieldRef::Generated { owner, ty, .. } => {
                        symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(*owner)));
                        ty.symbols(symbols);
                    }
                }
            }
            JavaExprKind::Cast { target, value }
            | JavaExprKind::InterfaceCoercion {
                target,
                value,
                implementation: _,
            } => {
                target.symbols(symbols);
                value.symbols(symbols);
            }
            JavaExprKind::ArrayOwnershipTransition { value, .. } => value.symbols(symbols),
            JavaExprKind::InstanceOf { value, target, .. } => {
                value.symbols(symbols);
                target.symbols(symbols);
            }
            JavaExprKind::Lambda { parameters, body } => {
                for parameter in parameters {
                    parameter.ty.symbols(symbols);
                }
                body.symbols(symbols);
            }
        }
    }

    pub fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        let expression_type_use = if matches!(self.kind, JavaExprKind::Call { .. })
            && matches!(self.ty, JavaType::Wildcard { .. })
        {
            JavaTypeUse::GenericArgument
        } else {
            JavaTypeUse::Value
        };
        let mut violations = self.ty.verify(expression_type_use);
        match &self.kind {
            JavaExprKind::Literal(JavaLiteral::CharScalar(value))
                if char::from_u32(*value).is_none() =>
            {
                violations.push(type_error("character literal is not a Unicode scalar"));
            }
            JavaExprKind::Literal(literal) => {
                if !literal_matches_type(literal, &self.ty) {
                    violations.push(type_error("literal does not match its declared Java type"));
                }
            }
            JavaExprKind::Value(value) => match value {
                JavaValueRef::Generated(GeneratedSymbolId::Value(id)) => {
                    match generated_value_matches(*id, &self.ty, context) {
                        Some(true) => {}
                        Some(false) => violations.push(type_error(
                            "generated Java value reference type disagrees with its authoritative registration",
                        )),
                        None => violations.push(AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        "generated Java value reference is not registered",
                        )),
                    }
                }
                JavaValueRef::Generated(_) => violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "Java value expression references a generated symbol from the wrong category",
                )),
                JavaValueRef::EnumVariant {
                    enumeration,
                    variant,
                } => {
                    match generated_enum_variant_matches(
                        *enumeration,
                        *variant,
                        &self.ty,
                        context,
                    ) {
                        Some(true) => {}
                        Some(false) => violations.push(type_error(
                            "generated Java enum variant reference disagrees with its declaration",
                        )),
                        None => violations.push(AstViolation::new(
                            DiagnosticCode::UnresolvedReference,
                            "generated Java enum variant reference is not registered",
                        )),
                    }
                }
                JavaValueRef::KnownField(field) if self.ty != field.ty() => {
                    violations.push(type_error(
                        "known Java field reference type disagrees with its catalogue entry",
                    ));
                }
                JavaValueRef::KnownField(_) | JavaValueRef::Local(_) | JavaValueRef::This => {}
            },
            JavaExprKind::Unary { operator, operand } => {
                violations.extend(operand.verify(context));
                if !unary_signature_matches(*operator, &operand.ty, &self.ty) {
                    violations.push(type_error("unary operator type mismatch"));
                }
            }
            JavaExprKind::Binary {
                operator,
                left,
                right,
            } => {
                violations.extend(left.verify(context));
                violations.extend(right.verify(context));
                if !binary_signature_matches(*operator, &left.ty, &right.ty, &self.ty) {
                    violations.push(type_error("binary operator type mismatch"));
                }
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } => {
                violations.extend(condition.verify(context));
                violations.extend(when_true.verify(context));
                violations.extend(when_false.verify(context));
                if condition.ty != JavaType::Primitive(JavaPrimitive::Boolean)
                    || when_true.ty != self.ty
                    || when_false.ty != self.ty
                {
                    violations.push(type_error("conditional operand type mismatch"));
                }
            }
            JavaExprKind::Call {
                callable,
                receiver,
                arguments,
            } => {
                match callable.signature(context) {
                    Some(signature) => verify_call(
                        &mut violations,
                        &signature,
                        receiver.as_deref(),
                        arguments,
                        &self.ty,
                        context,
                    ),
                    None => violations.push(AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        format!(
                            "callable is absent or violates its catalogue signature: {callable:?}"
                        ),
                    )),
                }
                if let Some(receiver) = receiver {
                    violations.extend(receiver.verify(context));
                }
                for argument in arguments {
                    violations.extend(argument.verify(context));
                }
            }
            JavaExprKind::New {
                constructor,
                arguments,
            } => {
                match constructor.signature(context) {
                    Some((owner, parameters))
                        if owner == self.ty
                            && parameters.len() == arguments.len()
                            && parameters.iter().zip(arguments).all(|(expected, actual)| {
                                invocation_types_match(expected, &actual.ty)
                            }) => {}
                    _ => {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidInvocation,
                            "constructor arguments do not match its authoritative signature",
                        ));
                    }
                }
                for argument in arguments {
                    violations.extend(argument.verify(context));
                }
            }
            JavaExprKind::NewArray { component, length } => {
                violations.extend(component.verify(JavaTypeUse::Value));
                violations.extend(length.verify(context));
                let expected = JavaType::Array {
                    component: Box::new(component.clone()),
                    ownership: JavaArrayOwnership::InternalMutable,
                };
                if self.ty != expected || length.ty != JavaType::Primitive(JavaPrimitive::Int) {
                    violations.push(type_error(
                        "new-array component, length, or result type mismatch",
                    ));
                }
            }
            JavaExprKind::ArrayIndex { array, index } => {
                violations.extend(array.verify(context));
                violations.extend(index.verify(context));
                let component = match &array.ty {
                    JavaType::Array { component, .. } => Some(component.as_ref()),
                    _ => None,
                };
                if component != Some(&self.ty)
                    || index.ty != JavaType::Primitive(JavaPrimitive::Int)
                {
                    violations.push(type_error("array-index operand or result type mismatch"));
                }
            }
            JavaExprKind::Field { receiver, field } => {
                violations.extend(receiver.verify(context));
                if field.ty() != self.ty {
                    violations.push(type_error("field result type mismatch"));
                }
                match field {
                    JavaFieldRef::Known(value) => {
                        if receiver.ty != JavaType::known(value.owner()) {
                            violations.push(AstViolation::new(
                                DiagnosticCode::UnresolvedReference,
                                "known Java field receiver does not match its catalogue owner",
                            ));
                        }
                    }
                    JavaFieldRef::Structural { name, ty } => {
                        let metadata = structural_field_metadata(&receiver.ty, name, context);
                        let deferred_runtime_owner = metadata.is_none()
                            && matches!(
                                receiver.ty,
                                JavaType::Reference(JavaTypeName::Known(known))
                                    | JavaType::Generic {
                                        raw: JavaTypeName::Known(known),
                                        ..
                                    } if known.runtime_helper().is_some()
                            );
                        if metadata
                            .as_ref()
                            .is_some_and(|metadata| metadata.ty != *ty)
                            || (metadata.is_none() && !deferred_runtime_owner)
                        {
                            violations.push(AstViolation::new(
                                DiagnosticCode::UnresolvedReference,
                                format!(
                                    "structural Java field reference {:?}.{} does not name a field with type {ty:?}",
                                    receiver.ty,
                                    name.as_str()
                                ),
                            ));
                        }
                    }
                    JavaFieldRef::Generated {
                        owner,
                        field,
                        name,
                        ty,
                    } => {
                        if receiver.ty
                            != JavaType::Reference(JavaTypeName::Generated(*owner))
                            || !generated_field_matches(*owner, *field, name, ty, context)
                        {
                            violations.push(AstViolation::new(
                                DiagnosticCode::UnresolvedReference,
                                "generated Java field reference does not match its declared receiver/owner/name/type",
                            ));
                        }
                    }
                }
            }
            JavaExprKind::Cast { target, value } => {
                violations.extend(target.verify(JavaTypeUse::Value));
                violations.extend(value.verify(context));
                if target != &self.ty {
                    violations.push(type_error("cast target type mismatch"));
                }
                if target == &value.ty {
                    violations.push(type_error(
                        "Java cast is redundant and would fail javac -Xlint:cast -Werror",
                    ));
                } else if !java_cast_is_legal(target, &value.ty, context) {
                    violations.push(type_error(
                        "Java cast is not legal between the declared source and target types",
                    ));
                }
            }
            JavaExprKind::InterfaceCoercion {
                implementation,
                target,
                value,
            } => {
                violations.extend(implementation.verify(&value.ty, target, context));
                violations.extend(target.verify(JavaTypeUse::Value));
                violations.extend(value.verify(context));
                if target != &self.ty {
                    violations.push(type_error("interface coercion target type mismatch"));
                }
                if target == &value.ty {
                    violations.push(type_error(
                        "Java interface coercion is redundant and would fail javac -Xlint:cast -Werror",
                    ));
                } else if !java_cast_is_legal(target, &value.ty, context) {
                    violations.push(type_error(
                        "Java interface coercion is not legal between the declared source and target types",
                    ));
                }
            }
            JavaExprKind::ArrayOwnershipTransition { transition, value } => {
                violations.extend(value.verify(context));
                let valid = match transition {
                    JavaArrayOwnershipTransition::FreshCopyToBoundary => matches!(
                        (&value.ty, &self.ty),
                        (
                            JavaType::Array {
                                component: source,
                                ownership: JavaArrayOwnership::InternalMutable,
                            },
                            JavaType::Array {
                                component: target,
                                ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                            },
                        ) if source == target
                    ),
                };
                if !valid {
                    violations.push(type_error("invalid Java array ownership transition"));
                }
            }
            JavaExprKind::InstanceOf { value, target, .. } => {
                violations.extend(value.verify(context));
                violations.extend(target.verify(JavaTypeUse::Value));
                if self.ty != JavaType::Primitive(JavaPrimitive::Boolean) {
                    violations.push(type_error("instanceof result must be boolean"));
                }
                if !java_type_is_reifiable(target) {
                    violations.push(type_error("Java instanceof target must be reifiable"));
                }
                if !java_instanceof_is_legal(&value.ty, target, context) {
                    violations.push(type_error(&format!(
                        "Java instanceof is not legal from {:?} to {target:?}",
                        value.ty
                    )));
                }
            }
            JavaExprKind::Lambda { parameters, body } => {
                for parameter in parameters {
                    violations.extend(parameter.ty.verify(JavaTypeUse::Parameter));
                }
                violations.extend(body.verify(context));
                violations.push(type_error(
                    "lambda targets are not part of the portable Java dialect",
                ));
            }
        }
        violations
    }
}
