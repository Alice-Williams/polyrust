//! Observe actual mapper nodes; no mutation or renderer reconstruction.
use super::{BitwiseInput, BitwiseOperands, BitwiseOperator};
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
use rustc_middle::ty;

pub(super) fn check(reader: &Reader<'_>, input: &BitwiseInput<'_>, value: &CValue) {
    let expression = match input.operands() {
        BitwiseOperands::Complement(operand) | BitwiseOperands::Binary(_, operand, _) => operand,
    };
    let wide = matches!(
        reader.checked.expr_ty(expression).kind(),
        ty::Int(ty::IntTy::I64)
    );
    let scalar = if wide {
        CScalarType::I64
    } else {
        CScalarType::I32
    };
    assert_eq!(value.ty().kind(), &CObjectTypeKind::Scalar(scalar));
    let operation = if wide {
        value
    } else {
        let CValueKind::Convert {
            conversion: CConversion::Numeric(CScalarType::I32),
            operand,
        } = value.kind()
        else {
            panic!("I32 normalization")
        };
        assert_eq!(
            operand.ty().kind(),
            &CObjectTypeKind::Scalar(CScalarType::Int)
        );
        operand
    };
    match input.operands() {
        BitwiseOperands::Complement(_) => {
            let CValueKind::Unary {
                operator: CUnaryOperator::BitNot,
                operand,
            } = operation.kind()
            else {
                panic!("typed complement")
            };
            assert_eq!(operand.ty().kind(), &CObjectTypeKind::Scalar(scalar));
        }
        BitwiseOperands::Binary(source, _, _) => {
            let CValueKind::Binary {
                operator,
                left,
                right,
            } = operation.kind()
            else {
                panic!("typed binary")
            };
            let expected = match source {
                BitwiseOperator::And => CBinaryOperator::BitAnd,
                BitwiseOperator::Or => CBinaryOperator::BitOr,
                BitwiseOperator::Xor => CBinaryOperator::BitXor,
            };
            assert_eq!(*operator, expected);
            assert_eq!(left.ty().kind(), &CObjectTypeKind::Scalar(scalar));
            assert_eq!(right.ty().kind(), &CObjectTypeKind::Scalar(scalar));
        }
    }
    eprintln!("BITWISE_AST\tc");
}
