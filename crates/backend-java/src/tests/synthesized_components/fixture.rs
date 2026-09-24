use crate::ast::*;
use crate::dialect::JavaDialect;
use crate::tests::source_record_fixture::{add_file, int, name, source};
use portable_codegen::*;

pub struct Fixture {
    pub builder: TargetAstBuilder<JavaDialect>,
    pub facade: JavaTypeDeclaration,
    pub record: JavaTypeDeclaration,
    pub other: JavaTypeDeclaration,
    pub owner: GeneratedTypeId,
    pub other_id: GeneratedTypeId,
    pub facade_id: GeneratedTypeId,
}

pub fn field_id(owner: GeneratedTypeId) -> JavaSynthesizedField {
    JavaSynthesizedField {
        owner,
        role: JavaSynthesizedFieldRole::ScalarResultPayload,
    }
}
pub fn ty(owner: GeneratedTypeId) -> JavaType {
    JavaType::Reference(JavaTypeName::Generated(owner))
}
pub fn read(receiver: JavaExpr, owner: GeneratedTypeId) -> JavaExpr {
    JavaExpr {
        ty: int(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Field {
            receiver: Box::new(receiver),
            field: JavaFieldRef::Synthesized {
                field: field_id(owner),
                name: name("value"),
                ty: int(),
            },
        },
    }
}
pub fn accessor(receiver: JavaExpr, owner: GeneratedTypeId) -> JavaExpr {
    JavaExpr {
        ty: int(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: ty(owner),
                name: name("value"),
                signature: Box::new(JavaMethodSignature {
                    receiver: Some(ty(owner)),
                    parameters: vec![],
                    result: int(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                }),
                origin: JavaMemberOrigin::SynthesizedField(field_id(owner)),
            },
            receiver: Some(Box::new(receiver)),
            arguments: vec![],
        },
    }
}
pub fn method(
    spelling: &str,
    result: JavaType,
    parameters: Vec<JavaParameter>,
    value: JavaExpr,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters: vec![],
        return_type: result,
        name: name(spelling),
        parameters,
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(value))])),
    })
}
pub fn parameter(spelling: &str, ty: JavaType) -> JavaParameter {
    JavaParameter {
        ty,
        name: name(spelling),
        final_parameter: true,
    }
}
fn declaration(
    id: GeneratedTypeId,
    spelling: &str,
    kind: JavaDeclarationKind,
    visibility: JavaVisibility,
) -> JavaTypeDeclaration {
    JavaTypeDeclaration {
        declared: Some(id),
        kind,
        visibility,
        modifiers: vec![],
        name: name(spelling),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![],
    }
}
pub(super) fn register(
    builder: &mut TargetAstBuilder<JavaDialect>,
    spelling: &str,
    kind: JavaDeclarationKind,
    visibility: JavaVisibility,
    reason: SynthesisReason,
) -> GeneratedTypeId {
    builder.generated_type(GeneratedType {
        name: spelling.into(),
        kind,
        visibility,
        origin: GeneratedOrigin::Synthesized(reason),
        source: source(),
    })
}
pub(super) fn build_record(
    owner: GeneratedTypeId,
    spelling: &str,
    visibility: JavaVisibility,
) -> JavaTypeDeclaration {
    let mut declaration = declaration(owner, spelling, JavaDeclarationKind::Record, visibility);
    declaration.record_components.push(JavaRecordComponent {
        origin: JavaRecordComponentOrigin::Synthesized(field_id(owner)),
        ty: int(),
        name: name("value"),
    });
    declaration
        .members
        .push(JavaMember::Constructor(JavaConstructor {
            modifiers: vec![if visibility == JavaVisibility::Public {
                JavaModifier::Public
            } else {
                JavaModifier::Private
            }],
            name: name(spelling),
            parameters: vec![parameter("value", int())],
            body: JavaBlock::new(vec![JavaStmt::Assign {
                target: read(
                    JavaExpr {
                        ty: ty(owner),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Value(JavaValueRef::This),
                    },
                    owner,
                ),
                value: JavaExpr::local(int(), name("value")),
            }]),
        }));
    declaration
}
impl Fixture {
    pub fn new() -> Self {
        Self::with_visibility(JavaVisibility::Public)
    }
    pub fn with_visibility(visibility: JavaVisibility) -> Self {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let facade_id = register(
            &mut builder,
            "SynthesizedFixture",
            JavaDeclarationKind::FinalClass,
            JavaVisibility::Public,
            SynthesisReason::TestHarness,
        );
        let owner = register(
            &mut builder,
            "Payload",
            JavaDeclarationKind::Record,
            visibility,
            SynthesisReason::InterfaceAdapter,
        );
        let other_id = register(
            &mut builder,
            "Other",
            JavaDeclarationKind::Record,
            visibility,
            SynthesisReason::InterfaceAdapter,
        );
        let record = build_record(owner, "Payload", visibility);
        let other = build_record(other_id, "Other", visibility);
        let mut facade = declaration(
            facade_id,
            "SynthesizedFixture",
            JavaDeclarationKind::FinalClass,
            JavaVisibility::Public,
        );
        let value = || JavaExpr::local(ty(owner), name("item"));
        facade.members = vec![
            method(
                "create",
                ty(owner),
                vec![parameter("value", int())],
                JavaExpr {
                    ty: ty(owner),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::New {
                        constructor: JavaConstructorRef::Generated {
                            owner,
                            parameters: vec![int()],
                        },
                        arguments: vec![JavaExpr::local(int(), name("value"))],
                    },
                },
            ),
            method(
                "copy",
                ty(owner),
                vec![parameter("item", ty(owner))],
                value(),
            ),
            method(
                "direct",
                int(),
                vec![parameter("item", ty(owner))],
                read(value(), owner),
            ),
            method(
                "accessor",
                int(),
                vec![parameter("item", ty(owner))],
                accessor(value(), owner),
            ),
        ];
        Self {
            builder,
            facade,
            record,
            other,
            owner,
            other_id,
            facade_id,
        }
    }
    pub fn constructor(&mut self) -> &mut JavaConstructor {
        let JavaMember::Constructor(value) = &mut self.record.members[0] else {
            unreachable!()
        };
        value
    }
    pub fn finish(mut self) -> TargetAstPackage<JavaDialect> {
        self.facade.members.extend([
            JavaMember::NestedType(self.record),
            JavaMember::NestedType(self.other),
        ]);
        let declared = vec![self.facade_id, self.owner, self.other_id]
            .into_iter()
            .map(GeneratedSymbolId::Type)
            .collect();
        add_file(
            &mut self.builder,
            "SynthesizedFixture",
            declared,
            self.facade,
        );
        self.builder.build()
    }
}
pub fn certify(package: TargetAstPackage<JavaDialect>) -> RenderReadyPackage<JavaDialect> {
    let verified = verify_unresolved_package(&JavaDialect, package).unwrap();
    let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
    certify_resolved_package(&JavaDialect, linked).unwrap()
}
