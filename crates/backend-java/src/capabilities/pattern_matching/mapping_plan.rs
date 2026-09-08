//! Pattern and match-plan certificates.
mod patterns;
use super::*;
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
java_input_plan!(JavaPatternMatchingInput, JavaPatternMatchingNode);
fn representation(input: &JavaPatternMatchingInput) -> R {
    match input {
        JavaPatternMatchingInput::BindingRead { .. } => R::Direct,
        JavaPatternMatchingInput::Match(_) => R::StructuredControl,
        JavaPatternMatchingInput::Pattern(pattern) => match pattern.as_ref() {
            JavaPatternInput::Wildcard | JavaPatternInput::Bool { .. } => R::Direct,
            JavaPatternInput::None { .. } => R::RuntimeHelper,
            JavaPatternInput::EnumVariant { .. }
            | JavaPatternInput::Some { .. }
            | JavaPatternInput::Ok { .. }
            | JavaPatternInput::Err { .. } => R::StructuredControl,
        },
    }
}
fn verify(input: &JavaPatternMatchingInput, output: &JavaPatternMatchingNode) -> bool {
    match input {
        JavaPatternMatchingInput::BindingRead {
            binding_type,
            binding,
        } => matches!(output, JavaPatternMatchingNode::Expression(a)
            if a.as_ref() == &JavaExpr::local(binding_type.clone(), identifier(binding))),
        JavaPatternMatchingInput::Pattern(i) => {
            matches!(output, JavaPatternMatchingNode::Pattern(a) if patterns::verify(i, a))
        }
        JavaPatternMatchingInput::Match(i) => {
            let JavaPatternMatchingNode::Match(a) = output else {
                return false;
            };
            if a.value != JavaExpr::local(i.result_type.clone(), i.result_name.clone()) {
                return false;
            }
            let Some(owned) = a.statements.strip_prefix(i.prefix.as_slice()) else {
                return false;
            };
            let [
                JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: matched_ty,
                    name: matched_name,
                    value: Some(_),
                },
                JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty: result_ty,
                    name: result_name,
                    value: None,
                },
                rest @ ..,
            ] = owned
            else {
                return false;
            };
            if matched_ty != &i.matched.ty
                || matched_name != &i.matched_name
                || result_ty != &i.result_type
                || result_name != &i.result_name
            {
                return false;
            }
            let mut chain = rest;
            for arm in &i.arms {
                let [
                    JavaStmt::If {
                        condition,
                        then_block,
                        else_block: Some(else_block),
                    },
                ] = chain
                else {
                    return false;
                };
                if condition != &arm.pattern.condition {
                    return false;
                }
                let Some(body) = then_block
                    .statements
                    .strip_prefix(arm.pattern.bindings.as_slice())
                else {
                    return false;
                };
                if body != arm.body.statements {
                    return false;
                }
                chain = &else_block.statements;
            }
            matches!(chain, [JavaStmt::ThrowAssertion(message)] if message == &string_literal("verified CoreIR match was unexpectedly non-exhaustive"))
        }
    }
}
