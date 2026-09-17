//! Exact primitive unary shape, one temporary operand and original callable identities.
use super::FloatingInput;
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
#[path = "floating_c_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn check<'tcx>(
    reader: &Reader<'tcx>,
    input: &FloatingInput<'tcx>,
    value: &CValue,
    start: usize,
) {
    source::checked_input(reader.tcx, reader.checked, input);
    assert_eq!(
        value.ty().kind(),
        &CObjectTypeKind::Scalar(CScalarType::F64)
    );
    let CValueKind::Unary {
        operator: CUnaryOperator::Negate,
        operand,
    } = value.kind()
    else {
        panic!("primitive floating negation")
    };
    assert_eq!(operand.ty(), value.ty());
    let CValueKind::Read(place) = operand.kind() else {
        panic!("pure receiver local")
    };
    let CPlaceKind::Local(local) = place.kind() else {
        panic!("receiver local identity")
    };
    dataflow::check(reader, input.operand(), operand);
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
    let expected: Vec<_> = source::calls(reader.checked, input.operand())
        .into_iter()
        .map(|id| match id.as_local() {
            Some(id) => &reader.functions[&id],
            None => &reader.foreign_functions[&id],
        })
        .collect();
    assert_eq!(actual, expected, "exact receiver calls and original owners");
    eprintln!("FLOATING_AST\tc\tF64");
}
