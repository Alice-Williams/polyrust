//! Independent mapping-owned output checks.
use super::*;
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
java_input_plan!(JavaConstantsInput, JavaConstantsNode);
fn representation(input: &JavaConstantsInput) -> R {
    match input {
        JavaConstantsInput::Declaration { .. } => R::Declaration,
        JavaConstantsInput::Reference { .. } => R::Direct,
    }
}
fn verify(input: &JavaConstantsInput, output: &JavaConstantsNode) -> bool {
    match input {
        JavaConstantsInput::Declaration {
            declared,
            visibility,
            name,
            ty,
            ..
        } => matches!(output, JavaConstantsNode::Declaration(a) if a.declared == Some(*declared)
                && a.modifiers == [visibility_modifier(*visibility), JavaModifier::Static, JavaModifier::Final]
                && a.name == identifier(name) && &a.ty == ty && a.initializer.is_some()),
        JavaConstantsInput::Reference { symbol, result } => {
            matches!(output, JavaConstantsNode::Expression(a) if &a.ty == result &&
                a.kind == JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(*symbol))))
        }
    }
}
