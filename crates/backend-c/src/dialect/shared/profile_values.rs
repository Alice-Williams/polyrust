//! Scalar expression admission; arithmetic safety remains a separate proof.
use super::{Node, signature};
use crate::ast::{
    CBinaryOperator, CCallableKind, CConversion, CLiteral, CObjectTypeKind, CPlaceKind,
    CScalarType, CSignedLiteral, CUnaryOperator, CUnsignedLiteral, CValue, CValueKind,
};

#[path = "profile_integer_operations.rs"]
mod integer;

pub(super) fn visit<'a>(
    value: &'a CValue,
    registry: Option<&crate::ast::CRegistry>,
    add: &mut impl FnMut(Node<'a>),
) -> Result<(), String> {
    add(Node::Type(value.ty()));
    if integer::visit(value, add) {
        return Ok(());
    }
    match value.kind() {
        CValueKind::KnownConstant(crate::ast::CKnownConstant::DoubleInfinity) => {}
        CValueKind::Call(call) => {
            match call.callable().kind() {
                CCallableKind::Direct(function) => signature(function, registry)?,
                CCallableKind::Known(
                    crate::dialect::CKnownCall::FloatTruncate
                    | crate::dialect::CKnownCall::FloatRemainder,
                ) => {}
                _ => {
                    return Err("C shared call profile requires an admitted target".into());
                }
            }
            for argument in call.arguments() {
                add(Node::Value(argument));
            }
        }
        CValueKind::Literal(
            CLiteral::F64(_)
            | CLiteral::Bool(_)
            | CLiteral::Unsigned(CUnsignedLiteral::U32(_) | CUnsignedLiteral::U64(_))
            | CLiteral::Signed(
                CSignedLiteral::I32(_) | CSignedLiteral::I64(_) | CSignedLiteral::Int(_),
            ),
        ) => {}
        CValueKind::Read(place) => add(Node::Place(place)),
        CValueKind::AddressOf(place) if !matches!(place.kind(), CPlaceKind::Global(_)) => {
            add(Node::Place(place))
        }
        CValueKind::Unary {
            operator: CUnaryOperator::LogicalNot,
            operand,
        } if matches!(
            operand.ty().kind(),
            CObjectTypeKind::Scalar(CScalarType::Bool)
        ) =>
        {
            add(Node::Value(operand));
        }
        CValueKind::Unary {
            operator: CUnaryOperator::Negate,
            operand,
        } if matches!(
            operand.ty().kind(),
            CObjectTypeKind::Scalar(CScalarType::F64)
        ) && value.ty() == operand.ty() =>
        {
            add(Node::Value(operand));
        }
        CValueKind::Unary {
            operator: CUnaryOperator::BitNot | CUnaryOperator::Negate,
            operand,
        } if matches!(
            operand.ty().kind(),
            CObjectTypeKind::Scalar(CScalarType::I32 | CScalarType::I64)
        ) && matches!(
            (operand.ty().kind(), value.ty().kind()),
            (
                CObjectTypeKind::Scalar(CScalarType::I32),
                CObjectTypeKind::Scalar(CScalarType::Int)
            ) | (
                CObjectTypeKind::Scalar(CScalarType::I64),
                CObjectTypeKind::Scalar(CScalarType::I64)
            )
        ) =>
        {
            add(Node::Value(operand));
        }
        CValueKind::Conditional {
            condition,
            then_value,
            else_value,
        } if condition.ty().kind() == &CObjectTypeKind::Scalar(CScalarType::Bool)
            && matches!(
                value.ty().kind(),
                CObjectTypeKind::Scalar(CScalarType::F64 | CScalarType::U32)
            )
            && then_value.ty() == value.ty()
            && else_value.ty() == value.ty() =>
        {
            add(Node::Value(condition));
            add(Node::Value(then_value));
            add(Node::Value(else_value));
        }
        CValueKind::Conditional {
            condition,
            then_value,
            else_value,
        } if matches!(
            condition.ty().kind(),
            CObjectTypeKind::Scalar(CScalarType::Bool)
        ) && [then_value.as_ref(), else_value.as_ref(), value]
            .iter()
            .all(|child| {
                matches!(
                    child.ty().kind(),
                    CObjectTypeKind::Scalar(CScalarType::I32 | CScalarType::Int | CScalarType::I64)
                )
            })
            && matches!(
                (then_value.ty().kind(), else_value.ty().kind()),
                (
                    CObjectTypeKind::Scalar(CScalarType::I64),
                    CObjectTypeKind::Scalar(CScalarType::I64)
                ) | (
                    CObjectTypeKind::Scalar(CScalarType::I32 | CScalarType::Int),
                    CObjectTypeKind::Scalar(CScalarType::I32 | CScalarType::Int)
                )
            ) =>
        {
            add(Node::Value(condition));
            add(Node::Value(then_value));
            add(Node::Value(else_value));
        }
        CValueKind::Binary {
            operator: CBinaryOperator::BitAnd | CBinaryOperator::BitOr | CBinaryOperator::BitXor,
            left,
            right,
        } if matches!(
            left.ty().kind(),
            CObjectTypeKind::Scalar(CScalarType::Bool | CScalarType::I32 | CScalarType::I64)
        ) && left.ty().kind() == right.ty().kind()
            && matches!(
                (left.ty().kind(), value.ty().kind()),
                (
                    CObjectTypeKind::Scalar(CScalarType::Bool | CScalarType::I32),
                    CObjectTypeKind::Scalar(CScalarType::Int)
                ) | (
                    CObjectTypeKind::Scalar(CScalarType::I64),
                    CObjectTypeKind::Scalar(CScalarType::I64)
                )
            ) =>
        {
            add(Node::Value(left));
            add(Node::Value(right));
        }
        CValueKind::Binary {
            operator:
                CBinaryOperator::Add
                | CBinaryOperator::Subtract
                | CBinaryOperator::Multiply
                | CBinaryOperator::Divide,
            left,
            right,
        } if left.ty().kind() == &CObjectTypeKind::Scalar(CScalarType::F64)
            && right.ty() == left.ty()
            && value.ty() == left.ty() =>
        {
            add(Node::Value(left));
            add(Node::Value(right));
        }
        CValueKind::Binary {
            operator,
            left,
            right,
        } => {
            if !matches!(
                operator,
                CBinaryOperator::Equal
                    | CBinaryOperator::NotEqual
                    | CBinaryOperator::Less
                    | CBinaryOperator::LessEqual
                    | CBinaryOperator::Greater
                    | CBinaryOperator::GreaterEqual
            ) || left.ty().kind() != right.ty().kind()
                || !matches!(
                    left.ty().kind(),
                    CObjectTypeKind::Scalar(
                        CScalarType::Bool
                            | CScalarType::Int
                            | CScalarType::I32
                            | CScalarType::I64
                            | CScalarType::U32
                            | CScalarType::U64
                            | CScalarType::F64
                    )
                )
            {
                return Err("only scalar comparisons are admitted by the first C profile".into());
            }
            add(Node::Value(left));
            add(Node::Value(right));
        }
        CValueKind::Convert {
            conversion:
                CConversion::Numeric(CScalarType::Bool | CScalarType::Int | CScalarType::I32),
            operand,
        } if matches!(
            operand.ty().kind(),
            CObjectTypeKind::Scalar(CScalarType::Bool | CScalarType::Int | CScalarType::I32)
        ) =>
        {
            add(Node::Value(operand))
        }
        CValueKind::Convert {
            conversion: CConversion::AddConst(ty),
            operand,
        } => {
            add(Node::Type(ty));
            add(Node::Value(operand));
        }
        _ => return Err("C expression is outside the first shared profile".into()),
    }
    Ok(())
}
