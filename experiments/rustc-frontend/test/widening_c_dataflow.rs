//! Reconstruct only this fixture's operand grammar, independently of lowering.
use crate::c_lower::Reader;
use crate::source_capabilities::{LiteralInput, LiteralValue};
use portable_backend_c::ast::*;
use rustc_hir::{self as hir, def::Res};

fn expected<'tcx>(reader: &Reader<'tcx>, source: &'tcx hir::Expr<'tcx>) -> CValue {
    let e = reader.expressions();
    match source.kind {
        hir::ExprKind::Path(ref path) => {
            let Res::Local(id) = reader.checked.qpath_res(path, source.hir_id) else {
                panic!("local")
            };
            e.read(reader.bindings[&id].clone()).unwrap()
        }
        hir::ExprKind::Unary(hir::UnOp::Deref, base) => e
            .read(e.dereference(expected(reader, base)).unwrap())
            .unwrap(),
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
        hir::ExprKind::Lit(_) | hir::ExprKind::Unary(hir::UnOp::Neg, _) => {
            let LiteralValue::I32(value) = LiteralInput::read(reader.tcx, reader.checked, source)
                .unwrap()
                .value()
            else {
                panic!("i32 literal")
            };
            e.literal(CLiteral::Signed(CSignedLiteral::I32(value)))
                .unwrap()
        }
        _ => panic!("closed widening probe source grammar"),
    }
}
fn expanded(reader: &Reader<'_>, value: &CValue, prelude: &[CStatement], depth: usize) -> CValue {
    assert!(depth < 128);
    let e = reader.expressions();
    match value.kind() {
        CValueKind::Read(place) => {
            if let CPlaceKind::Local(local) = place.kind() {
                let matches: Vec<_> = prelude
                    .iter()
                    .filter_map(|statement| match statement.kind() {
                        CStatementKind::Declare(d) if d.local() == local => Some(d),
                        _ => None,
                    })
                    .collect();
                assert!(matches.len() <= 1);
                if let Some(d) = matches.first() {
                    let CInitializerKind::Expression(value) = d.initializer().unwrap().kind()
                    else {
                        panic!("initializer")
                    };
                    return expanded(reader, value, prelude, depth + 1);
                }
            }
            value.clone()
        }
        CValueKind::Call(call) => e
            .call_value(
                call.callable().clone(),
                call.arguments()
                    .iter()
                    .map(|arg| expanded(reader, arg, prelude, depth + 1))
                    .collect(),
            )
            .unwrap(),
        CValueKind::Literal(_) => value.clone(),
        _ => panic!("closed widening probe target grammar"),
    }
}
pub(super) fn check<'tcx>(
    reader: &Reader<'tcx>,
    source: &'tcx hir::Expr<'tcx>,
    operand: &CValue,
) -> bool {
    let wanted = expected(reader, source);
    assert_eq!(
        expanded(reader, operand, &reader.prelude, 0),
        wanted,
        "original operand dataflow"
    );
    if let hir::ExprKind::Call(_, [argument]) = source.kind
        && matches!(argument.kind, hir::ExprKind::Path(_))
    {
        let mut detached = reader.prelude.clone();
        let CStatementKind::Declare(last) = detached.last().unwrap().kind() else {
            panic!("temporary")
        };
        *detached.last_mut().unwrap() = reader
            .statements()
            .unwrap()
            .declare(
                last.local().clone(),
                Some(
                    reader
                        .expressions()
                        .expression_initializer(expected(reader, argument))
                        .unwrap(),
                ),
            )
            .unwrap();
        assert_eq!(
            &detached[..detached.len() - 1],
            &reader.prelude[..reader.prelude.len() - 1]
        );
        assert_ne!(
            expanded(reader, operand, &detached, 0),
            wanted,
            "calls retained but result disconnected"
        );
        true
    } else {
        false
    }
}
