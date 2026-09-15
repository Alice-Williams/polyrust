//! Read-only observations of production typed nodes before certification.
use super::{LazyBooleanInput, LazyBooleanOperator};
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
#[path = "short_circuit_hir_ast.rs"]
mod source;

pub(super) fn check(
    reader: &Reader<'_>,
    input: &LazyBooleanInput<'_>,
    start: usize,
    result: &CValue,
) {
    let (last, prefix) = reader.prelude[start..].split_last().unwrap();
    let CStatementKind::If {
        condition,
        then_block,
        else_block,
    } = last.kind()
    else {
        panic!("lazy branch")
    };
    let CValueKind::Read(place) = result.kind() else {
        panic!("lazy result place")
    };
    let CPlaceKind::Local(local) = place.kind() else {
        panic!("local lazy result")
    };
    assert_eq!(reader.active_scope.as_ref(), Some(local.scope()));
    assert_eq!(then_block.scope().parent(), Some(local.scope()));
    assert_eq!(else_block.scope().parent(), Some(local.scope()));
    assert_ne!(then_block.scope(), else_block.scope());
    assert!(else_block.statements().is_empty());
    let CStatementKind::Assign { place: target, .. } =
        then_block.statements().last().unwrap().kind()
    else {
        panic!("lazy assignment")
    };
    assert_eq!(target, place.as_ref());
    match input.operator() {
        LazyBooleanOperator::And => assert_eq!(condition, result),
        LazyBooleanOperator::Or => {
            let CValueKind::Convert {
                conversion: CConversion::Numeric(CScalarType::Bool),
                operand,
            } = condition.kind()
            else {
                panic!("bool result normalization")
            };
            let CValueKind::Unary {
                operator: CUnaryOperator::LogicalNot,
                operand,
            } = operand.kind()
            else {
                panic!("Or guard")
            };
            assert_eq!(operand.as_ref(), result);
        }
    }
    assert_eq!(
        calls(prefix),
        source::calls(input.left()),
        "left prelude call count"
    );
    assert_eq!(
        calls(then_block.statements()),
        source::calls(input.right()),
        "branch-local right calls"
    );
    eprintln!("LAZY_AST\tc");
}

enum Node<'a> {
    Statement(&'a CStatement),
    Value(&'a CValue),
    Place(&'a CPlace),
    Initializer(&'a CInitializer),
}
fn calls(statements: &[CStatement]) -> usize {
    let mut pending: Vec<_> = statements.iter().map(Node::Statement).collect();
    let mut calls = 0;
    while let Some(node) = pending.pop() {
        match node {
            Node::Statement(statement) => match statement.kind() {
                CStatementKind::Block(block) => {
                    pending.extend(block.statements().iter().map(Node::Statement))
                }
                CStatementKind::Declare(local) => {
                    pending.push(Node::Initializer(local.initializer().unwrap()))
                }
                CStatementKind::Assign { place, value } => {
                    pending.extend([Node::Place(place), Node::Value(value)])
                }
                CStatementKind::Discard(value) | CStatementKind::Return(Some(value)) => {
                    pending.push(Node::Value(value))
                }
                CStatementKind::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    pending.push(Node::Value(condition));
                    pending.extend(
                        then_block
                            .statements()
                            .iter()
                            .chain(else_block.statements())
                            .map(Node::Statement),
                    );
                }
                _ => panic!("unadmitted statement in lazy probe"),
            },
            Node::Value(value) => match value.kind() {
                CValueKind::Literal(_) => {}
                CValueKind::Read(place) | CValueKind::AddressOf(place) => {
                    pending.push(Node::Place(place))
                }
                CValueKind::Unary { operand, .. } | CValueKind::Convert { operand, .. } => {
                    pending.push(Node::Value(operand))
                }
                CValueKind::Binary { left, right, .. } => {
                    pending.extend([Node::Value(left), Node::Value(right)])
                }
                CValueKind::Call(call) => {
                    calls += 1;
                    pending.extend(call.arguments().iter().map(Node::Value));
                }
                _ => panic!("unadmitted value in lazy probe"),
            },
            Node::Place(place) => match place.kind() {
                CPlaceKind::Local(_) | CPlaceKind::Parameter(_) => {}
                CPlaceKind::Member { base, .. } => pending.push(Node::Place(base)),
                CPlaceKind::Dereference(value) => pending.push(Node::Value(value)),
                _ => panic!("unadmitted place in lazy probe"),
            },
            Node::Initializer(value) => match value.kind() {
                CInitializerKind::Expression(value) => pending.push(Node::Value(value)),
                CInitializerKind::Struct { members, .. } => {
                    pending.extend(members.iter().map(|(_, value)| Node::Initializer(value)))
                }
                _ => panic!("unadmitted initializer in lazy probe"),
            },
        }
    }
    calls
}
