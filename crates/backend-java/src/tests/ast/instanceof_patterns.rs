//! Java 21 admits both conditional and unconditional reference patterns.

use super::{fixture_declaration, parameter, structural_method, verify_fixture};
use crate::ast::{
    JavaArrayOwnership, JavaArrayOwnershipTransition, JavaBlock, JavaExpr, JavaExprKind,
    JavaIdentifier, JavaKnownType, JavaLiteral, JavaMember, JavaPrecedence, JavaPrimitive,
    JavaStmt, JavaType,
};

fn method(source: JavaType, target: JavaType, binding: bool) -> JavaMember {
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let result = |value| {
        JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
            boolean.clone(),
            JavaLiteral::Boolean(value),
        )))])
    };
    structural_method(
        "patternCheck",
        boolean.clone(),
        vec![parameter(source.clone(), "value")],
        JavaBlock::new(vec![JavaStmt::If {
            condition: JavaExpr {
                ty: boolean.clone(),
                precedence: JavaPrecedence::Relational,
                kind: JavaExprKind::InstanceOf {
                    value: Box::new(JavaExpr::local(
                        source,
                        JavaIdentifier::new("value").unwrap(),
                    )),
                    target,
                    binding: binding.then(|| JavaIdentifier::new("matched").unwrap()),
                },
            },
            then_block: result(true),
            else_block: Some(result(false)),
        }]),
    )
}

pub(super) fn legal_methods() -> Vec<JavaMember> {
    let string = JavaType::known(JavaKnownType::String);
    let object = JavaType::known(JavaKnownType::Object);
    let mut methods = [
        (string.clone(), object.clone(), true),
        (string.clone(), string.clone(), true),
        (string.clone(), object.clone(), false),
        (object.clone(), string, true),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (source, target, binds))| {
        let mut member = method(source, target, binds);
        let JavaMember::Method(value) = &mut member else {
            unreachable!()
        };
        value.name = JavaIdentifier::new(format!("instanceOfControl{index}")).unwrap();
        member
    })
    .collect::<Vec<_>>();
    let byte = JavaType::primitive(JavaPrimitive::Byte);
    let array = JavaExpr {
        ty: JavaType::Array {
            component: Box::new(byte.clone()),
            ownership: JavaArrayOwnership::InternalMutable,
        },
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::NewArray {
            component: byte.clone(),
            length: Box::new(JavaExpr::literal(
                JavaType::primitive(JavaPrimitive::Int),
                JavaLiteral::I32(1),
            )),
        },
    };
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    methods.push(structural_method(
        "unboundArrayTypeCheck",
        boolean.clone(),
        vec![],
        JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: boolean,
            precedence: JavaPrecedence::Relational,
            kind: JavaExprKind::InstanceOf {
                value: Box::new(array.clone()),
                target: object.clone(),
                binding: None,
            },
        }))]),
    ));
    let copied = JavaExpr {
        ty: JavaType::Array {
            component: Box::new(byte),
            ownership: JavaArrayOwnership::DefensiveCopyBoundary,
        },
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::ArrayOwnershipTransition {
            transition: JavaArrayOwnershipTransition::FreshCopyToBoundary,
            value: Box::new(array),
        },
    };
    methods.push(structural_method(
        "copiedArrayAsObject",
        object.clone(),
        vec![],
        JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: object.clone(),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Cast {
                target: object,
                value: Box::new(copied),
            },
        }))]),
    ));
    methods
}

#[test]
fn instanceof_patterns_admit_java21_unconditional_bindings() {
    let string = JavaType::known(JavaKnownType::String);
    let object = JavaType::known(JavaKnownType::Object);
    for (source, target, binding, expected) in [
        (string.clone(), object.clone(), true, true),
        (string.clone(), string.clone(), true, true),
        (string.clone(), object.clone(), false, true),
        (string.clone(), string.clone(), false, true),
        (object, string, true, true),
    ] {
        let result = verify_fixture(
            portable_codegen::TargetAstBuilder::new(crate::dialect::JavaDialect),
            vec![(
                vec![],
                fixture_declaration(vec![method(source, target, binding)]),
            )],
        );
        assert_eq!(result.is_ok(), expected, "{result:?}");
    }
}
