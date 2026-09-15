//! Closed local-storage syntax used only to derive scalar call effects.
use crate::ast::{
    CBlock, CCallableKind, CConversion, CFunctionRef, CInitializer, CInitializerKind, CLiteral,
    CObjectType, CObjectTypeKind, CPlace, CPlaceKind, CReturnType, CScalarType, CSignedLiteral,
    CStatement, CStatementKind, CUnaryOperator, CValue, CValueKind,
};
use std::collections::BTreeSet;

enum Node<'a> {
    Block(&'a CBlock),
    Statement(&'a CStatement),
    Initializer(&'a CInitializer),
    Value(&'a CValue),
    Place(&'a CPlace),
}

fn scalar(ty: &CObjectType) -> bool {
    matches!(
        ty.kind(),
        CObjectTypeKind::Scalar(
            CScalarType::I32 | CScalarType::I64 | CScalarType::Int | CScalarType::Bool
        )
    )
}

fn signature(function: &CFunctionRef) -> bool {
    matches!(function.signature().return_type(), CReturnType::Value(value) if scalar(value.declared_type()))
        && function
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
                CValueKind::Literal(
                    CLiteral::Bool(_)
                    | CLiteral::Signed(
                        CSignedLiteral::I32(_) | CSignedLiteral::I64(_) | CSignedLiteral::Int(_),
                    ),
                ) => {}
                CValueKind::Read(place) | CValueKind::AddressOf(place) => {
                    pending.push(Node::Place(place))
                }
                CValueKind::Binary { left, right, .. } => {
                    pending.extend([Node::Value(left), Node::Value(right)])
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
                CValueKind::Convert {
                    conversion: CConversion::Numeric(_) | CConversion::AddConst(_),
                    operand,
                } => pending.push(Node::Value(operand)),
                CValueKind::Call(call) => {
                    let CCallableKind::Direct(callee) = call.callable().kind() else {
                        return None;
                    };
                    if !signature(callee) {
                        return None;
                    }
                    edges.insert(callee.as_ref().clone());
                    pending.extend(call.arguments().iter().map(Node::Value));
                }
                _ => return None,
            },
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
