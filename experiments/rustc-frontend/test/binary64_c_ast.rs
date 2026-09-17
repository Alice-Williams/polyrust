//! Typed target literal bits cannot drift from the canonical compiler witness.
use super::{LiteralInput, LiteralValue};
use crate::c_lower::Reader;
use portable_backend_c::ast::*;
pub(super) fn check<'tcx>(reader: &Reader<'tcx>, input: LiteralInput<'tcx>, value: &CValue) {
    let LiteralValue::F64(bits) = input.value() else {
        return;
    };
    input.probe(reader.tcx, reader.checked);
    assert_eq!(
        value.ty().kind(),
        &CObjectTypeKind::Scalar(CScalarType::F64)
    );
    assert_eq!(value.kind(), &CValueKind::Literal(CLiteral::F64(bits)));
    eprintln!("BINARY64_AST\tc\t{:016x}", bits.to_bits());
}

#[path = "binary64_comparison_source.rs"]
mod source;
pub(super) fn comparison<'tcx>(
    reader: &Reader<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
    value: &CValue,
    start: usize,
) {
    let Some((operator, identities)) = source::operands(reader.checked, expression) else {
        return;
    };
    let CValueKind::Convert {
        conversion: CConversion::Numeric(CScalarType::Bool),
        operand,
    } = value.kind()
    else {
        panic!("Boolean normalization")
    };
    let CValueKind::Binary {
        operator: actual,
        left,
        right,
    } = operand.kind()
    else {
        panic!("comparison")
    };
    use rustc_hir::BinOpKind as Op;
    assert_eq!(
        *actual,
        match operator {
            Op::Eq => CBinaryOperator::Equal,
            Op::Ne => CBinaryOperator::NotEqual,
            Op::Lt => CBinaryOperator::Less,
            Op::Le => CBinaryOperator::LessEqual,
            Op::Gt => CBinaryOperator::Greater,
            Op::Ge => CBinaryOperator::GreaterEqual,
            _ => panic!("closed comparisons"),
        }
    );
    let mut calls = Vec::new();
    let mut locals = Vec::new();
    for statement in &reader.prelude[start..] {
        let CStatementKind::Declare(declaration) = statement.kind() else {
            panic!("materialized")
        };
        let CInitializerKind::Expression(value) = declaration.initializer().unwrap().kind() else {
            panic!("initializer")
        };
        if let CValueKind::Call(call) = value.kind() {
            let CCallableKind::Direct(function) = call.callable().kind() else {
                panic!("direct")
            };
            calls.push(function.as_ref());
            locals.push(declaration.local());
        }
    }
    let expected: Vec<_> = identities
        .iter()
        .map(|id| &reader.functions[&id.as_local().unwrap()])
        .collect();
    assert_eq!(calls, expected, "original functions in source order");
    assert_eq!(locals.len(), 2);
    for (operand, local) in [left, right].into_iter().zip(locals) {
        assert_eq!(
            operand.ty().kind(),
            &CObjectTypeKind::Scalar(CScalarType::F64)
        );
        let CValueKind::Read(place) = operand.kind() else {
            panic!("read")
        };
        assert!(matches!(place.kind(), CPlaceKind::Local(id) if id == local));
    }
    eprintln!("BINARY64_COMPARE\tc\t{operator:?}");
}
