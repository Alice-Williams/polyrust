//! Exact cast-pair admission and original-child traversal, independent of emission.
use super::*;
use crate::ast::numeric_fixture::Fixture;

#[test]
fn signed_widening_profile_admits_only_the_exact_added_conversion_pair() {
    for from in [S::I32, S::I64, S::U32, S::U64, S::Int, S::Bool, S::F64] {
        let fixture = Fixture::new(&[from]);
        for to in [S::I32, S::I64, S::U32, S::U64, S::Int, S::Bool, S::F64] {
            let original = fixture.input(0);
            let conversion = fixture
                .values()
                .numeric_conversion(to, original.clone())
                .unwrap();
            let mut children = Vec::new();
            let accepted = visit(&conversion, &mut |node| match node {
                Node::Value(value) => children.push(value.clone()),
                _ => panic!("conversion must visit its original value"),
            });
            let expected = matches!(
                (from, to),
                (S::I32, S::U32)
                    | (S::I64, S::U64)
                    | (S::U32, S::I32)
                    | (S::U64, S::I64)
                    | (S::I32, S::I64)
            );
            assert_eq!(accepted, expected, "{from:?} -> {to:?}");
            assert_eq!(children, if expected { vec![original] } else { vec![] });
        }
    }
}
