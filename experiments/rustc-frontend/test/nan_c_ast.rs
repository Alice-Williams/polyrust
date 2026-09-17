//! Exact primitive unary shape, one temporary operand and original callable identities.
use super::NaNInput;
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
#[path = "floating_c_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &NaNInput<'tcx>,
    value: &CValue,
    start: usize,
) {
    input.probe(reader.tcx, reader.checked);
    assert_eq!(
        value.ty().kind(),
        &CObjectTypeKind::Scalar(CScalarType::Bool)
    );
    let CValueKind::Convert {
        conversion: CConversion::Numeric(CScalarType::Bool),
        operand: comparison,
    } = value.kind()
    else {
        panic!("bool normalization")
    };
    assert_eq!(
        comparison.ty().kind(),
        &CObjectTypeKind::Scalar(CScalarType::Int)
    );
    let CValueKind::Binary {
        operator: CBinaryOperator::NotEqual,
        left: operand,
        right,
    } = comparison.kind()
    else {
        panic!("primitive inequality")
    };
    assert_eq!(
        operand, right,
        "both sides read the same single receiver local"
    );
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
        eprintln!("NAN_DETACHED\tc");
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
    eprintln!("NAN_AST\tc\tF64");
}
