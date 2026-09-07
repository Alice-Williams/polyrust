//! Native fixtures whose private names collide across different Java nests.

use super::{fixture_declaration, structural_method, verifier_source};
use crate::ast::{
    JavaBlock, JavaCallableRef, JavaConstructor, JavaConstructorRef, JavaDeclarationKind, JavaExpr,
    JavaExprKind, JavaField, JavaFileItem, JavaIdentifier, JavaLiteral, JavaMember,
    JavaMethodDeclaration, JavaMethodSignature, JavaModifier, JavaPrecedence, JavaPrimitive,
    JavaStmt, JavaType, JavaTypeName, JavaValueRef, JavaVisibility,
};
use crate::dialect::JavaDialect;
use portable_codegen::{
    GeneratedCallable, GeneratedOrigin, GeneratedSymbolId, GeneratedType, GeneratedValue,
    SynthesisReason, TargetAstBuilder,
};

pub(super) fn items(builder: &mut TargetAstBuilder<JavaDialect>) -> Vec<JavaFileItem> {
    ["SyntheticA", "SyntheticB"]
        .into_iter()
        .map(|name| {
            let origin = GeneratedOrigin::Synthesized(SynthesisReason::TestHarness);
            let owner = builder.generated_type(GeneratedType {
                name: "Foo".to_owned(),
                kind: JavaDeclarationKind::FinalClass,
                visibility: JavaVisibility::Private,
                origin: origin.clone(),
                source: verifier_source("private-foo"),
            });
            let int = JavaType::primitive(JavaPrimitive::Int);
            let constant = builder.value(GeneratedValue {
                name: "C".to_owned(),
                ty: JavaDialect.registered_type(&int),
                visibility: JavaVisibility::Private,
                origin: origin.clone(),
                source: verifier_source("private-constant"),
            });
            let signature = JavaMethodSignature {
                receiver: None,
                parameters: vec![],
                result: int.clone(),
                checked_exceptions: vec![],
                nullable_result: false,
                pure: true,
            };
            let function = builder.callable(GeneratedCallable {
                name: "f".to_owned(),
                signature: JavaDialect.coarse_signature(&signature),
                visibility: JavaVisibility::Private,
                origin,
                source: verifier_source("private-function"),
            });
            let mut function_node = structural_method(
                "f",
                int.clone(),
                vec![],
                JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                    ty: int.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(
                        constant,
                    ))),
                }))]),
            );
            if let JavaMember::Method(method) = &mut function_node {
                method.declared = JavaMethodDeclaration::Callable(function);
                method.modifiers = vec![JavaModifier::Private, JavaModifier::Static];
            }
            let mut foo = fixture_declaration(vec![JavaMember::Constructor(JavaConstructor {
                name: JavaIdentifier::from_portable("Foo"),
                modifiers: vec![JavaModifier::Private],
                parameters: vec![],
                body: JavaBlock::new(vec![]),
            })]);
            foo.name = JavaIdentifier::from_portable("Foo");
            foo.declared = Some(owner);
            foo.visibility = JavaVisibility::Private;
            foo.modifiers = vec![JavaModifier::Static];
            let ty = JavaType::Reference(JavaTypeName::Generated(owner));
            let mut root = fixture_declaration(vec![
                JavaMember::NestedType(foo),
                JavaMember::Field(JavaField {
                    declared: Some(constant),
                    name: JavaIdentifier::from_portable("C"),
                    ty: int.clone(),
                    modifiers: vec![
                        JavaModifier::Private,
                        JavaModifier::Static,
                        JavaModifier::Final,
                    ],
                    initializer: Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(7))),
                }),
                function_node,
                structural_method(
                    "read",
                    int.clone(),
                    vec![],
                    JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                        ty: int,
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Call {
                            callable: JavaCallableRef::Generated {
                                symbol: function,
                                signature,
                            },
                            receiver: None,
                            arguments: vec![],
                        },
                    }))]),
                ),
                structural_method(
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
                ),
            ]);
            root.name = JavaIdentifier::from_portable(name);
            JavaFileItem::Type {
                conformances: crate::ast::JavaConformanceInventory::structural().into(),
                declared: vec![
                    GeneratedSymbolId::Type(owner),
                    GeneratedSymbolId::Value(constant),
                    GeneratedSymbolId::Callable(function),
                ],
                declaration: root,
            }
        })
        .collect()
}
