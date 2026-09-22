//! Reconstruct fixture expressions and substitute actual typed temporary bindings.
use crate::c_lower::Reader;
use crate::source_capabilities::{LiteralInput, LiteralValue};
use portable_backend_c::ast::*;
use rustc_hir::{self as hir, def::Res};
use rustc_middle::ty;
#[path = "subtraction_source_ast.rs"]
mod source;

fn difference(reader: &Reader<'_>, width: CScalarType, left: CValue, right: CValue) -> CValue {
    let e = reader.expressions();
    let (unsigned, max, minus_one) = match width {
        CScalarType::I32 => (
            CScalarType::U32,
            CUnsignedLiteral::U32(0x7fff_ffff),
            CSignedLiteral::I32(-1),
        ),
        CScalarType::I64 => (
            CScalarType::U64,
            CUnsignedLiteral::U64(0x7fff_ffff_ffff_ffff),
            CSignedLiteral::I64(-1),
        ),
        _ => panic!("signed width"),
    };
    let unsigned_difference = e
        .binary(
            CBinaryOperator::Subtract,
            e.numeric_conversion(unsigned, left).unwrap(),
            e.numeric_conversion(unsigned, right).unwrap(),
        )
        .unwrap();
    let guard = e
        .binary(
            CBinaryOperator::LessEqual,
            unsigned_difference.clone(),
            e.literal(CLiteral::Unsigned(max)).unwrap(),
        )
        .unwrap();
    let nonnegative = e
        .numeric_conversion(width, unsigned_difference.clone())
        .unwrap();
    let negative = e
        .binary(
            CBinaryOperator::Subtract,
            e.literal(CLiteral::Signed(minus_one)).unwrap(),
            e.numeric_conversion(
                width,
                e.unary(CUnaryOperator::BitNot, unsigned_difference)
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let value = e
        .conditional(
            e.numeric_conversion(CScalarType::Bool, guard).unwrap(),
            nonnegative,
            negative,
        )
        .unwrap();
    if width == CScalarType::I32 {
        e.numeric_conversion(width, value).unwrap()
    } else {
        value
    }
}

pub(super) fn expected<'tcx>(reader: &Reader<'tcx>, value: &'tcx hir::Expr<'tcx>) -> CValue {
    let e = reader.expressions();
    if let Some((left, right)) = source::operands(reader.tcx, reader.checked, value) {
        let width = match reader.checked.expr_ty(value).kind() {
            ty::Int(ty::IntTy::I32) => CScalarType::I32,
            ty::Int(ty::IntTy::I64) => CScalarType::I64,
            _ => panic!("width"),
        };
        return difference(
            reader,
            width,
            expected(reader, left),
            expected(reader, right),
        );
    }
    match value.kind {
        hir::ExprKind::Path(ref path) => {
            let Res::Local(id) = reader.checked.qpath_res(path, value.hir_id) else {
                panic!("local")
            };
            e.read(reader.bindings[&id].clone()).unwrap()
        }
        hir::ExprKind::Call(callee, arguments) => {
            let hir::ExprKind::Path(ref path) = callee.kind else {
                panic!("callee")
            };
            let Res::Def(_, id) = reader.checked.qpath_res(path, callee.hir_id) else {
                panic!("identity")
            };
            let function = match id.as_local() {
                Some(id) => &reader.functions[&id],
                None => &reader.foreign_functions[&id],
            };
            e.call_value(
                e.direct(function.clone()).unwrap(),
                arguments.iter().map(|arg| expected(reader, arg)).collect(),
            )
            .unwrap()
        }
        hir::ExprKind::Lit(_) => {
            let literal = match LiteralInput::read(reader.tcx, reader.checked, value)
                .unwrap()
                .value()
            {
                LiteralValue::I32(n) => CSignedLiteral::I32(n),
                LiteralValue::I64(n) => CSignedLiteral::I64(n),
                _ => panic!("integer"),
            };
            e.literal(CLiteral::Signed(literal)).unwrap()
        }
        _ => panic!("closed subtraction fixture grammar"),
    }
}

pub(super) fn expanded(
    reader: &Reader<'_>,
    value: &CValue,
    statements: &[CStatement],
    depth: usize,
) -> CValue {
    assert!(depth < 128);
    let e = reader.expressions();
    match value.kind() {
        CValueKind::Read(place) => {
            if let CPlaceKind::Local(local) = place.kind() {
                let definitions: Vec<_> = statements
                    .iter()
                    .filter_map(|statement| match statement.kind() {
                        CStatementKind::Declare(d) if d.local() == local => Some(d),
                        _ => None,
                    })
                    .collect();
                assert!(definitions.len() <= 1);
                if let Some(d) = definitions.first() {
                    let CInitializerKind::Expression(value) = d.initializer().unwrap().kind()
                    else {
                        panic!("initializer")
                    };
                    return expanded(reader, value, statements, depth + 1);
                }
            }
            value.clone()
        }
        CValueKind::Convert {
            conversion: CConversion::Numeric(width),
            operand,
        } => e
            .numeric_conversion(*width, expanded(reader, operand, statements, depth + 1))
            .unwrap(),
        CValueKind::Binary {
            operator,
            left,
            right,
        } => e
            .binary(
                *operator,
                expanded(reader, left, statements, depth + 1),
                expanded(reader, right, statements, depth + 1),
            )
            .unwrap(),
        CValueKind::Unary { operator, operand } => e
            .unary(*operator, expanded(reader, operand, statements, depth + 1))
            .unwrap(),
        CValueKind::Conditional {
            condition,
            then_value,
            else_value,
        } => e
            .conditional(
                expanded(reader, condition, statements, depth + 1),
                expanded(reader, then_value, statements, depth + 1),
                expanded(reader, else_value, statements, depth + 1),
            )
            .unwrap(),
        CValueKind::Call(call) => e
            .call_value(
                call.callable().clone(),
                call.arguments()
                    .iter()
                    .map(|arg| expanded(reader, arg, statements, depth + 1))
                    .collect(),
            )
            .unwrap(),
        CValueKind::Literal(_) => value.clone(),
        _ => panic!("closed subtraction target grammar"),
    }
}
