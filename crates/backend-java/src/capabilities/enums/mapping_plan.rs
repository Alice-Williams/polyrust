//! Native enum and payload representation certificates.
use super::{JavaEnumEqualityOperator, JavaEnumShape, JavaEnumsInput, JavaEnumsNode, enum_type};
use crate::ast::{
    JavaBinaryOperator, JavaConstructorRef, JavaDeclarationKind, JavaExprKind, JavaHeritage,
    JavaKnownType, JavaMember, JavaModifier, JavaPattern, JavaPrimitive, JavaStmt, JavaType,
    JavaValueRef,
};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
use crate::lower::{identifier, java_visibility, string_literal};
java_input_plan!(JavaEnumsInput, JavaEnumsNode);

fn representation(input: &JavaEnumsInput) -> R {
    match input {
        JavaEnumsInput::PayloadDeclaration { .. } | JavaEnumsInput::PayloadConstruction { .. } => {
            R::TaggedValue
        }
        JavaEnumsInput::Declaration { .. } => R::Declaration,
        JavaEnumsInput::Branch { .. } => R::StructuredControl,
        JavaEnumsInput::Type { shape, .. } => match shape {
            JavaEnumShape::Native => R::Direct,
            JavaEnumShape::Payload => R::TaggedValue,
        },
        JavaEnumsInput::Variant { .. } | JavaEnumsInput::Equality { .. } => R::Direct,
    }
}
fn verify(input: &JavaEnumsInput, output: &JavaEnumsNode) -> bool {
    match input {
        JavaEnumsInput::PayloadDeclaration {
            declared,
            visibility,
            name,
            variants,
        } => {
            let JavaEnumsNode::Declaration(actual) = output else {
                return false;
            };
            let Some((head, tail)) = actual.split_first() else {
                return false;
            };
            head.declared == Some(*declared)
                && head.kind == JavaDeclarationKind::SealedInterface
                && head.visibility == java_visibility(*visibility)
                && head.modifiers == [JavaModifier::Static]
                && head.name == identifier(name)
                && head.type_parameters.is_empty()
                && head.record_components.is_empty()
                && head.heritage == JavaHeritage::None
                && head.members.is_empty()
                && head.permits
                    == variants
                        .iter()
                        .map(|v| enum_type(v.declared))
                        .collect::<Vec<_>>()
                && tail.len() == variants.len()
                && tail.iter().zip(variants).all(|(a, v)| {
                    a.declared == Some(v.declared)
                        && a.kind == JavaDeclarationKind::Record
                        && a.visibility == java_visibility(*visibility)
                        && a.modifiers == [JavaModifier::Static]
                        && a.name == identifier(&v.name)
                        && a.type_parameters.is_empty()
                        && a.record_components == v.components
                        && a.members == v.members
                        && a.permits.is_empty()
                        && a.heritage
                            == JavaHeritage::Interfaces(vec![
                                enum_type(*declared),
                                JavaType::known(JavaKnownType::RuntimeSemanticValue),
                            ])
                })
        }
        JavaEnumsInput::Declaration {
            declared,
            visibility,
            name,
            variants,
        } => {
            let JavaEnumsNode::Declaration(actual) = output else {
                return false;
            };
            let [a] = actual.as_slice() else {
                return false;
            };
            a.declared == Some(*declared) && a.kind == JavaDeclarationKind::Enum
                && a.visibility == java_visibility(*visibility) && a.modifiers.is_empty() && a.name == identifier(name)
                && a.type_parameters.is_empty() && a.record_components.is_empty() && a.heritage == JavaHeritage::None
                && a.permits.is_empty() && a.members.len() == variants.len()
                && a.members.iter().zip(variants).all(|(member, v)| matches!(member,
                    JavaMember::EnumConstant(c) if c.declared == v.declared && c.name == identifier(&v.name)))
        }
        JavaEnumsInput::Type { enumeration, .. } => {
            matches!(output, JavaEnumsNode::Type(a) if a == &enum_type(*enumeration))
        }
        JavaEnumsInput::Variant {
            enumeration,
            variant,
        } => matches!(output, JavaEnumsNode::Expression(a)
            if a.ty == enum_type(*enumeration) && a.kind == JavaExprKind::Value(JavaValueRef::EnumVariant { enumeration: *enumeration, variant: *variant })),
        JavaEnumsInput::Equality { operator, .. } => {
            let expected = match operator {
                JavaEnumEqualityOperator::Equal => JavaBinaryOperator::Equal,
                JavaEnumEqualityOperator::NotEqual => JavaBinaryOperator::NotEqual,
            };
            matches!(output, JavaEnumsNode::Expression(a) if a.ty == JavaType::primitive(JavaPrimitive::Boolean)
                && matches!(a.kind, JavaExprKind::Binary { operator, .. } if operator == expected))
        }
        JavaEnumsInput::PayloadConstruction {
            variant,
            arguments,
            result,
        } => {
            let JavaEnumsNode::Expression(a) = output else {
                return false;
            };
            if &a.ty != result {
                return false;
            }
            let created = if result == &enum_type(*variant) {
                a.as_ref()
            } else {
                let JavaExprKind::Cast { target, value } = &a.kind else {
                    return false;
                };
                if target != result {
                    return false;
                }
                value
            };
            created.ty == enum_type(*variant)
                && matches!(&created.kind,
                JavaExprKind::New { constructor: JavaConstructorRef::Generated { owner, parameters }, arguments: args }
                if owner == variant && parameters == &arguments.iter().map(|v| v.ty.clone()).collect::<Vec<_>>() && args.len() == arguments.len())
        }
        JavaEnumsInput::Branch {
            enumeration, arms, ..
        } => {
            let JavaEnumsNode::Statement(a) = output else {
                return false;
            };
            let JavaStmt::Switch { arms: actual, .. } = a.as_ref() else {
                return false;
            };
            let Some((fallback, actual)) = actual.split_last() else {
                return false;
            };
            actual.len() == arms.len()
                && actual.iter().zip(arms).all(|(a, i)| {
                    a.pattern
                        == JavaPattern::EnumVariant {
                            enumeration: *enumeration,
                            variant: i.variant,
                        }
                        && a.body == i.body
                })
                && fallback.pattern == JavaPattern::Default
                && matches!(
                    fallback.body.statements.as_slice(),
                    [JavaStmt::ThrowAssertion(message)] if message == &string_literal("unreachable exhaustive enum branch")
                )
        }
    }
}
