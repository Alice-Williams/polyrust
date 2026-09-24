use super::expressions::*;
use crate::ast::*;
use crate::dialect::{JavaDialect, JavaScalarResultTypes};
use crate::tests::source_record_fixture::{add_file, boolean, int, name, source};
use portable_codegen::*;

pub struct Family {
    pub types: JavaScalarResultTypes,
    pub interface: JavaTypeDeclaration,
    pub success: JavaTypeDeclaration,
    pub error: JavaTypeDeclaration,
}
pub struct Fixture {
    pub builder: TargetAstBuilder<JavaDialect>,
    pub facade: JavaTypeDeclaration,
    pub family: Family,
    pub other: Family,
}
pub fn register(
    builder: &mut TargetAstBuilder<JavaDialect>,
    spelling: &str,
    kind: JavaDeclarationKind,
) -> GeneratedTypeId {
    builder.generated_type(GeneratedType {
        name: spelling.into(),
        kind,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
        source: source(),
    })
}
pub fn declaration(
    id: GeneratedTypeId,
    spelling: &str,
    kind: JavaDeclarationKind,
) -> JavaTypeDeclaration {
    JavaTypeDeclaration {
        declared: Some(id),
        kind,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: name(spelling),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![],
    }
}
impl Family {
    pub fn new(builder: &mut TargetAstBuilder<JavaDialect>, prefix: &str) -> Self {
        Self::with_interface_reason(builder, prefix, SynthesisReason::InterfaceAdapter)
    }

    pub fn with_interface_reason(
        builder: &mut TargetAstBuilder<JavaDialect>,
        prefix: &str,
        reason: SynthesisReason,
    ) -> Self {
        Self::with_interface_origin(builder, prefix, GeneratedOrigin::Synthesized(reason))
    }

    pub fn with_interface_origin(
        builder: &mut TargetAstBuilder<JavaDialect>,
        prefix: &str,
        origin: GeneratedOrigin<JavaDialect>,
    ) -> Self {
        let interface_name = format!("{prefix}Outcome");
        let success_name = format!("{prefix}Success");
        let error_name = format!("{prefix}Error");
        let types = JavaScalarResultTypes {
            interface: builder.generated_type(GeneratedType {
                name: interface_name.clone(),
                kind: JavaDeclarationKind::SealedInterface,
                visibility: JavaVisibility::Public,
                origin,
                source: source(),
            }),
            success: register(builder, &success_name, JavaDeclarationKind::Record),
            error: register(builder, &error_name, JavaDeclarationKind::Record),
        };
        let mut interface = declaration(
            types.interface,
            &interface_name,
            JavaDeclarationKind::SealedInterface,
        );
        interface.permits = vec![reference(types.success), reference(types.error)];
        let mut success = declaration(types.success, &success_name, JavaDeclarationKind::Record);
        success.heritage = JavaHeritage::Interfaces(vec![reference(types.interface)]);
        success.record_components.push(JavaRecordComponent {
            origin: JavaRecordComponentOrigin::Synthesized(payload(types.success)),
            ty: int(),
            name: name("value"),
        });
        success
            .members
            .push(JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Public],
                name: name(&success_name),
                parameters: vec![parameter(int(), "value")],
                body: JavaBlock::new(vec![JavaStmt::Assign {
                    target: JavaExpr {
                        ty: int(),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Field {
                            receiver: Box::new(JavaExpr {
                                ty: reference(types.success),
                                precedence: JavaPrecedence::Primary,
                                kind: JavaExprKind::Value(JavaValueRef::This),
                            }),
                            field: JavaFieldRef::Synthesized {
                                field: payload(types.success),
                                name: name("value"),
                                ty: int(),
                            },
                        },
                    },
                    value: local(int(), "value"),
                }]),
            }));
        let mut error = declaration(types.error, &error_name, JavaDeclarationKind::Record);
        error.heritage = JavaHeritage::Interfaces(vec![reference(types.interface)]);
        error.members.push(JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Public],
            name: name(&error_name),
            parameters: vec![],
            body: JavaBlock::new(vec![]),
        }));
        Self {
            types,
            interface,
            success,
            error,
        }
    }
    pub fn declarations(self) -> [JavaTypeDeclaration; 3] {
        [self.interface, self.success, self.error]
    }
}

impl Fixture {
    pub fn new() -> Self {
        Self::with_interface_reason(SynthesisReason::InterfaceAdapter)
    }

    pub fn with_interface_reason(reason: SynthesisReason) -> Self {
        Self::with_interface_origin(GeneratedOrigin::Synthesized(reason))
    }

    pub fn with_interface_origin(origin: GeneratedOrigin<JavaDialect>) -> Self {
        Self::with_layout(origin, JavaVisibility::Public)
    }

    pub fn with_facade_visibility(visibility: JavaVisibility) -> Self {
        Self::with_layout(
            GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
            visibility,
        )
    }

