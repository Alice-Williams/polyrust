//! Closed local-storage syntax used only to derive scalar call effects.
use crate::ast::{
    CBlock, CCall, CCallableKind, CConversion, CFunctionRef, CInitializer, CInitializerKind,
    CLiteral, CObjectType, CObjectTypeKind, CPlace, CPlaceKind, CReturnType, CScalarType,
    CSignedLiteral, CStatement, CStatementKind, CUnaryOperator, CUnsignedLiteral, CValue,
    CValueKind,
};
use std::collections::BTreeSet;

enum Node<'a> {
    Block(&'a CBlock),
    Statement(&'a CStatement),
    Initializer(&'a CInitializer),
    Value(&'a CValue),
    Place(&'a CPlace),
    Call(&'a CCall),
}

fn scalar(ty: &CObjectType) -> bool {
    matches!(
        ty.kind(),
        CObjectTypeKind::Scalar(
            CScalarType::I32
                | CScalarType::I64
                | CScalarType::Int
                | CScalarType::Bool
                | CScalarType::F64
        )
    )
}

fn signature(function: &CFunctionRef) -> bool {
    (match function.signature().return_type() {
        CReturnType::Void => true,
        CReturnType::Value(value) => scalar(value.declared_type()),
    }) && function
        .signature()
        .parameters()
        .iter()
        .all(|p| scalar(p.declared_type()))
}

pub(super) fn dependencies(
    function: &CFunctionRef,
    body: &CBlock,
) -> Option<BTreeSet<CFunctionRef>> {
    if !signature(function) {
        return None;
    }
    let mut pending = vec![Node::Block(body)];
    let mut edges = BTreeSet::new();
    let mut nodes = 0usize;
    while let Some(node) = pending.pop() {
        nodes = nodes.checked_add(1)?;
        if nodes > 100_000 {
            return None;
        }
        match node {
            Node::Block(block) => pending.extend(block.statements().iter().map(Node::Statement)),
            Node::Statement(statement) => match statement.kind() {
                CStatementKind::Block(block) => pending.push(Node::Block(block)),
                CStatementKind::Declare(local) => {
                    pending.push(Node::Initializer(local.initializer()?))
                }
                CStatementKind::Assign { place, value }
                    if matches!(place.kind(), CPlaceKind::Local(_))
                        && matches!(
                            place.ty().kind(),
                            CObjectTypeKind::Scalar(CScalarType::Bool)
                        ) =>
                {
                    pending.extend([Node::Place(place), Node::Value(value)]);
                }
                CStatementKind::Discard(value) | CStatementKind::Return(Some(value)) => {
                    pending.push(Node::Value(value))
                }
                CStatementKind::Return(None) => {}
                CStatementKind::Evaluate(effect) => pending.push(Node::Call(effect.call())),
                CStatementKind::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    pending.extend([
                        Node::Value(condition),
                        Node::Block(then_block),
                        Node::Block(else_block),
                    ]);
                }
                _ => return None,
            },
            Node::Initializer(initializer) => match initializer.kind() {
                CInitializerKind::Expression(value) => pending.push(Node::Value(value)),
                CInitializerKind::Struct { members, .. } => {
                    pending.extend(members.iter().map(|(_, value)| Node::Initializer(value)))
                }
                _ => return None,
            },
            Node::Value(value) => match value.kind() {
                // A typed floating macro is a scalar value, not a storage read.
                // Do not admit pointer-valued standard streams through this rule.
                CValueKind::KnownConstant(crate::ast::CKnownConstant::DoubleInfinity) => {}
                CValueKind::Literal(
                    CLiteral::F64(_)
                    | CLiteral::Bool(_)
                    | CLiteral::Unsigned(CUnsignedLiteral::U32(_) | CUnsignedLiteral::U64(_))
                    | CLiteral::Signed(
                        CSignedLiteral::I32(_) | CSignedLiteral::I64(_) | CSignedLiteral::Int(_),
                    ),
                ) => {}
                CValueKind::Read(place)
                    if matches!(place.kind(), CPlaceKind::Global(object)
                    if object.ty().constness() == crate::ast::CConstness::Const
                        && scalar(object.ty())) => {}
                CValueKind::Read(place) | CValueKind::AddressOf(place) => {
                    pending.push(Node::Place(place))
                }
                CValueKind::Binary { left, right, .. } => {
                    pending.extend([Node::Value(left), Node::Value(right)])
                }
                CValueKind::Unary {
                    operator: CUnaryOperator::BitNot,
                    operand,
                } if matches!(
                    operand.ty().kind(),
                    CObjectTypeKind::Scalar(CScalarType::U32 | CScalarType::U64)
                ) =>
                {
                    pending.push(Node::Value(operand));
                }
                CValueKind::Unary {
                    operator: CUnaryOperator::LogicalNot,
                    operand,
                } if matches!(
                    operand.ty().kind(),
                    CObjectTypeKind::Scalar(CScalarType::Bool)
                ) =>
                {
                    pending.push(Node::Value(operand));
                }
                CValueKind::Unary {
                    operator: CUnaryOperator::Negate,
                    operand,
                } if matches!(
                    operand.ty().kind(),
                    CObjectTypeKind::Scalar(CScalarType::F64)
                ) && value.ty() == operand.ty() =>
                {
                    pending.push(Node::Value(operand));
                }
                CValueKind::Unary {
                    operator: CUnaryOperator::BitNot | CUnaryOperator::Negate,
                    operand,
                } if matches!(
                    operand.ty().kind(),
                    CObjectTypeKind::Scalar(CScalarType::I32 | CScalarType::I64)
                ) =>
                {
                    pending.push(Node::Value(operand));
                }
                CValueKind::Conditional {
                    condition,
                    then_value,
                    else_value,
                } => {
                    pending.extend([
                        Node::Value(condition),
                        Node::Value(then_value),
                        Node::Value(else_value),
                    ]);
                }
                CValueKind::Convert {
                    conversion: CConversion::Numeric(_) | CConversion::AddConst(_),
                    operand,
                } => pending.push(Node::Value(operand)),
                CValueKind::Call(call) => pending.push(Node::Call(call)),
                _ => return None,
            },
            Node::Call(call) => {
                match call.callable().kind() {
                    CCallableKind::Direct(callee) if signature(callee) => {
                        edges.insert(callee.as_ref().clone());
                    }
                    // No generated-storage effect; arguments still retain every edge.
                    CCallableKind::Known(
                        crate::dialect::CKnownCall::FloatTruncate
                        | crate::dialect::CKnownCall::FloatRemainder,
                    ) => {}
                    _ => return None,
                }
                pending.extend(call.arguments().iter().map(Node::Value));
            }
            Node::Place(place) => match place.kind() {
                CPlaceKind::Local(_) | CPlaceKind::Parameter(_) => {}
                CPlaceKind::Member { base, .. } => pending.push(Node::Place(base)),
                CPlaceKind::Dereference(value) => pending.push(Node::Value(value)),
                _ => return None,
            },
        }
    }
    Some(edges)
}
