//! Independent mapping-owned output checks.
use super::*;
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
java_input_plan!(JavaConditionalsInput, JavaConditionalsNode);
fn representation(input: &JavaConditionalsInput) -> R {
    match input {
        JavaConditionalsInput::Value(_) => R::StructuredControl,
    }
}
fn verify(input: &JavaConditionalsInput, output: &JavaConditionalsNode) -> bool {
    match input {
        JavaConditionalsInput::Value(i) => {
            let JavaConditionalsNode::Value { statements, value } = output;
            let Some(owned) = statements.strip_prefix(i.prefix.as_slice()) else {
                return false;
            };
            matches!(owned, [
                JavaStmt::Local { finality: JavaLocalFinality::Mutable, ty, name, value: None },
                JavaStmt::If { condition, then_block, else_block: Some(else_block) },
            ] if ty == &i.result_type && name == &i.result_name && condition == &i.condition
                && then_block == &i.then_block && else_block == &i.else_block)
                && value.as_ref() == &JavaExpr::local(i.result_type.clone(), i.result_name.clone())
        }
    }
}
