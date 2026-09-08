//! Long text plans own all receiver and argument chunks, not only call roots.

use super::checked;
use crate::ast::{JavaCallableRef, JavaExpr, JavaExprKind, JavaKnownType, JavaLiteral, JavaType};
use crate::capabilities::support::{JavaValueNode, plans::JavaMappingPlan};
use crate::capabilities::{JavaTextValues, text_values::JavaTextValuesInput};

fn alter_first_literal(value: &mut JavaExpr) {
    match &mut value.kind {
        JavaExprKind::Literal(JavaLiteral::String(text)) => text.push('x'),
        JavaExprKind::Call {
            receiver: Some(receiver),
            ..
        } => alter_first_literal(receiver),
        _ => panic!("expected text construction"),
    }
}

#[test]
fn text_plan_authenticates_complete_balanced_assembly() {
    let (plan, output) = checked(
        JavaTextValues,
        JavaTextValuesInput::Value("a\0😀".repeat(30_000)),
    );
    let JavaValueNode::Expression(original) = output else {
        panic!("text expression")
    };
    for mutation in 0..8 {
        let mut changed = original.clone();
        let JavaExprKind::Call {
            callable,
            receiver,
            arguments,
        } = &mut changed.kind
        else {
            panic!("multi-chunk call")
        };
        match mutation {
            0 => alter_first_literal(receiver.as_mut().unwrap()),
            1 => alter_first_literal(&mut arguments[0]),
            2 => *receiver = None,
            3 => arguments.clear(),
            4 => arguments.push(arguments[0].clone()),
            5 => {
                let JavaCallableRef::Member { signature, .. } = callable else {
                    panic!("known member")
                };
                signature.nullable_result = true;
            }
            6 => {
                let JavaCallableRef::Member { owner, .. } = callable else {
                    panic!("known member")
                };
                *owner = JavaType::known(JavaKnownType::Object);
            }
            7 => changed.ty = JavaType::known(JavaKnownType::Object),
            _ => unreachable!(),
        }
        assert!(
            !plan.verify_output(&JavaValueNode::Expression(changed)),
            "mutation {mutation}"
        );
    }
}
