//! The generator-side typed fixture is independent of the native expected values.
use super::model::*;
use portable_backend_java::{ast::*, dialect::*};
use portable_codegen::*;

#[derive(Clone, Copy)]
pub enum Fault {
    None,
    WrongKind,
    ZeroSuccess,
}

pub struct Fixture {
    pub builder: TargetAstBuilder<JavaDialect>,
    pub facade: JavaTypeDeclaration,
    pub interface: JavaTypeDeclaration,
    pub success: JavaTypeDeclaration,
    pub error: JavaTypeDeclaration,
    pub types: JavaScalarResultTypes,
    pub kinds: JavaErrorKindValues,
}
impl Fixture {
    pub fn new(fault: Fault) -> Self {
        Self::with_value_origin(fault, SynthesisReason::InterfaceAdapter)
    }
    pub fn with_value_origin(fault: Fault, reason: SynthesisReason) -> Self {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let root = register(
            &mut builder,
            "ErrorFixture",
            JavaDeclarationKind::FinalClass,
        );
        let types = JavaScalarResultTypes {
            interface: register(
                &mut builder,
                "Outcome",
                JavaDeclarationKind::SealedInterface,
            ),
            success: register(&mut builder, "Success", JavaDeclarationKind::Record),
            error: register(&mut builder, "Error", JavaDeclarationKind::Enum),
        };
        let mut interface = declaration(
            types.interface,
            "Outcome",
            JavaDeclarationKind::SealedInterface,
        );
        interface.permits = vec![reference(types.success), reference(types.error)];
        let mut success = declaration(types.success, "Success", JavaDeclarationKind::Record);
        success.heritage = JavaHeritage::Interfaces(vec![reference(types.interface)]);
        let field = JavaSynthesizedField {
            owner: types.success,
            role: JavaSynthesizedFieldRole::ScalarResultPayload,
        };
        success.record_components = vec![JavaRecordComponent {
            origin: JavaRecordComponentOrigin::Synthesized(field),
            ty: int(),
            name: name("value"),
        }];
        success.members = vec![JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Public],
            name: name("Success"),
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
                            field,
                            name: name("value"),
                            ty: int(),
                        },
                    },
                },
                value: JavaExpr::local(int(), name("value")),
            }]),
        })];
        let mut error = declaration(types.error, "Error", JavaDeclarationKind::Enum);
        error.heritage = JavaHeritage::Interfaces(vec![reference(types.interface)]);
        let mut constant = |spelling: &str| {
            let id = builder.value(GeneratedValue {
                name: spelling.into(),
                ty: TargetTypeRef::Generated(types.error),
                visibility: JavaVisibility::Public,
                origin: GeneratedOrigin::Synthesized(reason),
                source: source(),
            });
            error
                .members
                .push(JavaMember::EnumConstant(JavaEnumConstant {
                    declared: id,
                    name: name(spelling),
                }));
            id
        };
        let kinds = JavaErrorKindValues {
            empty: constant("EMPTY"),
            invalid_digit: constant("INVALID_DIGIT"),
            positive_overflow: constant("POS_OVERFLOW"),
            negative_overflow: constant("NEG_OVERFLOW"),
            zero: constant("ZERO"),
            not_a_power_of_two: constant("NOT_A_POWER_OF_TWO"),
        };
        let mut facade = declaration(root, "ErrorFixture", JavaDeclarationKind::FinalClass);
        for kind in RustIntegerErrorKind::ALL {
            let selected = if matches!(fault, Fault::WrongKind)
                && kind == RustIntegerErrorKind::NotAPowerOfTwo
            {
                kinds.empty
            } else {
                kinds.value(kind)
            };
            let value = JavaExpr {
                ty: reference(types.error),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::EnumVariant {
                    enumeration: types.error,
                    variant: selected,
                }),
            };
            facade.members.push(method(
                &mut builder,
                &format!("error{}", kind.transport_code()),
                vec![],
                upcast(value, types.error, types.interface),
            ));
        }
        let success_value = if matches!(fault, Fault::ZeroSuccess) {
            JavaExpr::literal(int(), JavaLiteral::I32(0))
        } else {
            JavaExpr::local(int(), name("value"))
        };
        facade.members.push(method(
            &mut builder,
            "success",
            vec![parameter(int(), "value")],
            upcast(
                JavaExpr {
                    ty: reference(types.success),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::New {
                        constructor: JavaConstructorRef::Generated {
                            owner: types.success,
                            parameters: vec![int()],
                        },
                        arguments: vec![success_value],
                    },
                },
                types.success,
                types.interface,
            ),
        ));
        facade.members.push(method(
            &mut builder,
            "forward",
            vec![parameter(reference(types.interface), "input")],
            nonnull(JavaExpr::local(reference(types.interface), name("input"))),
        ));
        Self {
            builder,
            facade,
            interface,
            success,
            error,
            types,
            kinds,
        }
    }
    pub fn finish(mut self) -> TargetAstPackage<JavaDialect> {
        self.facade.members.extend(
            [self.interface, self.success, self.error]
                .into_iter()
                .map(JavaMember::NestedType),
        );
        finish(self.builder, self.facade)
    }
}

pub fn checked(fault: Fault) -> JavaErrorResultFamily {
    let fixture = Fixture::new(fault);
    let (types, kinds) = (fixture.types, fixture.kinds);
    JavaErrorResultFamily::from_certificate(certify(fixture.finish()).unwrap(), types, kinds)
        .unwrap()
}
