//! Exact guard shape, one temporary receiver and original callable identities.
use super::{WrappingInput, WrappingWidth};
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
#[path = "wrapping_source_ast.rs"]
mod source;

pub(super) fn check<'tcx>(
    reader: &Reader<'tcx>,
    input: &WrappingInput<'tcx>,
    value: &CValue,
    start: usize,
) {
    source::checked_input(reader.tcx, reader.checked, input);
    let (scalar, literal) = match input.width() {
        WrappingWidth::I32 => (CScalarType::I32, CSignedLiteral::I32(i32::MIN)),
        WrappingWidth::I64 => (CScalarType::I64, CSignedLiteral::I64(i64::MIN)),
    };
    assert_eq!(value.ty().kind(), &CObjectTypeKind::Scalar(scalar));
    let operation = if scalar == CScalarType::I32 {
        let CValueKind::Convert {
            conversion: CConversion::Numeric(CScalarType::I32),
            operand,
        } = value.kind()
        else {
            panic!("exact I32 normalization")
        };
        assert_eq!(
            operand.ty().kind(),
            &CObjectTypeKind::Scalar(CScalarType::Int)
        );
        operand.as_ref()
    } else {
        value
    };
    let CValueKind::Conditional {
        condition,
        then_value,
        else_value,
    } = operation.kind()
    else {
        panic!("guarded negation")
    };
    // C's conditional usual arithmetic conversion promotes the i32 minimum arm.
    let minimum = match then_value.kind() {
        CValueKind::Convert {
            conversion: CConversion::Numeric(CScalarType::Int),
            operand,
        } => operand.as_ref(),
        _ => then_value.as_ref(),
    };
    assert_eq!(
        minimum.kind(),
        &CValueKind::Literal(CLiteral::Signed(literal))
    );
    let CValueKind::Unary {
        operator: CUnaryOperator::Negate,
        operand,
    } = else_value.kind()
    else {
        panic!("negative branch")
    };
    let CValueKind::Convert {
        conversion: CConversion::Numeric(CScalarType::Bool),
        operand: comparison,
    } = condition.kind()
    else {
        panic!("typed boolean condition")
    };
    let CValueKind::Binary {
        operator: CBinaryOperator::Equal,
        left,
        right,
    } = comparison.kind()
    else {
        panic!("minimum equality")
    };
    assert_eq!(left, operand);
    assert_eq!(right.as_ref(), minimum);
    let CValueKind::Read(place) = operand.kind() else {
        panic!("pure receiver local")
    };
    let CPlaceKind::Local(local) = place.kind() else {
        panic!("receiver local identity")
    };
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
    eprintln!("WRAPPING_AST\tc\t{:?}", input.width());
}
