//! Exact Double operators/precedence, source dataflow, and ordered calls.
use super::ArithmeticInput;
use crate::java_lower::Reader;
use portable_backend_java::ast::*;
#[path = "floating_java_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &ArithmeticInput<'tcx>,
    value: &JavaExpr,
    start: usize,
    middle: usize,
) {
    let source = input.probe(reader.tcx, reader.checked);
    assert!(!dataflow::check(reader, source, value));
    let JavaExprKind::Binary { left, right, .. } = &value.kind else {
        panic!("binary node")
    };
    for (operand, expression, statements) in [
        (left, input.left(), &reader.prelude[start..middle]),
        (right, input.right(), &reader.prelude[middle..]),
    ] {
        assert_eq!(operand.ty, JavaType::primitive(JavaPrimitive::Double));
        let JavaExprKind::Value(JavaValueRef::Local(local)) = &operand.kind else {
            panic!("materialized operand")
        };
        let JavaStmt::Local {
            name,
            value: Some(_),
            ..
        } = statements.last().unwrap()
        else {
            panic!("declaration")
        };
        assert_eq!(local, name);
        let actual: Vec<_> = statements
            .iter()
            .filter_map(|s| {
                let JavaStmt::Local { value: Some(v), .. } = s else {
                    panic!("prelude declaration")
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
        let expected: Vec<_> = source::calls(reader.checked, expression)
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
            "each operand's original calls, exactly once and in order"
        );
    }
    eprintln!("ARITHMETIC_AST\tjava\t{:?}", input.operator());
}
