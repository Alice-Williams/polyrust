//! Postorder value/place reconstruction through the existing typed constructors.
use super::super::{
    CCall, CCallableKind, CIndexBase, CPlace, CPlaceKind as P, CPointerTest, CValue,
    CValueKind as V,
};
use super::{CContextError as E, Recheck};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
pub(super) enum Node<'a> {
    Value(&'a CValue),
    Place(&'a CPlace),
}
pub(super) enum Built {
    Value(Box<CValue>),
    Place(Box<CPlace>),
}
pub(super) type Children = VecDeque<Built>;

pub(super) fn value(children: &mut Children) -> Result<CValue, E> {
    match children.pop_front() {
        Some(Built::Value(value)) => Ok(*value),
        _ => Err(E::StoredStructureMismatch),
    }
}
pub(super) fn place(children: &mut Children) -> Result<CPlace, E> {
    match children.pop_front() {
        Some(Built::Place(place)) => Ok(*place),
        _ => Err(E::StoredStructureMismatch),
    }
}

pub(super) fn call_children(call: &CCall) -> Vec<Node<'_>> {
    let mut children = Vec::new();
    match call.callable().kind() {
        CCallableKind::Indirect { pointer, .. } => children.push(Node::Value(pointer)),
        CCallableKind::Direct(_) | CCallableKind::Known(_) => {}
    }
    children.extend(call.arguments().iter().map(Node::Value));
    children
}

fn children(node: Node<'_>) -> Vec<Node<'_>> {
    match node {
        Node::Value(value) => match value.kind() {
            V::Read(place) | V::AddressOf(place) => vec![Node::Place(place)],
            V::Call(call) => call_children(call),
            V::Unary { operand, .. } | V::Convert { operand, .. } => vec![Node::Value(operand)],
            V::Binary { left, right, .. } => vec![Node::Value(left), Node::Value(right)],
            V::PointerTest(test) => match test {
                CPointerTest::IsNull(value) | CPointerTest::IsNonNull(value) => {
                    vec![Node::Value(value)]
                }
                CPointerTest::SameSlot { left, right } => {
                    vec![Node::Value(left), Node::Value(right)]
                }
            },
            V::Conditional {
                condition,
                then_value,
                else_value,
            } => vec![
                Node::Value(condition),
                Node::Value(then_value),
                Node::Value(else_value),
            ],
            V::KnownConstant(_)
            | V::Literal(_)
            | V::Enumerator(_)
            | V::FunctionAddress(_)
            | V::SizeOf(_)
            | V::AlignOf(_) => vec![],
        },
        Node::Place(place) => match place.kind() {
            P::Member { base, .. } => vec![Node::Place(base)],
            P::Dereference(value) => vec![Node::Value(value)],
            P::Index { base, index } => vec![
                match base {
                    CIndexBase::Array(place) => Node::Place(place),
                    CIndexBase::Pointer(value) => Node::Value(value),
                },
                Node::Value(index),
            ],
            P::Local(_) | P::Parameter(_) | P::Global(_) => vec![],
        },
    }
}

enum Work<'a> {
    Visit(Node<'a>),
    Finish(Node<'a>, usize),
}

pub(super) fn rebuild(checker: &Recheck<'_>, root: Node<'_>) -> Result<Built, E> {
    let mut pending = vec![Work::Visit(root)];
    let mut built = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Visit(node) => {
                let children = children(node);
                pending.push(Work::Finish(node, children.len()));
                pending.extend(children.into_iter().rev().map(Work::Visit));
            }
            Work::Finish(node, count) => {
                let start = built
                    .len()
                    .checked_sub(count)
                    .ok_or(E::StoredStructureMismatch)?;
                let mut children = Children::from(built.split_off(start));
                let rebuilt = match node {
                    Node::Value(value) => {
                        Built::Value(Box::new(checker.value_node(value, &mut children)?))
                    }
                    Node::Place(place) => {
                        Built::Place(Box::new(checker.place_node(place, &mut children)?))
                    }
                };
                if !children.is_empty() {
                    return Err(E::StoredStructureMismatch);
                }
                built.push(rebuilt);
            }
        }
    }
    if built.len() != 1 {
        return Err(E::StoredStructureMismatch);
    }
    built.pop().ok_or(E::StoredStructureMismatch)
}
