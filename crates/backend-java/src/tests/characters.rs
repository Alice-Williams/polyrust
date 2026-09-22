//! Java Int character storage proof is separate from source Unicode validity.
use crate::{ast::*, dialect::*, tests::source_dependency_fixture as f};
use portable_codegen::*;
#[path = "characters_fixture.rs"]
mod fixture;
#[path = "characters_native.rs"]
mod native;
use fixture::*;

#[test]
fn characters_keep_primitive_storage_original_authority_and_resource_bounds() {
    let owners = chain();
    assert_eq!(owners[0].functions().count(), 28);
    for (index, owner) in owners.iter().enumerate() {
        let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
        assert_eq!(output.files().len(), 1);
        let OutputContents::Text(text) = output.files()[0].contents() else {
            panic!("text")
        };
        assert!(text.len() as u64 <= owner.source_byte_bound().unwrap());
        for forbidden in [
            "import ",
            "Runtime",
            "Character",
            "String",
            "char ",
            "Integer",
        ] {
            assert!(!text.contains(forbidden), "{forbidden}");
        }
        assert_eq!(
            owner
                .function(f::id(941 + index as u64, 40))
                .unwrap()
                .call_height(),
            index + 1
        );
    }
    // Independent, text-identical producers do not supply interchangeable certificates.
    let independent = api(f::package(941, functions()));
    let original = owners[0].function(f::id(941, 40)).unwrap();
    for registration in 0..3 {
        for arity in 0..3 {
            let (call, bindings) = imported(original.clone(), vec![input("input"); arity]);
            let bindings = match registration {
                0 => JavaDependencyScope::new().finish(),
                1 => bindings,
                _ => {
                    imported(
                        independent.function(f::id(941, 40)).unwrap().clone(),
                        vec![input("input")],
                    )
                    .1
                }
            };
            let declaration = function(40, "forward", &[("input", f::int())], call);
            assert_eq!(
                admitted(f::package_with_dependencies(
                    942,
                    vec![declaration],
                    bindings
                )),
                registration == 1 && arity == 1
            );
        }
    }
}

#[test]
fn character_target_int_does_not_claim_unicode_validation() {
    for raw in [i32::MIN, -1, 0xd800, 0xdfff, 0x110000, i32::MAX] {
        let value = JavaExpr::literal(f::int(), JavaLiteral::I32(raw));
        assert!(admitted(f::package(
            941,
            vec![function(10, "integer", &[], value)]
        )));
    }
}

#[test]
fn character_comparisons_reconstruct_operand_result_types_and_precedence() {
    let parameters = [("left", f::int()), ("right", f::int())];
    for operator in OPERATORS {
        let correct = comparison(operator);
        assert!(admitted(f::package(
            941,
            vec![function(10, "compare", &parameters, correct.clone())]
        )));
        for side in 0..3 {
            for ty in [
                JavaPrimitive::Char,
                JavaPrimitive::Byte,
                JavaPrimitive::Long,
                JavaPrimitive::Boolean,
                JavaPrimitive::Void,
            ] {
                let mut value = correct.clone();
                let JavaExprKind::Binary { left, right, .. } = &mut value.kind else {
                    unreachable!()
                };
                match side {
                    0 => left.ty = JavaType::primitive(ty),
                    1 => right.ty = JavaType::primitive(ty),
                    _ => value.ty = JavaType::primitive(ty),
                }
                assert_eq!(
                    admitted(f::package(
                        941,
                        vec![function(10, "compare", &parameters, value)]
                    )),
                    side == 2 && ty == JavaPrimitive::Boolean,
                    "{operator:?}, {side}, {ty:?}"
                );
            }
        }
        let mut bad = correct;
        bad.precedence = JavaPrecedence::Primary;
        assert!(!admitted(f::package(
            941,
            vec![function(10, "compare", &parameters, bad)]
        )));
    }
}

#[test]
fn character_conditionals_reconstruct_children_and_reject_unadmitted_casts_depth_and_locals() {
    let parameters = [
        ("condition", f::boolean()),
        ("left", f::int()),
        ("right", f::int()),
    ];
    let accepted = |value| {
        admitted(f::package(
            941,
            vec![function(10, "select", &parameters, value)],
        ))
    };
    assert!(accepted(selection()));
    for fault in 0..5 {
        let mut value = selection();
        let JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } = &mut value.kind
        else {
            unreachable!()
        };
        match fault {
            0 => **condition = input("left"),
            1 => when_true.ty = JavaType::primitive(JavaPrimitive::Char),
            2 => when_false.ty = JavaType::primitive(JavaPrimitive::Long),
            3 => value.ty = JavaType::primitive(JavaPrimitive::Long),
            _ => value.precedence = JavaPrecedence::Primary,
        }
        assert!(!accepted(value), "conditional fault {fault}");
    }
    for ty in [JavaPrimitive::Char, JavaPrimitive::Byte] {
        let cast = JavaExpr {
            ty: JavaType::primitive(ty),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Cast {
                target: JavaType::primitive(ty),
                value: Box::new(input("left")),
            },
        };
        assert!(!accepted(cast));
    }
    let mut value = input("left");
    for _ in 0..32 {
        value = JavaExpr {
            ty: f::int(),
            precedence: JavaPrecedence::Conditional,
            kind: JavaExprKind::Conditional {
                condition: Box::new(JavaExpr::local(f::boolean(), f::name("condition"))),
                when_true: Box::new(input("left")),
                when_false: Box::new(value),
            },
        };
    }
    assert!(accepted(value.clone()));
    for _ in 0..128 {
        value = JavaExpr {
            ty: f::int(),
            precedence: JavaPrecedence::Conditional,
            kind: JavaExprKind::Conditional {
                condition: Box::new(JavaExpr::local(f::boolean(), f::name("condition"))),
                when_true: Box::new(input("left")),
                when_false: Box::new(value),
            },
        };
    }
    assert!(!accepted(value));
    assert!(!accepted(input("missing")));
    let mut missing = function(10, "missing", &[], input("value"));
    missing.body = JavaBlock::new(vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: f::int(),
            name: f::name("value"),
            value: None,
        },
        JavaStmt::Return(Some(input("value"))),
    ]);
    assert!(!admitted(f::package(941, vec![missing])));
}
