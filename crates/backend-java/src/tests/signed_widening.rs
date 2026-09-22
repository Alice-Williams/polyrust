//! Exact widening certification with original dependency and resource proofs.
use crate::{ast::*, dialect::*, tests::source_dependency_fixture as f};
use portable_codegen::*;

#[path = "signed_widening_fixture.rs"]
mod fixture;
#[path = "signed_widening_native.rs"]
mod native;
use fixture::*;

#[test]
fn signed_widening_public_certificate_checks_types_and_annotations() {
    let accepted = |value| admitted(f::package(820, vec![function(value)]));
    let base = cast(input());
    assert!(accepted(base.clone()));
    for precedence in [
        JavaPrecedence::Assignment,
        JavaPrecedence::Conditional,
        JavaPrecedence::LogicalOr,
        JavaPrecedence::LogicalAnd,
        JavaPrecedence::BitOr,
        JavaPrecedence::BitXor,
        JavaPrecedence::BitAnd,
        JavaPrecedence::Equality,
        JavaPrecedence::Relational,
        JavaPrecedence::Shift,
        JavaPrecedence::Additive,
        JavaPrecedence::Multiplicative,
        JavaPrecedence::Unary,
        JavaPrecedence::Primary,
    ] {
        let mut value = base.clone();
        value.precedence = precedence;
        assert_eq!(accepted(value), precedence == JavaPrecedence::Unary);
    }
    for ty in [
        JavaPrimitive::Byte,
        JavaPrimitive::Char,
        JavaPrimitive::Int,
        JavaPrimitive::Long,
        JavaPrimitive::Double,
        JavaPrimitive::Boolean,
        JavaPrimitive::Void,
    ]
    .into_iter()
    .map(JavaType::primitive)
    .chain([
        JavaType::Boxed(JavaPrimitive::Int),
        JavaType::Boxed(JavaPrimitive::Long),
        JavaType::known(JavaKnownType::String),
    ]) {
        for side in 0..3 {
            let mut value = base.clone();
            let JavaExprKind::Cast {
                target,
                value: operand,
            } = &mut value.kind
            else {
                unreachable!()
            };
            match side {
                0 => operand.ty = ty.clone(),
                1 => *target = ty.clone(),
                _ => value.ty = ty.clone(),
            }
            assert_eq!(
                accepted(value),
                ty == if side == 0 { f::int() } else { wide() },
                "{ty:?} side {side}"
            );
        }
    }
}

#[test]
fn signed_widening_rejects_nested_unadmitted_operands_and_excessive_depth() {
    let accepted = |value| admitted(f::package(820, vec![function(cast(value))]));
    for operator in [
        JavaBinaryOperator::Divide,
        JavaBinaryOperator::Remainder,
        JavaBinaryOperator::ShiftLeft,
        JavaBinaryOperator::ShiftRight,
    ] {
        let value = JavaExpr {
            ty: f::int(),
            precedence: match operator {
                JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => {
                    JavaPrecedence::Shift
                }
                _ => JavaPrecedence::Multiplicative,
            },
            kind: JavaExprKind::Binary {
                operator,
                left: Box::new(input()),
                right: Box::new(JavaExpr::literal(f::int(), JavaLiteral::I32(1))),
            },
        };
        assert!(!accepted(value));
    }
    let mut operand = input();
    for _ in 0..32 {
        operand = JavaExpr {
            ty: f::int(),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Unary {
                operator: JavaUnaryOperator::BitNot,
                operand: Box::new(operand),
            },
        };
    }
    assert!(accepted(operand.clone()));
    let owner = api(f::package(820, vec![function(cast(operand.clone()))]));
    let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
    assert!(
        output
            .files()
            .iter()
            .map(|file| match file.contents() {
                OutputContents::Text(text) => text.len() as u64,
                _ => panic!("text"),
            })
            .sum::<u64>()
            <= owner.source_byte_bound().unwrap()
    );
    for _ in 0..128 {
        operand = JavaExpr {
            ty: f::int(),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Unary {
                operator: JavaUnaryOperator::BitNot,
                operand: Box::new(operand),
            },
        };
    }
    assert!(!accepted(operand));
}

#[test]
fn signed_widening_keeps_original_import_authority_and_arity() {
    let original = api(f::package(811, vec![function(input())]));
    let independent = api(f::package(811, vec![function(input())]));
    for registration in 0..3 {
        for arity in 0..3 {
            let (call, bindings) = imported(
                original.functions().next().unwrap().clone(),
                vec![input(); arity],
            );
            let bindings = match registration {
                0 => JavaDependencyScope::new().finish(),
                1 => bindings,
                _ => {
                    imported(
                        independent.functions().next().unwrap().clone(),
                        vec![input()],
                    )
                    .1
                }
            };
            let package = f::package_with_dependencies(812, vec![function(cast(call))], bindings);
            assert_eq!(admitted(package), registration == 1 && arity == 1);
        }
    }
}

#[test]
fn signed_widening_traverses_call_height_and_source_byte_accounting() {
    for materialized in [false, true] {
        for (index, owner) in chain(materialized).iter().enumerate() {
            assert_eq!(owner.functions().count(), 1);
            assert_eq!(owner.functions().next().unwrap().call_height(), index + 1);
            let output =
                render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
            assert_eq!(output.files().len(), 1);
            let OutputContents::Text(text) = output.files()[0].contents() else {
                panic!("text")
            };
            assert!(text.len() as u64 <= owner.source_byte_bound().unwrap());
            assert!(
                !text.contains("Runtime") && !text.contains("Math.") && !text.contains("import ")
            );
            if index == 1 {
                assert!(text.contains("r000000000000032b"));
            }
        }
    }
}
