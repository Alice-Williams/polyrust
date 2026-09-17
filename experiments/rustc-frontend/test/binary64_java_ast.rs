//! Primitive Double and exact finite bits, without generated conversion helpers.
use super::{LiteralInput, LiteralValue};
use crate::java_lower::{Reader, TypePlan, Value};
use portable_backend_java::ast::*;
pub(super) fn check<'tcx>(reader: &Reader<'tcx>, input: LiteralInput<'tcx>, value: &Value) {
    let LiteralValue::F64(bits) = input.value() else {
        return;
    };
    input.probe(reader.tcx, reader.checked);
    assert_eq!(value.plan(), &TypePlan::F64);
    let expression = value.clone().into_expression();
    assert_eq!(expression.ty, JavaType::primitive(JavaPrimitive::Double));
    assert_eq!(
        expression.kind,
        JavaExprKind::Literal(JavaLiteral::F64(bits))
    );
    eprintln!("BINARY64_AST\tjava\t{:016x}", bits.to_bits());
}

#[path = "binary64_comparison_source.rs"]
mod source;
pub(super) fn comparison<'tcx>(
    reader: &Reader<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
    value: &Value,
    start: usize,
) {
    let Some((operator, identities)) = source::operands(reader.checked, expression) else {
        return;
    };
    assert_eq!(value.plan(), &TypePlan::Bool);
    let expression = value.clone().into_expression();
    let JavaExprKind::Binary {
        operator: actual,
        left,
        right,
    } = expression.kind
    else {
        panic!("comparison")
    };
    use rustc_hir::BinOpKind as Op;
    assert_eq!(
        actual,
        match operator {
            Op::Eq => JavaBinaryOperator::Equal,
            Op::Ne => JavaBinaryOperator::NotEqual,
            Op::Lt => JavaBinaryOperator::Less,
            Op::Le => JavaBinaryOperator::LessEqual,
            Op::Gt => JavaBinaryOperator::Greater,
            Op::Ge => JavaBinaryOperator::GreaterEqual,
            _ => panic!("closed comparisons"),
        }
    );
    let mut calls = Vec::new();
    for statement in &reader.prelude[start..] {
        if let JavaStmt::Local {
            value: Some(value), ..
        } = statement
            && let JavaExprKind::Call { callable, .. } = &value.kind
        {
            calls.push(callable.clone());
        }
    }
    let expected: Vec<_> = identities
        .iter()
        .map(|id| {
            let function = &reader.functions[&id.as_local().unwrap()];
            JavaCallableRef::Generated {
                symbol: function.id,
                signature: function.signature.clone(),
            }
        })
        .collect();
    assert_eq!(calls, expected, "original functions in source order");
    for (operand, expected_call) in [left, right].into_iter().zip(expected) {
        assert_eq!(operand.ty, TypePlan::F64.java_type());
        let mut node = operand.as_ref();
        loop {
            match &node.kind {
                JavaExprKind::Value(JavaValueRef::Local(local)) => {
                    node = reader.prelude[start..]
                        .iter()
                        .find_map(|statement| match statement {
                            JavaStmt::Local {
                                name,
                                value: Some(value),
                                ..
                            } if name == local => Some(value),
                            _ => None,
                        })
                        .expect("operand resolves to its materialized local");
                }
                JavaExprKind::Call { callable, .. } => {
                    assert_eq!(
                        callable, &expected_call,
                        "exact left/right callable placement"
                    );
                    break;
                }
                _ => panic!("comparison operand must resolve to its original call"),
            }
        }
    }
    eprintln!("BINARY64_COMPARE\tjava\t{operator:?}");
}
