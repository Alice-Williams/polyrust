//! Exact catalogue call shape, one temporary operand and original callable identities.
use super::TruncationInput;
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
#[path = "floating_c_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &TruncationInput<'tcx>,
    value: &CValue,
    start: usize,
) {
    input.probe(reader.tcx, reader.checked);
    let CValueKind::Call(call) = value.kind() else {
        panic!("catalogue call")
    };
    assert!(matches!(
        call.callable().kind(),
        CCallableKind::Known(portable_backend_c::dialect::CKnownCall::FloatTruncate)
    ));
    assert_eq!(call.arguments().len(), 1);
    let operand = &call.arguments()[0];
    let expressions = reader.expressions();
    let expected = expressions
        .call_value(
            expressions.known(portable_backend_c::dialect::CKnownCall::FloatTruncate),
            vec![operand.clone()],
        )
        .unwrap();
    assert_eq!(value, &expected, "complete exact typed catalogue call");
    assert_eq!(
        operand.ty().kind(),
        &CObjectTypeKind::Scalar(CScalarType::F64)
    );
    let CValueKind::Read(place) = operand.kind() else {
        panic!("pure receiver local")
    };
    let CPlaceKind::Local(local) = place.kind() else {
        panic!("receiver local identity")
    };
    if dataflow::check(reader, input.receiver(), operand) {
        eprintln!("TRUNCATION_DETACHED\tc");
    }
    let prelude = &reader.prelude[start..];
    let CStatementKind::Declare(last) = prelude.last().unwrap().kind() else {
        panic!("materialization")
    };
    assert_eq!(last.local(), local);
    let mut actual = Vec::new();
    for statement in prelude {
        let CStatementKind::Declare(declaration) = statement.kind() else {
            panic!("declaration")
        };
        let CInitializerKind::Expression(value) = declaration.initializer().unwrap().kind() else {
            panic!("initializer")
        };
        if let CValueKind::Call(call) = value.kind() {
            let CCallableKind::Direct(function) = call.callable().kind() else {
                panic!("direct receiver call")
            };
            actual.push(function.as_ref());
        }
    }
    let expected: Vec<_> = source::calls(reader.checked, input.receiver())
        .into_iter()
        .map(|id| match id.as_local() {
            Some(id) => &reader.functions[&id],
            None => &reader.foreign_functions[&id],
        })
        .collect();
    assert_eq!(actual, expected, "exact receiver calls and original owners");
    eprintln!("TRUNCATION_AST\tc\tF64");
}
