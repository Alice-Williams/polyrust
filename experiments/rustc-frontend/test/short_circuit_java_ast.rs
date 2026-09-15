//! Source call inventory versus actual typed Java branch placement.
use super::{LazyBooleanInput, LazyBooleanOperator};
use crate::java_lower::Reader;
use portable_backend_java::ast::*;
#[path = "short_circuit_hir_ast.rs"]
mod source;

pub(super) fn check(
    reader: &Reader<'_>,
    input: &LazyBooleanInput<'_>,
    start: usize,
    result: &JavaExpr,
) {
    let (last, prefix) = reader.prelude[start..].split_last().unwrap();
    let JavaStmt::If {
        condition,
        then_block,
        else_block: Some(else_block),
    } = last
    else {
        panic!("lazy branch")
    };
    let JavaExprKind::Value(JavaValueRef::Local(result_name)) = &result.kind else {
        panic!("lazy result place")
    };
    let JavaStmt::Local {
        finality: JavaLocalFinality::Mutable,
        name,
        value: Some(_),
        ..
    } = prefix.last().unwrap()
    else {
        panic!("initialized mutable result")
    };
    assert_eq!(name, result_name);
    assert_eq!(reader.scopes[&input.right().hir_id], reader.active_scope);
    assert!(else_block.statements.is_empty());
    let JavaStmt::Assign { target, .. } = then_block.statements.last().unwrap() else {
        panic!("lazy assignment")
    };
    assert_eq!(target, result);
    match input.operator() {
        LazyBooleanOperator::And => assert_eq!(condition, result),
        LazyBooleanOperator::Or => {
            let JavaExprKind::Unary {
                operator: JavaUnaryOperator::Not,
                operand,
            } = &condition.kind
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
        calls(&then_block.statements),
        source::calls(input.right()),
        "branch-local right calls"
    );
    eprintln!("LAZY_AST\tjava");
}

enum Node<'a> {
    Statement(&'a JavaStmt),
    Expression(&'a JavaExpr),
}
fn calls(statements: &[JavaStmt]) -> usize {
    let mut pending: Vec<_> = statements.iter().map(Node::Statement).collect();
    let mut calls = 0;
    while let Some(node) = pending.pop() {
        match node {
            Node::Statement(statement) => match statement {
                JavaStmt::Local {
                    value: Some(value), ..
                }
                | JavaStmt::Return(Some(value)) => pending.push(Node::Expression(value)),
                JavaStmt::Assign { target, value } => {
                    pending.extend([Node::Expression(target), Node::Expression(value)])
                }
                JavaStmt::If {
                    condition,
                    then_block,
                    else_block: Some(else_block),
                } => {
                    pending.push(Node::Expression(condition));
                    pending.extend(
                        then_block
                            .statements
                            .iter()
                            .chain(&else_block.statements)
                            .map(Node::Statement),
                    );
                }
                _ => panic!("unadmitted statement in lazy probe"),
            },
            Node::Expression(value) => match &value.kind {
                JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {}
                JavaExprKind::Unary { operand, .. } => pending.push(Node::Expression(operand)),
                JavaExprKind::Field { receiver, .. } => pending.push(Node::Expression(receiver)),
                JavaExprKind::Binary { left, right, .. } => {
                    pending.extend([Node::Expression(left), Node::Expression(right)])
                }
                JavaExprKind::Conditional {
                    condition,
                    when_true,
                    when_false,
                } => pending.extend([
                    Node::Expression(condition),
                    Node::Expression(when_true),
                    Node::Expression(when_false),
                ]),
                JavaExprKind::Call {
                    receiver: None,
                    arguments,
                    ..
                } => {
                    calls += 1;
                    pending.extend(arguments.iter().map(Node::Expression));
                }
                JavaExprKind::New { arguments, .. } => {
                    pending.extend(arguments.iter().map(Node::Expression))
                }
                _ => panic!("unadmitted expression in lazy probe"),
            },
        }
    }
    calls
}
