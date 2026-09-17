//! Conditional child traversal retains exact callable identities, not spellings.
use super::{ScalarCalls, fixture::Fixture, shape};
use crate::ast::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
enum Position {
    Condition,
    Then,
    Else,
}

#[test]
fn every_floating_conditional_child_preserves_exact_call_edges() {
    for position in [Position::Condition, Position::Then, Position::Else] {
        for defined in [false, true] {
            let f = Fixture::with_result(
                &[vec![], vec![], vec![]],
                CObjectType::scalar(CScalarType::F64),
            );
            let e = f.values();
            let zero = e
                .literal(CLiteral::F64(
                    portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
                ))
                .unwrap();
            let call = f.call(2, vec![]);
            let condition = if matches!(position, Position::Condition) {
                e.numeric_conversion(CScalarType::Bool, call.clone())
                    .unwrap()
            } else {
                e.literal(CLiteral::Bool(true)).unwrap()
            };
            let then_value = if matches!(position, Position::Then) {
                call.clone()
            } else {
                zero.clone()
            };
            let else_value = if matches!(position, Position::Else) {
                call
            } else {
                zero.clone()
            };
            let value = e.conditional(condition, then_value, else_value).unwrap();
            let body = f
                .statements(1)
                .block(f.scopes[1].clone(), f.returning(1, value.clone()))
                .unwrap();
            assert_eq!(
                shape::dependencies(&f.functions[1], &body),
                Some(BTreeSet::from([f.functions[2].clone()])),
                "{position:?}"
            );
            let mut bodies = vec![f.returning(0, f.call(1, vec![])), f.returning(1, value)];
            if defined {
                bodies.push(f.returning(2, zero));
            }
            let proof = ScalarCalls::derive(&[f.source(bodies)]);
            for function in &f.functions {
                assert_eq!(
                    proof.accepts(&e.direct(function.clone()).unwrap()),
                    defined,
                    "{position:?}, defined={defined}"
                );
            }
        }
    }
}
