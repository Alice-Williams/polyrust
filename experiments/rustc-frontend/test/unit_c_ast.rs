//! Read-only mapper checks against checked source identity and effect nodes.
use super::*;
use crate::c_lower::functions;
use rustc_hir as hir;
#[path = "unit_source_ast.rs"]
mod source;

pub(super) fn check<'tcx>(
    reader: &Reader<'tcx>,
    input: &UnitInput<'tcx>,
    statements: &[CStatement],
) {
    assert!(reader.prelude.is_empty());
    let label = match input.operation() {
        UnitOperation::Empty => {
            assert!(statements.is_empty());
            "Empty"
        }
        UnitOperation::Call(source) => {
            let CStatementKind::Evaluate(effect) = statements.last().unwrap().kind() else {
                panic!("unit call lacks a typed effect");
            };
            let CCallableKind::Direct(actual) = effect.call().callable().kind() else {
                panic!("direct callable")
            };
            let expected = match functions::resolve(reader.tcx, reader.checked, source).unwrap() {
                functions::CallTarget::Local(id) => &reader.functions[&id],
                functions::CallTarget::Foreign(id) => &reader.foreign_functions[&id],
            };
            assert_eq!(actual.as_ref(), expected);
            assert!(matches!(
                actual.signature().return_type(),
                CReturnType::Void
            ));
            let hir::ExprKind::Call(_, arguments) = source.kind else {
                unreachable!()
            };
            assert_eq!(arguments.len(), effect.call().arguments().len());
            let prelude = &statements[..statements.len() - 1];
            let mut offset = 0;
            for (source, argument) in arguments.iter().zip(effect.call().arguments()) {
                offset += source::counts(reader.checked, source).1;
                let CStatementKind::Declare(declaration) = prelude[offset].kind() else {
                    panic!("argument declaration")
                };
                let CValueKind::Read(place) = argument.kind() else {
                    panic!("argument read")
                };
                let CPlaceKind::Local(local) = place.kind() else {
                    panic!("argument local")
                };
                assert_eq!(
                    declaration.local(),
                    local,
                    "source-ordered argument materialization"
                );
                offset += 1;
                assert!(
                    matches!(argument.kind(), CValueKind::Read(place) if matches!(place.kind(), CPlaceKind::Local(_)))
                );
            }
            assert_eq!(
                prelude.len(),
                offset,
                "exactly the source argument/call temporaries"
            );
            for statement in prelude {
                let CStatementKind::Declare(declaration) = statement.kind() else {
                    panic!("prelude is declarations only")
                };
                let CInitializerKind::Expression(value) = declaration.initializer().unwrap().kind()
                else {
                    panic!("value initializer")
                };
                assert!(
                    matches!(value.ty().kind(), CObjectTypeKind::Scalar(_)),
                    "no unit temporary"
                );
            }
            "Call"
        }
        UnitOperation::Block(_) => {
            assert!(
                matches!(statements, [statement] if matches!(statement.kind(), CStatementKind::Block(_)))
            );
            "Block"
        }
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
            assert!(matches!(
                statements.last().unwrap().kind(),
                CStatementKind::If { .. }
            ));
            let CStatementKind::If {
                then_block,
                else_block,
                ..
            } = statements.last().unwrap().kind()
            else {
                unreachable!()
            };
            assert_eq!(
                effect_count(then_block.statements()),
                source::counts(reader.checked, then_value).0
            );
            assert_eq!(
                effect_count(else_block.statements()),
                else_value.map_or(0, |value| source::counts(reader.checked, value).0)
            );
            assert!(
                statements[..statements.len() - 1]
                    .iter()
                    .all(|value| matches!(value.kind(), CStatementKind::Declare(_))),
                "condition prelude has no branch effects"
            );
            "Conditional"
        }
    };
    no_return(statements);
    eprintln!("UNIT_AST\tc\t{label}");
}
fn no_return(statements: &[CStatement]) {
    for statement in statements {
        match statement.kind() {
            CStatementKind::Return(_) => {
                panic!("unit effect returned from its containing function")
            }
            CStatementKind::Block(block) => no_return(block.statements()),
            CStatementKind::If {
                then_block,
                else_block,
                ..
            } => {
                no_return(then_block.statements());
                no_return(else_block.statements());
            }
            _ => {}
        }
    }
}

pub(crate) fn completion(
    reader: &Reader<'_>,
    input: &super::super::ControlInput<'_>,
    statements: &[CStatement],
) {
    use super::super::ControlCompletion;
    if input.completion == ControlCompletion::Effect {
        no_return(statements);
    } else if matches!(reader.checked.expr_ty(input.expression).kind(), rustc_middle::ty::Tuple(fields) if fields.is_empty())
    {
        assert!(matches!(
            statements.last().unwrap().kind(),
            CStatementKind::Return(None)
        ));
        no_return(&statements[..statements.len() - 1]);
        eprintln!("UNIT_COMPLETION\tc");
    }
}

fn effect_count(statements: &[CStatement]) -> usize {
    statements
        .iter()
        .map(|value| match value.kind() {
            CStatementKind::Evaluate(_) => 1,
            CStatementKind::Block(block) => effect_count(block.statements()),
            CStatementKind::If {
                then_block,
                else_block,
                ..
            } => effect_count(then_block.statements()) + effect_count(else_block.statements()),
            _ => 0,
        })
        .sum()
}
