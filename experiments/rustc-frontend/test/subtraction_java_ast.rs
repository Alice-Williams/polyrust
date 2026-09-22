//! Exact primitive Subtract, original ordered producers and connected typed dataflow.
use super::{SubtractionInput, SubtractionWidth};
use crate::java_lower::Reader;
use portable_backend_java::ast::*;
#[path = "subtraction_java_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &SubtractionInput<'tcx>,
    value: &JavaExpr,
    start: usize,
    middle: usize,
) {
    let expression = input.probe(reader.tcx, reader.checked);
    let wanted = dataflow::expected(reader, expression);
    assert_eq!(dataflow::expanded(value, &reader.prelude, 0), wanted);
    let primitive = match input.width() {
        SubtractionWidth::I32 => JavaPrimitive::Int,
        SubtractionWidth::I64 => JavaPrimitive::Long,
    };
    assert_eq!(value.ty, JavaType::primitive(primitive));
    assert_eq!(value.precedence, JavaPrecedence::Additive);
    let JavaExprKind::Binary {
        operator: JavaBinaryOperator::Subtract,
        left,
        right,
    } = &value.kind
    else {
        panic!("exact Subtract")
    };
    for (operand, source_expression, statements) in [
        (left, input.left(), &reader.prelude[start..middle]),
        (right, input.right(), &reader.prelude[middle..]),
    ] {
        assert_eq!(operand.ty, value.ty);
        let JavaExprKind::Value(JavaValueRef::Local(local)) = &operand.kind else {
            panic!("operand temporary")
        };
        let JavaStmt::Local {
            name,
            ty,
            value: Some(_),
            finality: JavaLocalFinality::Final,
        } = statements.last().unwrap()
        else {
            panic!("final materialization")
        };
        assert_eq!(local, name);
        assert_eq!(*ty, value.ty);
        assert_eq!(
            dataflow::expanded(operand, &reader.prelude, 0),
            dataflow::expected(reader, source_expression)
        );
        let actual: Vec<_> = statements
            .iter()
            .filter_map(|s| {
                let JavaStmt::Local { value: Some(v), .. } = s else {
                    panic!("initialized local")
                };
                if let JavaExprKind::Call {
                    callable,
                    receiver: None,
                    ..
                } = &v.kind
                {
                    Some(callable.clone())
                } else {
                    None
                }
            })
            .collect();
        let expected: Vec<_> = source::calls(reader.checked, source_expression)
            .into_iter()
            .map(|id| match id.as_local() {
                Some(id) => {
                    let f = &reader.functions[&id];
                    JavaCallableRef::Generated {
                        symbol: f.id,
                        signature: f.signature.clone(),
                    }
                }
                None => JavaCallableRef::Dependency(reader.imported[&id].clone()),
            })
            .collect();
        assert_eq!(
            actual, expected,
            "ordered once-only original producers per operand"
        );
    }
    let mut detached = reader.prelude.clone();
    let JavaStmt::Local {
        value: Some(last), ..
    } = detached.last_mut().unwrap()
    else {
        panic!("right temporary")
    };
    *last = *left.clone();
    assert_ne!(
        dataflow::expanded(value, &detached, 0),
        wanted,
        "original calls retained but result disconnected"
    );
    let mut wrong = value.clone();
    let JavaExprKind::Binary { left, right, .. } = &mut wrong.kind else {
        unreachable!()
    };
    std::mem::swap(left, right);
    assert_ne!(
        dataflow::expanded(&wrong, &reader.prelude, 0),
        wanted,
        "noncommutative values retain ordered source operands"
    );
    eprintln!("SUBTRACTION_DETACHED\tjava");
    eprintln!("SUBTRACTION_AST\tjava\t{:?}", input.width());
}
