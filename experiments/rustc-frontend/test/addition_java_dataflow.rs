//! Independent source reconstruction and expansion through actual Java locals.
use crate::java_lower::Reader;
use crate::source_capabilities::{LiteralInput, LiteralValue};
use portable_backend_java::ast::*;
use rustc_hir::{self as hir, def::Res};
use rustc_middle::ty;
#[path = "addition_source_ast.rs"]
mod source;

pub(super) fn expected<'tcx>(reader: &Reader<'tcx>, value: &'tcx hir::Expr<'tcx>) -> JavaExpr {
    let primitive = match reader.checked.expr_ty(value).kind() {
        ty::Int(ty::IntTy::I32) => JavaPrimitive::Int,
        ty::Int(ty::IntTy::I64) => JavaPrimitive::Long,
        _ => panic!("width"),
    };
    let ty = JavaType::primitive(primitive);
    if let Some((left, right)) = source::operands(reader.tcx, reader.checked, value) {
        return JavaExpr {
            ty,
            precedence: JavaPrecedence::Additive,
            kind: JavaExprKind::Binary {
                operator: JavaBinaryOperator::Add,
                left: Box::new(expected(reader, left)),
                right: Box::new(expected(reader, right)),
            },
        };
    }
    match value.kind {
        hir::ExprKind::Path(ref path) => {
            let Res::Local(id) = reader.checked.qpath_res(path, value.hir_id) else {
                panic!("local")
            };
            reader.bindings[&id].value().into_expression()
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
                    let function = &reader.functions[&id];
                    JavaCallableRef::Generated {
                        symbol: function.id,
                        signature: function.signature.clone(),
                    }
                }
                None => JavaCallableRef::Dependency(reader.imported[&id].clone()),
            };
            JavaExpr {
                ty,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable,
                    receiver: None,
                    arguments: arguments.iter().map(|arg| expected(reader, arg)).collect(),
                },
            }
        }
        hir::ExprKind::Lit(_) => {
            let literal = match LiteralInput::read(reader.tcx, reader.checked, value)
                .unwrap()
                .value()
            {
                LiteralValue::I32(n) => JavaLiteral::I32(n),
                LiteralValue::I64(n) => JavaLiteral::I64(n),
                _ => panic!("integer"),
            };
            JavaExpr::literal(ty, literal)
        }
        _ => panic!("closed addition fixture grammar"),
    }
}

pub(super) fn expanded(value: &JavaExpr, statements: &[JavaStmt], depth: usize) -> JavaExpr {
    assert!(depth < 128);
    let mut output = value.clone();
    match &mut output.kind {
        JavaExprKind::Value(JavaValueRef::Local(name)) => {
            let definitions: Vec<_> = statements
                .iter()
                .filter_map(|s| match s {
                    JavaStmt::Local {
                        name: n,
                        value: Some(value),
                        ..
                    } if n == name => Some(value),
                    _ => None,
                })
                .collect();
            assert!(definitions.len() <= 1);
            if let Some(value) = definitions.first() {
                return expanded(value, statements, depth + 1);
            }
        }
        JavaExprKind::Binary { left, right, .. } => {
            **left = expanded(left, statements, depth + 1);
            **right = expanded(right, statements, depth + 1);
        }
        JavaExprKind::Call {
            arguments,
            receiver: None,
            ..
        } => {
            for argument in arguments {
                *argument = expanded(argument, statements, depth + 1);
            }
        }
        JavaExprKind::Literal(_) => {}
        _ => panic!("closed addition target grammar"),
    }
    output
}
