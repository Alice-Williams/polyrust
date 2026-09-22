//! Observe exact guarded expression, ordered original calls and connected operands.
use super::{MultiplicationInput, MultiplicationWidth};
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
#[path = "multiplication_c_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &MultiplicationInput<'tcx>,
    value: &CValue,
    start: usize,
    middle: usize,
    end: usize,
) {
    let expression = input.probe(reader.tcx, reader.checked);
    let wanted = dataflow::expected(reader, expression);
    assert_eq!(
        dataflow::expanded(reader, value, &reader.prelude, 0),
        wanted
    );
    let (signed, unsigned) = match input.width() {
        MultiplicationWidth::I32 => (CScalarType::I32, CScalarType::U32),
        MultiplicationWidth::I64 => (CScalarType::I64, CScalarType::U64),
    };
    assert_eq!(value.ty().kind(), &CObjectTypeKind::Scalar(signed));
    assert_eq!(
        reader.prelude.len(),
        end + 1,
        "one unsigned product temporary"
    );
    let e = reader.expressions();
    let mut operands = Vec::new();
    for (source_expression, statements) in [
        (input.left(), &reader.prelude[start..middle]),
        (input.right(), &reader.prelude[middle..end]),
    ] {
        let CStatementKind::Declare(last) = statements.last().unwrap().kind() else {
            panic!("operand materialization")
        };
        let operand = e.read(e.local(last.local().clone()).unwrap()).unwrap();
        assert_eq!(operand.ty().kind(), &CObjectTypeKind::Scalar(signed));
        assert_eq!(
            dataflow::expanded(reader, &operand, &reader.prelude, 0),
            dataflow::expected(reader, source_expression)
        );
        operands.push(operand);
        let actual: Vec<_> = statements
            .iter()
            .filter_map(|s| {
                let CStatementKind::Declare(d) = s.kind() else {
                    panic!("declaration")
                };
                let CInitializerKind::Expression(v) = d.initializer().unwrap().kind() else {
                    panic!("initializer")
                };
                if let CValueKind::Call(call) = v.kind() {
                    let CCallableKind::Direct(function) = call.callable().kind() else {
                        panic!("original call")
                    };
                    Some(function.as_ref())
                } else {
                    None
                }
            })
            .collect();
        let expected: Vec<_> = source::calls(reader.checked, source_expression)
            .into_iter()
            .map(|id| match id.as_local() {
                Some(id) => &reader.functions[&id],
                None => &reader.foreign_functions[&id],
            })
            .collect();
        assert_eq!(
            actual, expected,
            "ordered once-only original producers per operand"
        );
    }
    let CStatementKind::Declare(product) = reader.prelude[end].kind() else {
        panic!("product")
    };
    let CInitializerKind::Expression(product_value) = product.initializer().unwrap().kind() else {
        panic!("product initializer")
    };
    assert_eq!(
        product_value.ty().kind(),
        &CObjectTypeKind::Scalar(unsigned)
    );
    assert_eq!(
        *product_value,
        e.binary(
            CBinaryOperator::Multiply,
            e.numeric_conversion(unsigned, operands[0].clone()).unwrap(),
            e.numeric_conversion(unsigned, operands[1].clone()).unwrap()
        )
        .unwrap()
    );
    let mut detached = reader.prelude.clone();
    let CStatementKind::Declare(right) = detached[end - 1].kind() else {
        panic!("right")
    };
    detached[end - 1] = reader
        .statements()
        .unwrap()
        .declare(
            right.local().clone(),
            Some(e.expression_initializer(operands[0].clone()).unwrap()),
        )
        .unwrap();
    assert_ne!(
        dataflow::expanded(reader, value, &detached, 0),
        wanted,
        "original calls retained but result disconnected"
    );
    eprintln!("MULTIPLICATION_DETACHED\tc");
    eprintln!("MULTIPLICATION_AST\tc\t{:?}", input.width());
}
