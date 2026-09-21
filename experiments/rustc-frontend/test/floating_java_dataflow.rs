//! Read-only checked-source reconstruction, independent of expression lowering.
use crate::java_lower::Reader;
use crate::source_capabilities::{LiteralInput, LiteralValue};
use portable_backend_java::ast::*;
use rustc_hir::{self as hir, def::Res};

fn expected<'tcx>(reader: &Reader<'tcx>, source: &'tcx hir::Expr<'tcx>) -> JavaExpr {
    let double = JavaType::primitive(JavaPrimitive::Double);
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
        hir::ExprKind::Unary(hir::UnOp::Neg, operand)
            if !matches!(operand.kind, hir::ExprKind::Lit(_)) =>
        {
            JavaExpr {
                ty: double,
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Unary {
                    operator: JavaUnaryOperator::Negate,
                    operand: Box::new(expected(reader, operand)),
                },
            }
        }
        hir::ExprKind::Binary(operator, left, right) => {
            let (operator, precedence) = match operator.node {
                hir::BinOpKind::Add => (JavaBinaryOperator::Add, JavaPrecedence::Additive),
                hir::BinOpKind::Sub => (JavaBinaryOperator::Subtract, JavaPrecedence::Additive),
                hir::BinOpKind::Mul => {
                    (JavaBinaryOperator::Multiply, JavaPrecedence::Multiplicative)
                }
                hir::BinOpKind::Div => (JavaBinaryOperator::Divide, JavaPrecedence::Multiplicative),
                hir::BinOpKind::Rem => (
                    JavaBinaryOperator::Remainder,
                    JavaPrecedence::Multiplicative,
                ),
                _ => panic!("arithmetic operator"),
            };
            JavaExpr {
                ty: double,
                precedence,
                kind: JavaExprKind::Binary {
                    operator,
                    left: Box::new(expected(reader, left)),
                    right: Box::new(expected(reader, right)),
                },
            }
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
                ty: double,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable,
                    receiver: None,
                    arguments: arguments.iter().map(|arg| expected(reader, arg)).collect(),
                },
            }
        }
        hir::ExprKind::Lit(_) | hir::ExprKind::Unary(hir::UnOp::Neg, _) => {
            let LiteralValue::F64(bits) = LiteralInput::read(reader.tcx, reader.checked, source)
                .unwrap()
                .value()
            else {
                panic!("f64 literal")
            };
            JavaExpr::literal(double, JavaLiteral::F64(bits))
        }
        _ => panic!("closed floating probe source grammar"),
    }
}

fn expanded(value: &JavaExpr, prelude: &[JavaStmt], depth: usize) -> JavaExpr {
    assert!(depth < 128, "temporary chain cycle/depth");
    let mut result = value.clone();
    match &value.kind {
        JavaExprKind::Value(JavaValueRef::Local(local)) => {
            let matches: Vec<_> = prelude
                .iter()
                .filter_map(|statement| match statement {
                    JavaStmt::Local {
                        name,
                        value: Some(value),
                        ..
                    } if name == local => Some(value),
                    _ => None,
                })
                .collect();
            assert!(matches.len() <= 1, "unique temporary definition");
            if let Some(value) = matches.first() {
                return expanded(value, prelude, depth + 1);
            }
        }
        JavaExprKind::Unary { operator, operand } => {
            result.kind = JavaExprKind::Unary {
                operator: *operator,
                operand: Box::new(expanded(operand, prelude, depth + 1)),
            }
        }
        JavaExprKind::Binary {
            operator,
            left,
            right,
        } => {
            result.kind = JavaExprKind::Binary {
                operator: *operator,
                left: Box::new(expanded(left, prelude, depth + 1)),
                right: Box::new(expanded(right, prelude, depth + 1)),
            };
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
            }
        }
        JavaExprKind::Literal(_) => {}
        _ => panic!("closed floating probe target grammar"),
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
        "exact operand dataflow, not only call presence"
    );
    if let JavaExprKind::Binary {
        operator,
        left,
        right,
    } = &operand.kind
    {
        let mut wrong = operand.clone();
        wrong.kind = JavaExprKind::Binary {
            operator: if *operator == JavaBinaryOperator::Add {
                JavaBinaryOperator::Subtract
            } else {
                JavaBinaryOperator::Add
            },
            left: left.clone(),
            right: right.clone(),
        };
        assert_ne!(
            expanded(&wrong, &reader.prelude, 0),
            wanted,
            "wrong actual operator must fail the source reconstruction"
        );
        if expanded(left, &reader.prelude, 0) != expanded(right, &reader.prelude, 0) {
            let mut detached = reader.prelude.clone();
            let JavaStmt::Local { value, .. } = detached.last_mut().unwrap() else {
                panic!("right temporary")
            };
            *value = Some(left.as_ref().clone());
            assert_eq!(
                &detached[..detached.len() - 1],
                &reader.prelude[..reader.prelude.len() - 1]
            );
            assert_ne!(
                expanded(operand, &detached, 0),
                wanted,
                "right result disconnected while all original calls remain"
            );
            #[cfg(arithmetic_ast_probe)]
            eprintln!("ARITHMETIC_DETACHED\tjava");
            #[cfg(remainder_ast_probe)]
            eprintln!("REMAINDER_DETACHED\tjava");
        }
    }
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
            "disconnected result must fail"
        );
        true
    } else {
        false
    }
}
