//! Neither fmod argument may hide an undefined generated dependency.
use super::*;

#[test]
fn remainder_tracks_both_generated_argument_edges() {
    for defined in [0, 1, 2] {
        let f = Fixture::with_result(
            &[vec![], vec![], vec![]],
            CObjectType::scalar(CScalarType::F64),
        );
        let e = f.values();
        let value = e
            .call_value(
                e.known(CKnownCall::FloatRemainder),
                vec![f.call(1, vec![]), f.call(2, vec![])],
            )
            .unwrap();
        let body = f
            .statements(0)
            .block(f.scopes[0].clone(), f.returning(0, value.clone()))
            .unwrap();
        assert_eq!(
            shape::dependencies(&f.functions[0], &body),
            Some(BTreeSet::from([
                f.functions[1].clone(),
                f.functions[2].clone()
            ]))
        );
        let mut bodies = vec![f.returning(0, value)];
        for index in 1..=defined {
            let zero = e
                .literal(CLiteral::F64(
                    portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
                ))
                .unwrap();
            bodies.push(f.returning(index, zero));
        }
        let proof = ScalarCalls::derive(&[f.source(bodies)]);
        assert_eq!(
            proof.accepts(&e.direct(f.functions[0].clone()).unwrap()),
            defined == 2
        );
        assert!(proof.accepts(&e.known(CKnownCall::FloatRemainder)));
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
