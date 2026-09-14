//! Observe actual expression visits and their typed materialization, not results alone.
use super::{Reader, TypePlan, Value};
use portable_backend_java::ast::*;
use rustc_hir as hir;
use rustc_middle::ty;

fn materialized(reader: &Reader<'_>, source: &hir::Expr<'_>, target: &JavaExpr) -> (usize, usize) {
    let JavaExprKind::Value(JavaValueRef::Local(name)) = &target.kind else {
        panic!("operand was not materialized")
    };
    let (position, statement) = reader.prelude.iter().enumerate().find(|(_, statement)| {
        matches!(statement, JavaStmt::Local { name: local, .. } if local == name)
    }).unwrap();
    let (visit, (_, after, observed)) = reader
        .expression_observations
        .iter()
        .enumerate()
        .find(|(_, (id, _, _))| *id == source.hir_id)
        .unwrap();
    let JavaStmt::Local {
        ty,
        finality,
        value: Some(value),
        ..
    } = statement
    else {
        panic!("uninitialized operand")
    };
    assert_eq!(*finality, JavaLocalFinality::Final);
    assert_eq!(*ty, target.ty);
    assert_eq!(*value, observed.clone().into_expression());
    assert_eq!(
        position, *after,
        "materialize immediately after evaluating source operand"
    );
    (visit, position)
}

pub(super) fn call(reader: &Reader<'_>, source: &hir::Expr<'_>, target: &JavaExpr) {
    let hir::ExprKind::Call(_, source_args) = source.kind else {
        panic!("source call")
    };
    let JavaExprKind::Call {
        arguments,
        receiver,
        ..
    } = &target.kind
    else {
        panic!("target call")
    };
    assert!(receiver.is_none());
    assert_eq!(source_args.len(), arguments.len());
    let positions: Vec<_> = source_args
        .iter()
        .zip(arguments)
        .map(|(source, target)| materialized(reader, source, target))
        .collect();
    assert!(
        positions
            .windows(2)
            .all(|pair| pair[0].0 < pair[1].0 && pair[0].1 < pair[1].1)
    );
    println!("CALL_ORDER\t{}", arguments.len());
}

pub(super) fn comparison(reader: &Reader<'_>, source: &hir::Expr<'_>, target: &Value) {
    let hir::ExprKind::Binary(op, left_source, right_source) = source.kind else {
        panic!("source comparison")
    };
    let target = target.clone().into_expression();
    assert_eq!(target.ty, TypePlan::Bool.java_type());
    let JavaExprKind::Binary {
        operator,
        left,
        right,
    } = &target.kind
    else {
        panic!("target comparison")
    };
    let (expected, precedence, ordering) = match op.node {
        hir::BinOpKind::Eq => (JavaBinaryOperator::Equal, JavaPrecedence::Equality, false),
        hir::BinOpKind::Ne => (
            JavaBinaryOperator::NotEqual,
            JavaPrecedence::Equality,
            false,
        ),
        hir::BinOpKind::Lt => (JavaBinaryOperator::Less, JavaPrecedence::Relational, true),
        hir::BinOpKind::Le => (
            JavaBinaryOperator::LessEqual,
            JavaPrecedence::Relational,
            true,
        ),
        hir::BinOpKind::Gt => (
            JavaBinaryOperator::Greater,
            JavaPrecedence::Relational,
            true,
        ),
        hir::BinOpKind::Ge => (
            JavaBinaryOperator::GreaterEqual,
            JavaPrecedence::Relational,
            true,
        ),
        _ => panic!("unadmitted operator"),
    };
    assert_eq!(*operator, expected);
    assert_eq!(target.precedence, precedence);
    let boolean = matches!(
        reader.checked.expr_ty_adjusted(left_source).kind(),
        ty::Bool
    );
    let mut positions = Vec::new();
    for (source, operand) in [(left_source, left.as_ref()), (right_source, right.as_ref())] {
        let original = if boolean && ordering {
            assert_eq!(operand.ty, TypePlan::I32.java_type());
            assert_eq!(operand.precedence, JavaPrecedence::Conditional);
            let JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } = &operand.kind
            else {
                panic!("Boolean ordering must use typed conditional int operands")
            };
            assert_eq!(
                **when_true,
                JavaExpr::literal(TypePlan::I32.java_type(), JavaLiteral::I32(1))
            );
            assert_eq!(
                **when_false,
                JavaExpr::literal(TypePlan::I32.java_type(), JavaLiteral::I32(0))
            );
            condition.as_ref()
        } else {
            operand
        };
        assert_eq!(
            original.ty,
            if boolean {
                TypePlan::Bool
            } else {
                TypePlan::I32
            }
            .java_type()
        );
        positions.push(materialized(reader, source, original));
    }
    assert!(positions[0].0 < positions[1].0 && positions[0].1 < positions[1].1);
    if boolean {
        println!("BOOLEAN_AST\t{expected:?}");
    }
    println!("COMPARISON_ORDER");
}
