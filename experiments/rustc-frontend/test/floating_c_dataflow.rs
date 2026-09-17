//! Independent fixture reconstruction and temporary substitution, never lowering.
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
        hir::ExprKind::Unary(hir::UnOp::Neg, operand)
            if !matches!(operand.kind, hir::ExprKind::Lit(_)) =>
        {
            e.unary(CUnaryOperator::Negate, expected(reader, operand))
                .unwrap()
        }
        hir::ExprKind::Binary(operator, left, right) => {
            let operator = match operator.node {
                hir::BinOpKind::Add => CBinaryOperator::Add,
                hir::BinOpKind::Sub => CBinaryOperator::Subtract,
                hir::BinOpKind::Mul => CBinaryOperator::Multiply,
                hir::BinOpKind::Div => CBinaryOperator::Divide,
                _ => panic!("arithmetic operator"),
            };
            e.binary(operator, expected(reader, left), expected(reader, right))
                .unwrap()
        }
        hir::ExprKind::Call(callee, arguments) => {
            let hir::ExprKind::Path(ref path) = callee.kind else {
                panic!("callee path")
            };
            let Res::Def(_, id) = reader.checked.qpath_res(path, callee.hir_id) else {
                panic!("call identity")
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
            let LiteralValue::F64(bits) = LiteralInput::read(reader.tcx, reader.checked, source)
                .unwrap()
                .value()
            else {
                panic!("f64 literal")
            };
            e.literal(CLiteral::F64(bits)).unwrap()
        }
        _ => panic!("closed floating probe source grammar"),
    }
}

fn expanded(reader: &Reader<'_>, value: &CValue, prelude: &[CStatement], depth: usize) -> CValue {
    assert!(depth < 128, "temporary chain cycle/depth");
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
                assert!(matches.len() <= 1, "unique temporary definition");
                if let Some(declaration) = matches.first() {
                    let CInitializerKind::Expression(value) =
                        declaration.initializer().unwrap().kind()
                    else {
                        panic!("initializer")
                    };
                    return expanded(reader, value, prelude, depth + 1);
                }
            }
            value.clone()
        }
        CValueKind::Unary { operator, operand } => e
            .unary(*operator, expanded(reader, operand, prelude, depth + 1))
            .unwrap(),
        CValueKind::Binary {
            operator,
            left,
            right,
        } => e
            .binary(
                *operator,
                expanded(reader, left, prelude, depth + 1),
                expanded(reader, right, prelude, depth + 1),
            )
            .unwrap(),
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
        _ => panic!("closed floating probe target grammar"),
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
        "exact operand dataflow, not only call presence"
    );
    if let CValueKind::Binary {
        operator,
        left,
        right,
    } = operand.kind()
    {
        let wrong = if *operator == CBinaryOperator::Add {
            CBinaryOperator::Subtract
        } else {
            CBinaryOperator::Add
        };
        let e = reader.expressions();
        let changed = e
            .binary(wrong, left.as_ref().clone(), right.as_ref().clone())
            .unwrap();
        assert_ne!(
            expanded(reader, &changed, &reader.prelude, 0),
            wanted,
            "wrong actual operator must fail the source reconstruction"
        );
        if expanded(reader, left, &reader.prelude, 0) != expanded(reader, right, &reader.prelude, 0)
        {
            let mut detached = reader.prelude.clone();
            let CStatementKind::Declare(last) = detached.last().unwrap().kind() else {
                panic!("right temporary")
            };
            *detached.last_mut().unwrap() = reader
                .statements()
                .unwrap()
                .declare(
                    last.local().clone(),
                    Some(e.expression_initializer(left.as_ref().clone()).unwrap()),
                )
                .unwrap();
            assert_eq!(
                &detached[..detached.len() - 1],
                &reader.prelude[..reader.prelude.len() - 1]
            );
            assert_ne!(
                expanded(reader, operand, &detached, 0),
                wanted,
                "right result disconnected while all original calls remain"
            );
            #[cfg(arithmetic_ast_probe)]
            eprintln!("ARITHMETIC_DETACHED\tc");
        }
    }
    if let hir::ExprKind::Call(_, [argument]) = source.kind
        && matches!(argument.kind, hir::ExprKind::Path(_))
    {
        let mut detached = reader.prelude.clone();
        let CStatementKind::Declare(last) = detached.last().unwrap().kind() else {
            panic!("temporary")
        };
        let replacement = expected(reader, argument);
        let statement = reader
            .statements()
            .unwrap()
            .declare(
                last.local().clone(),
                Some(
                    reader
                        .expressions()
                        .expression_initializer(replacement)
                        .unwrap(),
                ),
            )
            .unwrap();
        *detached.last_mut().unwrap() = statement;
        // Every actual call declaration remains; only the final copy's input changes.
        assert_eq!(
            &detached[..detached.len() - 1],
            &reader.prelude[..reader.prelude.len() - 1]
        );
        assert_ne!(
            expanded(reader, operand, &detached, 0),
            wanted,
            "disconnected result must fail"
        );
        true
    } else {
        false
    }
}
