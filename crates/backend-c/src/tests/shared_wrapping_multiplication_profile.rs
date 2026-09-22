//! Exact profile categories and actual child scheduling, not textual operators.
use super::*;
use crate::ast::{CExpressions, CLiteral, CRegistry, CSignedLiteral, CUnsignedLiteral};

#[test]
fn wrapping_multiplication_profile_keeps_widths_exact_and_visits_both_children() {
    let registry = CRegistry::new();
    let e = CExpressions::new(&registry);
    let u32 = || {
        e.literal(CLiteral::Unsigned(CUnsignedLiteral::U32(1)))
            .unwrap()
    };
    let u64 = || {
        e.literal(CLiteral::Unsigned(CUnsignedLiteral::U64(1)))
            .unwrap()
    };
    let i32 = || e.literal(CLiteral::Signed(CSignedLiteral::I32(1))).unwrap();
    for operand in [u32(), u64()] {
        let value = e.binary(B::Multiply, operand.clone(), operand).unwrap();
        let mut children = Vec::new();
        assert!(visit(&value, &mut |node| children.push(node)));
        let CValueKind::Binary { left, right, .. } = value.kind() else {
            panic!("binary")
        };
        assert!(
            matches!(children.as_slice(), [Node::Value(a), Node::Value(b)]
            if std::ptr::eq(*a, left.as_ref()) && std::ptr::eq(*b, right.as_ref()))
        );
    }
    for (operator, left, right) in [
        (B::Multiply, u32(), u64()),
        (B::Multiply, u64(), u32()),
        (B::Multiply, i32(), u32()),
        (B::Multiply, u32(), i32()),
        (B::Divide, u32(), u32()),
        (B::Divide, u64(), u64()),
    ] {
        let value = e.binary(operator, left, right).unwrap();
        let mut visited = false;
        assert!(!visit(&value, &mut |_| visited = true));
        assert!(!visited);
    }
}
