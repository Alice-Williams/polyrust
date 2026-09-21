//! Exact typed arithmetic, original-call order, and expanded operand dataflow.
use super::RemainderInput;
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
#[path = "floating_c_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &RemainderInput<'tcx>,
    value: &CValue,
    start: usize,
    middle: usize,
) {
    let source = input.probe(reader.tcx, reader.checked);
    assert!(!dataflow::check(reader, source, value));
    let CValueKind::Call(call) = value.kind() else {
        panic!("catalogue call")
    };
    assert!(matches!(
        call.callable().kind(),
        CCallableKind::Known(portable_backend_c::dialect::CKnownCall::FloatRemainder)
    ));
    let [left, right] = call.arguments() else {
        panic!("two operands")
    };
    for (operand, expression, statements) in [
        (left, input.left(), &reader.prelude[start..middle]),
        (right, input.right(), &reader.prelude[middle..]),
    ] {
        assert_eq!(
            operand.ty().kind(),
            &CObjectTypeKind::Scalar(CScalarType::F64)
        );
        let CValueKind::Read(place) = operand.kind() else {
            panic!("materialized operand")
        };
        let CPlaceKind::Local(local) = place.kind() else {
            panic!("local")
        };
        let CStatementKind::Declare(last) = statements.last().unwrap().kind() else {
            panic!("declaration")
        };
        assert_eq!(local, last.local());
        let actual: Vec<_> = statements
            .iter()
            .filter_map(|s| {
                let CStatementKind::Declare(d) = s.kind() else {
                    panic!("prelude declaration")
                };
                let CInitializerKind::Expression(v) = d.initializer().unwrap().kind() else {
                    panic!("initializer")
                };
                if let CValueKind::Call(call) = v.kind() {
                    match call.callable().kind() {
                        CCallableKind::Direct(f) => Some(f.as_ref()),
                        CCallableKind::Known(
                            portable_backend_c::dialect::CKnownCall::FloatRemainder,
                        ) => None,
                        _ => panic!("ordinary source call or nested remainder"),
                    }
                } else {
                    None
                }
            })
            .collect();
        let expected: Vec<_> = source::calls(reader.checked, expression)
            .into_iter()
            .map(|id| match id.as_local() {
                Some(id) => &reader.functions[&id],
                None => &reader.foreign_functions[&id],
            })
            .collect();
        assert_eq!(
            actual, expected,
            "each operand's original calls, exactly once and in order"
        );
    }
    eprintln!("REMAINDER_AST\tc");
}
