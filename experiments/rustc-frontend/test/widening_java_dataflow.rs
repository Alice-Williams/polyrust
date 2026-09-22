//! Independent original operand reconstruction and temporary expansion.
use crate::java_lower::Reader;
use crate::source_capabilities::{LiteralInput, LiteralValue};
use portable_backend_java::ast::*;
use rustc_hir::{self as hir, def::Res};
fn expected<'tcx>(reader: &Reader<'tcx>, source: &'tcx hir::Expr<'tcx>) -> JavaExpr {
    let int = JavaType::primitive(JavaPrimitive::Int);
    match source.kind {
        hir::ExprKind::Path(ref path) => {
            let Res::Local(id) = reader.checked.qpath_res(path, source.hir_id) else {
                panic!("local")
            };
            reader.bindings[&id].value().into_expression()
        }
        hir::ExprKind::Unary(hir::UnOp::Deref, base) => {
            let hir::ExprKind::Path(ref path) = base.kind else {
                panic!("borrow path")
            };
            let Res::Local(id) = reader.checked.qpath_res(path, base.hir_id) else {
                panic!("borrow")
            };
            reader.bindings[&id]
                .clone()
                .dereference()
                .unwrap()
                .value()
                .into_expression()
        }
        hir::ExprKind::Call(callee, arguments) => {
            let hir::ExprKind::Path(ref path) = callee.kind else {
                panic!("callee")
            };
            let Res::Def(_, id) = reader.checked.qpath_res(path, callee.hir_id) else {
                panic!("identity")
            };
            let callable = match id.as_local() {
                Some(id) => {
                    let f = &reader.functions[&id];
                    JavaCallableRef::Generated {
                        symbol: f.id,
                        signature: f.signature.clone(),
                    }
                }
                None => JavaCallableRef::Dependency(reader.imported[&id].clone()),
            };
            JavaExpr {
                ty: int,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable,
                    receiver: None,
                    arguments: arguments.iter().map(|arg| expected(reader, arg)).collect(),
                },
            }
        }
        hir::ExprKind::Lit(_) | hir::ExprKind::Unary(hir::UnOp::Neg, _) => {
            let LiteralValue::I32(value) = LiteralInput::read(reader.tcx, reader.checked, source)
                .unwrap()
                .value()
            else {
                panic!("i32 literal")
            };
            JavaExpr::literal(int, JavaLiteral::I32(value))
        }
        _ => panic!("closed widening probe source grammar"),
    }
}
fn expanded(value: &JavaExpr, prelude: &[JavaStmt], depth: usize) -> JavaExpr {
    assert!(depth < 128);
    let mut result = value.clone();
    match &value.kind {
        JavaExprKind::Value(JavaValueRef::Local(local)) => {
            let matches: Vec<_> = prelude
                .iter()
                .filter_map(|s| match s {
                    JavaStmt::Local {
                        name,
                        value: Some(value),
                        ..
                    } if name == local => Some(value),
                    _ => None,
                })
                .collect();
            assert!(matches.len() <= 1);
            if let Some(value) = matches.first() {
                return expanded(value, prelude, depth + 1);
            }
        }
        JavaExprKind::Call {
            callable,
            receiver: None,
            arguments,
        } => {
            result.kind = JavaExprKind::Call {
                callable: callable.clone(),
                receiver: None,
                arguments: arguments
                    .iter()
                    .map(|arg| expanded(arg, prelude, depth + 1))
                    .collect(),
            };
        }
        JavaExprKind::Literal(_) => {}
        _ => panic!("closed widening probe target grammar"),
    }
    result
}
pub(super) fn check<'tcx>(
    reader: &Reader<'tcx>,
    source: &'tcx hir::Expr<'tcx>,
    operand: &JavaExpr,
) -> bool {
    let wanted = expected(reader, source);
    assert_eq!(
        expanded(operand, &reader.prelude, 0),
        wanted,
        "original operand dataflow"
    );
    if let hir::ExprKind::Call(_, [argument]) = source.kind
        && matches!(argument.kind, hir::ExprKind::Path(_))
    {
        let mut detached = reader.prelude.clone();
        let JavaStmt::Local { value, .. } = detached.last_mut().unwrap() else {
            panic!("temporary")
        };
        *value = Some(expected(reader, argument));
        assert_eq!(
            &detached[..detached.len() - 1],
            &reader.prelude[..reader.prelude.len() - 1]
        );
        assert_ne!(
            expanded(operand, &detached, 0),
            wanted,
            "calls retained but result disconnected"
        );
        true
    } else {
        false
    }
}
