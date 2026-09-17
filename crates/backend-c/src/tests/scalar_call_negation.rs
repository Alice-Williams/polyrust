//! Every newly admitted child position must retain transitive call authority.
use super::{ScalarCalls, fixture::Fixture};
use crate::ast::*;

#[derive(Clone, Copy, Debug)]
enum Position {
    Negate,
    Condition,
    Then,
    Else,
}

fn nested(f: &Fixture, position: Position) -> CValue {
    let call = f.call(2, vec![]);
    match position {
        Position::Negate => f.values().unary(CUnaryOperator::Negate, call).unwrap(),
        Position::Condition | Position::Then | Position::Else => {
            let condition = if matches!(position, Position::Condition) {
                f.values()
                    .numeric_conversion(CScalarType::Bool, call.clone())
                    .unwrap()
            } else {
                f.values().literal(CLiteral::Bool(true)).unwrap()
            };
            let then_value = if matches!(position, Position::Then) {
                call.clone()
            } else {
                f.literal(1)
            };
            let else_value = if matches!(position, Position::Else) {
                call
            } else {
                f.literal(1)
            };
            f.values()
                .conditional(condition, then_value, else_value)
                .unwrap()
        }
    }
}

#[test]
fn wrapping_negation_and_each_conditional_child_retain_transitive_call_authority() {
    for position in [
        Position::Negate,
        Position::Condition,
        Position::Then,
        Position::Else,
    ] {
        for defined in [false, true] {
            let f = Fixture::new(&[0, 0, 0]);
            let value = nested(&f, position);
            let mut bodies = vec![f.returning(0, f.call(1, vec![])), f.returning(1, value)];
            if defined {
                bodies.push(f.returning(2, f.literal(1)));
            }
            let source = f.source(bodies);
            let proof = ScalarCalls::derive(&[source]);
            for function in &f.functions {
                assert_eq!(
                    proof.accepts(&f.values().direct(function.clone()).unwrap()),
                    defined,
                    "{position:?}, defined={defined}"
                );
            }
        }
    }
}
