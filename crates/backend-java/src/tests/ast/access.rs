//! Actual declaration paths and Java top-level nest access boundaries.

use super::{fixture_declaration, parameter, structural_method, verifier_source, verify_fixture};
use crate::ast::{
    JavaBlock, JavaConstructor, JavaConstructorRef, JavaDeclarationKind, JavaExpr, JavaExprKind,
    JavaField, JavaFieldRef, JavaIdentifier, JavaLiteral, JavaMember, JavaModifier, JavaPrecedence,
    JavaPrimitive, JavaStmt, JavaType, JavaTypeName, JavaVisibility,
};
use crate::dialect::JavaDialect;
use portable_codegen::{
    GeneratedOrigin, GeneratedSymbolId, GeneratedType, SynthesisReason, TargetAstBuilder,
};

#[test]
fn private_static_members_cannot_cross_top_level_nests_in_one_file() {
    for change_call in [false, true] {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let mut items = super::synthetic_owner_fixtures::items(&mut builder);
        let symbols = items[0].declared_symbols();
        let crate::ast::JavaFileItem::Type { declaration, .. } = &mut items[1] else {
            unreachable!()
        };
        let method = declaration
            .members
            .iter_mut()
            .find_map(|member| match member {
                JavaMember::Method(method)
                    if method.name.as_str() == if change_call { "read" } else { "f" } =>
                {
                    Some(method)
                }
                _ => None,
            })
            .unwrap();
        let JavaStmt::Return(Some(value)) = &mut method.body.as_mut().unwrap().statements[0] else {
            unreachable!()
        };
        if change_call {
            let GeneratedSymbolId::Callable(replacement) = symbols[2] else {
                unreachable!()
            };
            let JavaExprKind::Call {
                callable: crate::ast::JavaCallableRef::Generated { symbol, .. },
                ..
            } = &mut value.kind
            else {
                unreachable!()
            };
            *symbol = replacement;
        } else {
            value.kind = JavaExprKind::Value(crate::ast::JavaValueRef::Generated(symbols[1]));
        }
        let result = super::verify_file_items(
            builder,
            portable_codegen::SourceRole::PublicApi,
            super::JavaFilePlacement::Main,
            items,
        );
        let diagnostics = result.unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|value| value.message.contains("outside its top-level nest")),
            "{diagnostics:?}"
        );
    }
}

#[test]
fn generated_record_component_fields_are_private_to_their_top_level_nest() {
    for same_nest in [false, true] {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let owner = builder.generated_type(GeneratedType {
            name: "Owned".to_owned(),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: verifier_source("record-owner"),
        });
        let ty = JavaType::Reference(JavaTypeName::Generated(owner));
        let int = JavaType::primitive(JavaPrimitive::Int);
        let field = super::fixture_core_field();
        let mut owned = fixture_declaration(vec![]);
        owned.declared = Some(owner);
        owned.name = JavaIdentifier::from_portable("Owned");
        owned.kind = JavaDeclarationKind::Record;
        owned
            .record_components
            .push(crate::ast::JavaRecordComponent {
                origin: crate::ast::JavaRecordComponentOrigin::Core(field),
                ty: int.clone(),
                name: JavaIdentifier::from_portable("value"),
            });
        let read = structural_method(
            "read",
            int.clone(),
            vec![parameter(ty.clone(), "input")],
            JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                ty: int.clone(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Field {
                    receiver: Box::new(JavaExpr::local(ty, JavaIdentifier::from_portable("input"))),
                    field: JavaFieldRef::Generated {
                        owner,
                        field,
                        name: JavaIdentifier::from_portable("value"),
                        ty: int,
                    },
                },
            }))]),
        );
        let mut items = Vec::new();
        if same_nest {
            owned.members.push(read);
        } else {
            items.push((vec![], fixture_declaration(vec![read])));
        }
        items.push((vec![GeneratedSymbolId::Type(owner)], owned));
        let result = verify_fixture(builder, items);
        assert_eq!(result.is_ok(), same_nest, "same={same_nest}: {result:?}");
    }
}

