//! Independent mapping-owned output checks.
use super::*;
use crate::ast::{JavaExprKind, JavaValueRef};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
java_input_plan!(JavaFunctionsInput, JavaFunctionsNode);

fn representation(input: &JavaFunctionsInput) -> R {
    match input {
        JavaFunctionsInput::Declaration(_) => R::Declaration,
        JavaFunctionsInput::ParameterRead { .. }
        | JavaFunctionsInput::Return { .. }
        | JavaFunctionsInput::Call { .. } => R::Direct,
    }
}
fn verify(input: &JavaFunctionsInput, output: &JavaFunctionsNode) -> bool {
    match input {
        JavaFunctionsInput::Declaration(i) => matches!(output, JavaFunctionsNode::Declaration(a) if
            a.declared == i.declared && a.annotations.is_empty()
            && a.modifiers == [visibility_modifier(i.visibility), JavaModifier::Static]
            && a.type_parameters.is_empty() && a.return_type == i.return_type
            && a.name == identifier(&i.name) && a.parameters == i.parameters && a.body.as_ref() == Some(&i.body)),
        JavaFunctionsInput::ParameterRead { ty, name } => {
            matches!(output, JavaFunctionsNode::Expression(a) if
            &a.ty == ty && a.kind == JavaExprKind::Value(JavaValueRef::Local(identifier(name))))
        }
        JavaFunctionsInput::Return { .. } => {
            matches!(output, JavaFunctionsNode::Statement(a) if matches!(a.as_ref(), JavaStmt::Return(Some(_))))
        }
        JavaFunctionsInput::Call {
            result,
            callable,
            arguments,
        } => matches!(output, JavaFunctionsNode::Expression(a) if
            &a.ty == result && matches!(&a.kind, JavaExprKind::Call { callable: c, receiver: None, arguments: args }
                if c == callable.as_ref() && args.len() == arguments.len())),
    }
}
