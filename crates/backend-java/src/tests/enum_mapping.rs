use portable_build::CapabilityMapping;
use portable_codegen::{
    GeneratedOrigin, GeneratedType, GeneratedValue, SynthesisReason, TargetAstBuilder,
    TargetTypeRef,
};

use super::*;
use crate::ast::{JavaIdentifier, JavaKnownType, JavaLiteral, JavaPrimitive, JavaVisibility};

fn source(label: &str) -> SourceRef {
    SourceRef::logical(["java-enums-mapping-test", label])
}

fn enum_symbols(name: &str) -> (GeneratedTypeId, GeneratedValueId, GeneratedValueId) {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let enumeration = builder.generated_type(GeneratedType {
        name: name.to_owned(),
        kind: JavaDeclarationKind::Enum,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
        source: source(name),
    });
    let value_type = TargetTypeRef::Generated(enumeration);
    let first = builder.value(GeneratedValue {
        visibility: crate::ast::JavaVisibility::Public,
        name: "FIRST".to_owned(),
        ty: value_type.clone(),
        origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
        source: source("first"),
    });
    let second = builder.value(GeneratedValue {
        visibility: crate::ast::JavaVisibility::Public,
        name: "SECOND".to_owned(),
        ty: value_type,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
        source: source("second"),
    });
    (enumeration, first, second)
}

fn local(enumeration: GeneratedTypeId, name: &str) -> JavaExpr {
    JavaExpr::local(enum_type(enumeration), JavaIdentifier::from_portable(name))
}

fn arm(variant: GeneratedValueId, value: i32) -> JavaEnumBranchInput {
    JavaEnumBranchInput {
        variant,
        body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
            JavaType::primitive(JavaPrimitive::Int),
            JavaLiteral::I32(value),
        )))]),
    }
}

#[test]
fn every_enum_mapping_operation_constructs_typed_java_ast() {
    let (enumeration, first, second) = enum_symbols("Choice");
    let mapping = JavaEnums;

    let declaration = mapping
        .lower(
            &mut (),
            JavaEnumsInput::Declaration {
                declared: enumeration,
                visibility: Visibility::Public,
                name: "Choice".to_owned(),
                variants: vec![
                    JavaEnumVariantInput {
                        declared: first,
                        name: "FIRST".to_owned(),
                    },
                    JavaEnumVariantInput {
                        declared: second,
                        name: "SECOND".to_owned(),
                    },
                ],
            },
        )
        .expect("enum declaration maps");
    assert!(matches!(
        declaration,
        JavaEnumsNode::Declaration(value)
            if value.len() == 1 && value[0].kind == JavaDeclarationKind::Enum && value[0].members.len() == 2
    ));

    let variant = mapping
        .lower(
            &mut (),
            JavaEnumsInput::Variant {
                enumeration,
                variant: first,
            },
        )
        .expect("enum variant maps");
    assert!(matches!(variant, JavaEnumsNode::Expression(_)));

    let equality = mapping
        .lower(
            &mut (),
            JavaEnumsInput::Equality {
                operator: JavaEnumEqualityOperator::Equal,
                enumeration,
                left: Box::new(local(enumeration, "left")),
                right: Box::new(local(enumeration, "right")),
            },
        )
        .expect("enum equality maps");
    assert!(matches!(equality, JavaEnumsNode::Expression(_)));

    let branch = mapping
        .lower(
            &mut (),
            JavaEnumsInput::Branch {
                selector: Box::new(local(enumeration, "value")),
                enumeration,
                declared_variants: vec![first, second],
                arms: vec![arm(first, 1), arm(second, 2)],
            },
        )
        .expect("exhaustive enum branch maps");
    assert!(matches!(
        branch,
        JavaEnumsNode::Statement(value)
            if matches!(*value, JavaStmt::Switch { ref arms, .. } if arms.len() == 3)
    ));
}

#[test]
fn enum_inequality_is_owned_by_the_enum_mapping() {
    let (enumeration, _, _) = enum_symbols("Choice");
    let result = JavaEnums
        .lower(
            &mut (),
            JavaEnumsInput::Equality {
                operator: JavaEnumEqualityOperator::NotEqual,
                enumeration,
                left: Box::new(local(enumeration, "left")),
                right: Box::new(local(enumeration, "right")),
            },
        )
        .expect("enum inequality maps");
    assert!(matches!(result, JavaEnumsNode::Expression(value)
    if matches!(value.kind, JavaExprKind::Binary {
        operator: JavaBinaryOperator::NotEqual, ..
    })));
}

#[test]
fn enum_mapping_rejects_wrong_types_and_non_exhaustive_branches() {
    let (enumeration, first, second) = enum_symbols("Choice");
    let mapping = JavaEnums;

    let equality = mapping.lower(
        &mut (),
        JavaEnumsInput::Equality {
            operator: JavaEnumEqualityOperator::Equal,
            enumeration,
            left: Box::new(local(enumeration, "left")),
            right: Box::new(JavaExpr::local(
                JavaType::known(JavaKnownType::String),
                JavaIdentifier::from_portable("right"),
            )),
        },
    );
    assert!(equality.is_err());

    let missing = mapping.lower(
        &mut (),
        JavaEnumsInput::Branch {
            selector: Box::new(local(enumeration, "value")),
            enumeration,
            declared_variants: vec![first, second],
            arms: vec![arm(first, 1)],
        },
    );
    assert!(missing.is_err());

    let duplicate = mapping.lower(
        &mut (),
        JavaEnumsInput::Branch {
            selector: Box::new(local(enumeration, "value")),
            enumeration,
            declared_variants: vec![first, second],
            arms: vec![arm(first, 1), arm(first, 2)],
        },
    );
    assert!(duplicate.is_err());
}