#[test]
fn constructor_access_requires_the_right_nest_and_static_nested_owner() {
    for same_nest in [false, true] {
        for private_type in [false, true] {
            for private_constructor in [false, true] {
                for static_nested in [false, true] {
                    let mut builder = TargetAstBuilder::new(JavaDialect);
                    let visibility = if private_type {
                        JavaVisibility::Private
                    } else {
                        JavaVisibility::Public
                    };
                    let owner = builder.generated_type(GeneratedType {
                        name: "Owned".to_owned(),
                        kind: JavaDeclarationKind::FinalClass,
                        visibility,
                        origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
                        source: verifier_source("owner"),
                    });
                    let ty = JavaType::Reference(JavaTypeName::Generated(owner));
                    let mut owned =
                        fixture_declaration(vec![JavaMember::Constructor(JavaConstructor {
                            name: JavaIdentifier::from_portable("Owned"),
                            modifiers: vec![if private_constructor {
                                JavaModifier::Private
                            } else {
                                JavaModifier::Public
                            }],
                            parameters: vec![],
                            body: JavaBlock::new(vec![]),
                        })]);
                    owned.declared = Some(owner);
                    owned.name = JavaIdentifier::from_portable("Owned");
                    owned.visibility = visibility;
                    if static_nested {
                        owned.modifiers.push(JavaModifier::Static);
                    }
                    let factory = structural_method(
                        "create",
                        ty.clone(),
                        vec![],
                        JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                            ty,
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::New {
                                constructor: JavaConstructorRef::Generated {
                                    owner,
                                    parameters: vec![],
                                },
                                arguments: vec![],
                            },
                        }))]),
                    );
                    let mut outer = fixture_declaration(vec![JavaMember::NestedType(owned)]);
                    let mut items = Vec::new();
                    if same_nest {
                        outer.members.push(factory);
                    } else {
                        let mut consumer = fixture_declaration(vec![factory]);
                        consumer.name = JavaIdentifier::from_portable("Consumer");
                        items.push((vec![], consumer));
                    }
                    items.push((vec![GeneratedSymbolId::Type(owner)], outer));
                    let result = verify_fixture(builder, items);
                    let expected =
                        static_nested && (same_nest || (!private_type && !private_constructor));
                    assert_eq!(
                        result.is_ok(),
                        expected,
                        "same={same_nest} private_type={private_type} private_ctor={private_constructor} static={static_nested}: {result:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn structural_private_fields_are_not_accessible_from_a_second_top_level_class() {
    for same_nest in [false, true] {
        for private_field in [false, true] {
            let mut builder = TargetAstBuilder::new(JavaDialect);
            let owner = builder.generated_type(GeneratedType {
                name: "Owned".to_owned(),
                kind: JavaDeclarationKind::FinalClass,
                visibility: JavaVisibility::Package,
                origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
                source: verifier_source("owner"),
            });
            let ty = JavaType::Reference(JavaTypeName::Generated(owner));
            let int = JavaType::primitive(JavaPrimitive::Int);
            let mut owned = fixture_declaration(vec![JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![if private_field {
                    JavaModifier::Private
                } else {
                    JavaModifier::Public
                }],
                ty: int.clone(),
                name: JavaIdentifier::from_portable("value"),
                initializer: Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(7))),
            })]);
            owned.declared = Some(owner);
            owned.name = JavaIdentifier::from_portable("Owned");
            let read = structural_method(
                "read",
                int.clone(),
                vec![parameter(ty.clone(), "input")],
                JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                    ty: int.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Field {
                        receiver: Box::new(JavaExpr::local(
                            ty,
                            JavaIdentifier::from_portable("input"),
                        )),
                        field: JavaFieldRef::Structural {
                            name: JavaIdentifier::from_portable("value"),
                            ty: int,
                        },
                    },
                }))]),
            );
            let mut items = Vec::new();
            if same_nest {
                owned.members.push(read);
            } else {
                items.push((vec![], fixture_declaration(vec![read])));
            }
            items.push((vec![GeneratedSymbolId::Type(owner)], owned));
            let result = verify_fixture(builder, items);
            assert_eq!(
                result.is_ok(),
                same_nest,
                "same={same_nest} private={private_field}: {result:?}"
            );
        }
    }
}
