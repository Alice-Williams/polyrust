//! A standard call never erases generated dependencies in its argument.
use super::{ScalarCalls, fixture::Fixture, shape};
use crate::{ast::*, dialect::CKnownCall};
use std::collections::BTreeSet;

#[test]
fn truncation_argument_retains_exact_original_generated_edge() {
    for defined in [false, true] {
        let f = Fixture::with_result(
            &[vec![], vec![], vec![]],
            CObjectType::scalar(CScalarType::F64),
        );
        let e = f.values();
        let value = e
            .call_value(e.known(CKnownCall::FloatTruncate), vec![f.call(2, vec![])])
            .unwrap();
        let body = f
            .statements(1)
            .block(f.scopes[1].clone(), f.returning(1, value.clone()))
            .unwrap();
        assert_eq!(
            shape::dependencies(&f.functions[1], &body),
            Some(BTreeSet::from([f.functions[2].clone()]))
        );
        let mut bodies = vec![f.returning(0, f.call(1, vec![])), f.returning(1, value)];
        if defined {
            let zero = e
                .literal(CLiteral::F64(
                    portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
                ))
                .unwrap();
            bodies.push(f.returning(2, zero));
        }
        let proof = ScalarCalls::derive(&[f.source(bodies)]);
        for function in &f.functions {
            assert_eq!(proof.accepts(&e.direct(function.clone()).unwrap()), defined);
        }
        assert!(proof.accepts(&e.known(CKnownCall::FloatTruncate)));
        for other in CKnownCall::ALL {
            if !matches!(
                other,
                CKnownCall::FloatTruncate | CKnownCall::FloatRemainder
            ) {
                assert!(!proof.accepts(&e.known(other)), "{other:?}");
            }
        }
    }
}

#[path = "scalar_call_remainder.rs"]
mod remainder;
