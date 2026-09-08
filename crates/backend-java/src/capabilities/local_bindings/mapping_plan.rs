//! Independent mapping-owned output checks.
use super::{JavaLocalBindingsInput, JavaLocalBindingsNode};
use crate::ast::{JavaExprKind, JavaLocalFinality, JavaStmt, JavaValueRef};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
use crate::lower::identifier;
java_input_plan!(JavaLocalBindingsInput, JavaLocalBindingsNode);
fn representation(input: &JavaLocalBindingsInput) -> R {
    match input {
        JavaLocalBindingsInput::Bind { .. } | JavaLocalBindingsInput::Read { .. } => R::Direct,
    }
}
fn verify(input: &JavaLocalBindingsInput, output: &JavaLocalBindingsNode) -> bool {
    match input {
        JavaLocalBindingsInput::Bind { name, ty, .. } => {
            matches!(output, JavaLocalBindingsNode::Statement(a)
            if matches!(a.as_ref(), JavaStmt::Local { finality: JavaLocalFinality::Final, name: n, ty: t, value: Some(_) }
                if n == &identifier(name) && t == ty))
        }
        JavaLocalBindingsInput::Read { name, ty } => {
            matches!(output, JavaLocalBindingsNode::Expression(a)
            if &a.ty == ty && a.kind == JavaExprKind::Value(JavaValueRef::Local(identifier(name))))
        }
    }
}
