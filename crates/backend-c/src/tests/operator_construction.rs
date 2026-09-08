//! Every operator constructor retains actual children, not just a result type.
use super::registry_nominals::registry;
use crate::ast::{
    CBinaryOperator as B, CExpressionError as E, CExpressions, CLiteral, CNullPointer, CObjectType,
    CPointerTarget, CScalarType as T, CSignedLiteral, CUnaryOperator as U, CValueKind,
};
#[test]
fn every_binary_variant_retains_its_operands_and_rejects_non_arithmetic() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let integer = ast
        .literal(CLiteral::Signed(CSignedLiteral::I16(3)))
        .unwrap();
    let truth = ast.literal(CLiteral::Bool(true)).unwrap();
    let falsity = ast.literal(CLiteral::Bool(false)).unwrap();
    let pointer = ast
        .literal(CLiteral::NullPointer(
            CNullPointer::new(CObjectType::pointer(CPointerTarget::Void(
                crate::ast::CConstness::Unqualified,
            )))
            .unwrap(),
        ))
        .unwrap();
    for operator in [
        B::Add,
        B::Subtract,
        B::Multiply,
        B::Divide,
        B::Remainder,
        B::ShiftLeft,
        B::ShiftRight,
        B::BitAnd,
        B::BitOr,
        B::BitXor,
        B::Equal,
        B::NotEqual,
        B::Less,
        B::LessEqual,
        B::Greater,
        B::GreaterEqual,
        B::LogicalAnd,
        B::LogicalOr,
    ] {
        let (left, right) = match operator {
            B::LogicalAnd | B::LogicalOr => (truth.clone(), falsity.clone()),
            B::Add
            | B::Subtract
            | B::Multiply
            | B::Divide
            | B::Remainder
            | B::ShiftLeft
            | B::ShiftRight
            | B::BitAnd
            | B::BitOr
            | B::BitXor
            | B::Equal
            | B::NotEqual
            | B::Less
            | B::LessEqual
            | B::Greater
            | B::GreaterEqual => (
                integer.clone(),
                ast.literal(CLiteral::Signed(CSignedLiteral::I16(1)))
                    .unwrap(),
            ),
        };
        let result = ast.binary(operator, left.clone(), right.clone()).unwrap();
        assert_eq!(result.ty(), &CObjectType::scalar(T::Int));
        assert_eq!(
            result.kind(),
            &CValueKind::Binary {
                operator,
                left: Box::new(left.clone()),
                right: Box::new(right.clone()),
            }
        );
        assert_eq!(
            ast.binary(operator, pointer.clone(), right),
            Err(E::ExpectedArithmetic)
        );
        assert_eq!(
            ast.binary(operator, left, pointer.clone()),
            Err(E::ExpectedArithmetic)
        );
    }
}
#[test]
fn every_unary_variant_retains_its_operand_and_rejects_non_arithmetic() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let pointer = ast
        .literal(CLiteral::NullPointer(
            CNullPointer::new(CObjectType::pointer(CPointerTarget::Void(
                crate::ast::CConstness::Unqualified,
            )))
            .unwrap(),
        ))
        .unwrap();
    for operator in [U::LogicalNot, U::BitNot, U::Negate] {
        let operand = match operator {
            U::LogicalNot => ast.literal(CLiteral::Bool(true)).unwrap(),
            U::BitNot | U::Negate => ast
                .literal(CLiteral::Signed(CSignedLiteral::I8(3)))
                .unwrap(),
        };
        assert_eq!(
            ast.unary(operator, operand.clone()).unwrap().kind(),
            &CValueKind::Unary {
                operator,
                operand: Box::new(operand)
            }
        );
        assert_eq!(
            ast.unary(operator, pointer.clone()),
            Err(E::ExpectedArithmetic)
        );
    }
}