    fn with_layout(origin: GeneratedOrigin<JavaDialect>, visibility: JavaVisibility) -> Self {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let id = builder.generated_type(GeneratedType {
            name: "ResultFixture".into(),
            kind: JavaDeclarationKind::FinalClass,
            visibility,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source(),
        });
        let family = Family::with_interface_origin(&mut builder, "", origin);
        let other = Family::new(&mut builder, "Other");
        let mut fixture = Self {
            builder,
            facade: declaration(id, "ResultFixture", JavaDeclarationKind::FinalClass),
            family,
            other,
        };
        fixture.facade.visibility = visibility;
        fixture.add_methods();
        fixture
    }
    fn add_method(
        &mut self,
        spelling: &str,
        result: JavaType,
        parameters: Vec<JavaParameter>,
        statements: Vec<JavaStmt>,
    ) -> (GeneratedCallableId, JavaMethodSignature) {
        let signature = signature(
            parameters.iter().map(|param| param.ty.clone()).collect(),
            result.clone(),
        );
        let id = self.builder.callable(GeneratedCallable {
            name: spelling.into(),
            signature: JavaDialect.coarse_signature(&signature),
            visibility: JavaVisibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source(),
        });
        self.facade.members.push(JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Callable(id),
            annotations: vec![],
            modifiers: vec![JavaModifier::Public, JavaModifier::Static],
            type_parameters: vec![],
            return_type: result,
            name: name(spelling),
            parameters,
            body: Some(JavaBlock::new(statements)),
        }));
        (id, signature)
    }
    fn add_methods(&mut self) {
        let types = self.family.types;
        let result = reference(types.interface);
        let (create, create_signature) = self.add_method(
            "create",
            result.clone(),
            vec![parameter(boolean(), "success"), parameter(int(), "value")],
            vec![
                JavaStmt::If {
                    condition: local(boolean(), "success"),
                    then_block: JavaBlock::new(vec![returned(upcast(
                        new(types.success, vec![local(int(), "value")]),
                        types.success,
                        types.interface,
                    ))]),
                    else_block: None,
                },
                returned(upcast(
                    new(types.error, vec![]),
                    types.error,
                    types.interface,
                )),
            ],
        );
        let (copy, copy_signature) = self.add_method(
            "copy",
            result.clone(),
            vec![parameter(result.clone(), "input")],
            vec![returned(nonnull(local(result.clone(), "input")))],
        );
        let (observe, observe_signature) = self.add_method(
            "observe",
            int(),
            vec![
                parameter(result.clone(), "input"),
                parameter(int(), "fallback"),
            ],
            vec![
                final_local("selected", nonnull(local(result.clone(), "input"))),
                JavaStmt::If {
                    condition: instance(
                        local(result.clone(), "selected"),
                        types.success,
                        Some("success"),
                    ),
                    then_block: JavaBlock::new(vec![returned(read_payload(
                        types.success,
                        "success",
                    ))]),
                    else_block: None,
                },
                JavaStmt::If {
                    condition: instance(local(result.clone(), "selected"), types.error, None),
                    then_block: JavaBlock::new(vec![returned(local(int(), "fallback"))]),
                    else_block: None,
                },
                JavaStmt::ThrowAssertion(JavaExpr::literal(int(), JavaLiteral::I32(0))),
            ],
        );
        self.add_method(
            "entry",
            int(),
            vec![
                parameter(boolean(), "success"),
                parameter(int(), "value"),
                parameter(int(), "fallback"),
            ],
            vec![
                final_local(
                    "created",
                    call(
                        create,
                        create_signature,
                        vec![local(boolean(), "success"), local(int(), "value")],
                    ),
                ),
                final_local(
                    "copied",
                    call(copy, copy_signature, vec![local(result.clone(), "created")]),
                ),
                returned(call(
                    observe,
                    observe_signature,
                    vec![local(result, "copied"), local(int(), "fallback")],
                )),
            ],
        );
    }
    pub fn finish(mut self) -> TargetAstPackage<JavaDialect> {
        // The upcast must remain visible to javac's instanceof static analysis.
        let types = self.family.types;
        self.add_method(
            "directProbe",
            int(),
            vec![parameter(int(), "value")],
            vec![
                JavaStmt::If {
                    condition: instance(
                        upcast(
                            new(types.success, vec![local(int(), "value")]),
                            types.success,
                            types.interface,
                        ),
                        types.success,
                        Some("directPattern"),
                    ),
                    then_block: JavaBlock::new(vec![returned(read_payload(
                        types.success,
                        "directPattern",
                    ))]),
                    else_block: None,
                },
                JavaStmt::ThrowAssertion(JavaExpr::literal(int(), JavaLiteral::I32(0))),
            ],
        );
        self.facade.members.extend(
            self.family
                .declarations()
                .into_iter()
                .chain(self.other.declarations())
                .map(JavaMember::NestedType),
        );
        let mut declared = vec![GeneratedSymbolId::Type(self.facade.declared.unwrap())];
        for member in &self.facade.members {
            match member {
                JavaMember::NestedType(child) => {
                    declared.push(GeneratedSymbolId::Type(child.declared.unwrap()))
                }
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Callable(id),
                    ..
                }) => declared.push(GeneratedSymbolId::Callable(*id)),
                _ => {}
            }
        }
        add_file(&mut self.builder, "ResultFixture", declared, self.facade);
        self.builder.build()
    }
}
pub fn certify(package: TargetAstPackage<JavaDialect>) -> RenderReadyPackage<JavaDialect> {
    let verified = verify_unresolved_package(&JavaDialect, package).unwrap();
    let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
    certify_resolved_package(&JavaDialect, linked).unwrap()
}
