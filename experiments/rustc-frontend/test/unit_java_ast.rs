//! Unit mapper output is a direct void invocation or structured non-returning effects.
use super::*;
use crate::java_lower::functions;
use portable_backend_java::ast::{JavaCallableRef, JavaExprKind, JavaValueRef};
use rustc_hir as hir;
#[path = "unit_source_ast.rs"]
mod source;

pub(super) fn check<'tcx>(reader: &Reader<'tcx>, input: &UnitInput<'tcx>, statements: &[JavaStmt]) {
    assert!(reader.prelude.is_empty());
    let label = match input.operation() {
        UnitOperation::Empty => {
            assert!(statements.is_empty());
            "Empty"
        }
        UnitOperation::Call(source) => {
            let JavaStmt::Expression(expression) = statements.last().unwrap() else {
                panic!("unit effect statement")
            };
            assert_eq!(expression.ty, JavaType::primitive(JavaPrimitive::Void));
            let JavaExprKind::Call {
                callable,
                receiver: None,
                arguments,
            } = &expression.kind
            else {
                panic!("direct call")
            };
            let target = functions::resolve(reader.tcx, reader.checked, source).unwrap();
            match (target.as_local(), callable) {
                (Some(id), JavaCallableRef::Generated { symbol, signature }) => {
                    assert_eq!(*symbol, reader.functions[&id].id);
                    assert_eq!(*signature, reader.functions[&id].signature);
                }
                (None, JavaCallableRef::Dependency(value)) => {
                    assert_eq!(value, &reader.imported[&target])
                }
                _ => panic!("call lost original target identity"),
            }
            let hir::ExprKind::Call(_, source_arguments) = source.kind else {
                unreachable!()
            };
            assert_eq!(arguments.len(), source_arguments.len());
            let prelude = &statements[..statements.len() - 1];
            let mut offset = 0;
            for (source, argument) in source_arguments.iter().zip(arguments) {
                offset += source::counts(reader.checked, source).1;
                let JavaStmt::Local { name, .. } = &prelude[offset] else {
                    panic!("argument declaration")
                };
                let JavaExprKind::Value(JavaValueRef::Local(local)) = &argument.kind else {
                    panic!("argument local")
                };
                assert_eq!(name, local, "source-ordered argument materialization");
                offset += 1;
                assert!(matches!(
                    argument.kind,
                    JavaExprKind::Value(JavaValueRef::Local(_))
                ));
                assert_ne!(argument.ty, JavaType::primitive(JavaPrimitive::Void));
            }
            assert_eq!(
                prelude.len(),
                offset,
                "exactly the source argument/call temporaries"
            );
            for statement in prelude {
                let JavaStmt::Local {
                    ty,
                    value: Some(value),
                    ..
                } = statement
                else {
                    panic!("prelude is initialized declarations only")
                };
                assert_ne!(*ty, JavaType::primitive(JavaPrimitive::Void));
                assert_ne!(value.ty, JavaType::primitive(JavaPrimitive::Void));
            }
            "Call"
        }
        UnitOperation::Block(_) => "Block",
        UnitOperation::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            if matches!(condition.kind, hir::ExprKind::Call(..)) {
                assert_eq!(
                    statements.len(),
                    3,
                    "condition argument and result precede the branch"
                );
            }
            assert!(matches!(statements.last().unwrap(), JavaStmt::If { .. }));
            let JavaStmt::If {
                then_block,
                else_block,
                ..
            } = statements.last().unwrap()
            else {
                unreachable!()
            };
            assert_eq!(
                effect_count(&then_block.statements),
                source::counts(reader.checked, then_value).0
            );
            assert_eq!(
                effect_count(else_block.as_ref().map_or(&[], |block| &block.statements)),
                else_value.map_or(0, |value| source::counts(reader.checked, value).0)
            );
            assert!(
                statements[..statements.len() - 1]
                    .iter()
                    .all(|value| matches!(value, JavaStmt::Local { .. })),
                "condition prelude has no branch effects"
            );
            "Conditional"
        }
    };
    no_return(statements);
    eprintln!("UNIT_AST\tjava\t{label}");
}
fn no_return(statements: &[JavaStmt]) {
    for statement in statements {
        match statement {
            JavaStmt::Return(_) => panic!("unit effect returned from its containing function"),
            JavaStmt::If {
                then_block,
                else_block,
                ..
            } => {
                no_return(&then_block.statements);
                if let Some(block) = else_block {
                    no_return(&block.statements);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn completion(
    reader: &Reader<'_>,
    input: &super::super::ControlInput<'_>,
    statements: &[JavaStmt],
) {
    use super::super::ControlCompletion;
    if input.completion == ControlCompletion::Effect {
        no_return(statements);
    } else if matches!(reader.checked.expr_ty(input.expression).kind(), rustc_middle::ty::Tuple(fields) if fields.is_empty())
    {
        assert!(matches!(statements.last().unwrap(), JavaStmt::Return(None)));
        no_return(&statements[..statements.len() - 1]);
        eprintln!("UNIT_COMPLETION\tjava");
    }
}

fn effect_count(statements: &[JavaStmt]) -> usize {
    statements
        .iter()
        .map(|value| match value {
            JavaStmt::Expression(value) if value.ty == JavaType::primitive(JavaPrimitive::Void) => {
                1
            }
            JavaStmt::If {
                then_block,
                else_block,
                ..
            } => {
                effect_count(&then_block.statements)
                    + effect_count(else_block.as_ref().map_or(&[], |block| &block.statements))
            }
            _ => 0,
        })
        .sum()
}
