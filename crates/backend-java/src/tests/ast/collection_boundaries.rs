//! Mutable JDK builders are local representations, not portable value boundaries.

use super::{fixture_declaration, parameter, structural_method, verify_fixture};
use crate::ast::{
    JavaBlock, JavaConstructorRef, JavaExpr, JavaExprKind, JavaField, JavaIdentifier,
    JavaKnownType, JavaMember, JavaModifier, JavaPrecedence, JavaPrimitive, JavaStmt, JavaType,
    JavaTypeUse,
};
use crate::dialect::{JavaDialect, JavaKnownConstructor};

fn list() -> JavaType {
    JavaType::generic(
        JavaKnownType::ArrayList,
        vec![JavaType::Boxed(JavaPrimitive::Int)],
    )
}

#[test]
fn mutable_collections_are_rejected_at_value_boundaries() {
    let list = list();
    let map = JavaType::generic(
        JavaKnownType::LinkedHashMap,
        vec![
            JavaType::known(JavaKnownType::String),
            JavaType::Boxed(JavaPrimitive::Int),
        ],
    );
    for ty in [
        list.clone(),
        map,
        JavaType::generic(JavaKnownType::List, vec![list]),
    ] {
        assert!(ty.verify(JavaTypeUse::Value).is_empty());
        for usage in [
            JavaTypeUse::Field,
            JavaTypeUse::Parameter,
            JavaTypeUse::Return,
        ] {
            assert!(
                !ty.verify(usage).is_empty(),
                "{ty:?} escaped through {usage:?}"
            );
        }
        let returned = JavaExpr::local(ty.clone(), JavaIdentifier::new("value").unwrap());
        for member in [
            structural_method(
                "identity",
                ty.clone(),
                vec![parameter(ty.clone(), "value")],
                JavaBlock::new(vec![JavaStmt::Return(Some(returned))]),
            ),
            JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![JavaModifier::Public],
                ty,
                name: JavaIdentifier::new("values").unwrap(),
                initializer: None,
            }),
        ] {
            assert!(
                verify_fixture(
                    portable_codegen::TargetAstBuilder::new(JavaDialect),
                    vec![(vec![], fixture_declaration(vec![member]))]
                )
                .is_err()
            );
        }
    }
}

#[test]
fn mutable_collection_cannot_escape_via_object_cast() {
    let list = list();
    let object = JavaType::known(JavaKnownType::Object);
    let fresh = JavaExpr {
        ty: list.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Known {
                constructor: JavaKnownConstructor::ArrayList,
                owner: list,
                parameters: vec![],
            },
            arguments: vec![],
        },
    };
    let erased = JavaExpr {
        ty: object.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: object.clone(),
            value: Box::new(fresh),
        },
    };
    let result = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(
            vec![],
            fixture_declaration(vec![structural_method(
                "leak",
                object,
                vec![],
                JavaBlock::new(vec![JavaStmt::Return(Some(erased))]),
            )]),
        )],
    );
    assert!(
        result.is_err(),
        "mutable collection erased through Object: {result:?}"
    );
}
