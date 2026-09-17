//! Exact primitive conditional shape, one temporary operand and original callable identities.
use super::AbsoluteInput;
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
#[path = "floating_c_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &AbsoluteInput<'tcx>,
    value: &CValue,
    start: usize,
) {
    input.probe(reader.tcx, reader.checked);
    let CValueKind::Conditional { condition, .. } = value.kind() else {
        panic!("outer zero conditional")
    };
    let CValueKind::Convert {
        conversion: CConversion::Numeric(CScalarType::Bool),
        operand: equality,
    } = condition.kind()
    else {
        panic!("Boolean zero condition")
    };
    let CValueKind::Binary {
        operator: CBinaryOperator::Equal,
        left: operand,
        ..
    } = equality.kind()
    else {
        panic!("zero equality")
    };
    let e = reader.expressions();
    let zero = e
        .literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
        ))
        .unwrap();
    let compare = |op| {
        e.numeric_conversion(
            CScalarType::Bool,
            e.binary(op, operand.as_ref().clone(), zero.clone())
                .unwrap(),
        )
        .unwrap()
    };
    let negative = e
        .unary(CUnaryOperator::Negate, operand.as_ref().clone())
        .unwrap();
    let magnitude = e
        .conditional(
            compare(CBinaryOperator::Less),
            negative,
            operand.as_ref().clone(),
        )
        .unwrap();
    let expected = e
        .conditional(compare(CBinaryOperator::Equal), zero, magnitude)
        .unwrap();
    assert_eq!(
        value, &expected,
        "complete typed zero/sign selection and original operand"
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
        eprintln!("ABSOLUTE_DETACHED\tc");
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
    eprintln!("ABSOLUTE_AST\tc\tF64");
}
