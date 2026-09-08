//! JDK subtyping determines whether later Java type-pattern arms are reachable.

use super::{fixture_declaration, parameter, structural_method, verify_fixture};
use crate::ast::{
    JavaBlock, JavaExpr, JavaIdentifier, JavaKnownType, JavaMember, JavaPattern, JavaPrimitive,
    JavaStmt, JavaSwitchArm, JavaType,
};
use crate::dialect::JavaDialect;

fn pairs() -> Vec<(JavaType, JavaType)> {
    let generic =
        |known, arity| JavaType::generic(known, vec![JavaType::Wildcard { bound: None }; arity]);
    vec![
        (
            generic(JavaKnownType::List, 1),
            generic(JavaKnownType::ArrayList, 1),
        ),
        (
            generic(JavaKnownType::Map, 2),
            generic(JavaKnownType::LinkedHashMap, 2),
        ),
        (
            JavaType::known(JavaKnownType::RuntimeException),
            JavaType::known(JavaKnownType::IllegalArgumentException),
        ),
        (
            JavaType::known(JavaKnownType::RuntimeException),
            JavaType::known(JavaKnownType::IllegalStateException),
        ),
    ]
}

fn method(index: usize, first: JavaType, second: JavaType) -> JavaMember {
    let object = JavaType::known(JavaKnownType::Object);
    structural_method(
        &format!("knownSubtypeSwitch{index}"),
        JavaType::primitive(JavaPrimitive::Void),
        vec![parameter(object.clone(), "selector")],
        JavaBlock::new(vec![JavaStmt::Switch {
            value: JavaExpr::local(object, JavaIdentifier::new("selector").unwrap()),
            arms: vec![
                JavaSwitchArm {
                    pattern: JavaPattern::Type {
                        ty: first,
                        binding: JavaIdentifier::new("first").unwrap(),
                    },
                    body: JavaBlock::new(vec![]),
                },
                JavaSwitchArm {
                    pattern: JavaPattern::Type {
                        ty: second,
                        binding: JavaIdentifier::new("second").unwrap(),
                    },
                    body: JavaBlock::new(vec![]),
                },
                JavaSwitchArm {
                    pattern: JavaPattern::Default,
                    body: JavaBlock::new(vec![]),
                },
            ],
        }]),
    )
}

pub(super) fn legal_methods() -> Vec<JavaMember> {
    pairs()
        .into_iter()
        .enumerate()
        .map(|(index, (parent, child))| method(index, child, parent))
        .collect()
}

#[test]
fn known_subtype_patterns_reject_dominated_arms_and_accept_reverse_order() {
    for (index, (parent, child)) in pairs().into_iter().enumerate() {
        for dominated in [false, true] {
            let (first, second) = if dominated {
                (parent.clone(), child.clone())
            } else {
                (child.clone(), parent.clone())
            };
            let result = verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(
                    vec![],
                    fixture_declaration(vec![method(index, first, second)]),
                )],
            );
            assert_eq!(result.is_ok(), !dominated, "pair {index}: {result:?}");
            if let Err(errors) = result {
                assert!(
                    errors
                        .iter()
                        .any(|error| error.message.contains("dominated"))
                );
            }
        }
    }
}
