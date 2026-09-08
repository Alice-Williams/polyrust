//! Independent mapping-owned output checks.
use super::{JavaLoopsInput, JavaLoopsNode};
use crate::ast::{JavaExprKind, JavaStmt, JavaValueRef};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
use crate::lower::identifier;
java_input_plan!(JavaLoopsInput, JavaLoopsNode);
fn representation(input: &JavaLoopsInput) -> R {
    match input {
        JavaLoopsInput::ForEach { .. } => R::StructuredControl,
        JavaLoopsInput::BindingRead { .. } => R::Direct,
    }
}
fn verify(input: &JavaLoopsInput, output: &JavaLoopsNode) -> bool {
    match input {
        JavaLoopsInput::ForEach {
            binding_type,
            binding,
            body,
            ..
        } => matches!(output, JavaLoopsNode::Statement(a)
            if matches!(a.as_ref(), JavaStmt::ForEach { binding_type: t, binding: n, body: b, .. }
                if t == binding_type && n == &identifier(binding) && b == body)),
        JavaLoopsInput::BindingRead {
            binding_type,
            binding,
        } => matches!(output, JavaLoopsNode::Expression(a)
            if &a.ty == binding_type && a.kind == JavaExprKind::Value(JavaValueRef::Local(identifier(binding)))),
    }
}
