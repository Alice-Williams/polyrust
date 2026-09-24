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
        Self::build(fault, reason, false, false)
    }
    // Shared with the independently compiled canonical-owner test target.
    #[allow(dead_code)]
    pub fn type_only() -> Self {
        Self::type_only_with_order(false)
    }
    #[allow(dead_code)]
    pub fn type_only_with_order(reverse: bool) -> Self {
        Self::build(
            Fault::None,
            SynthesisReason::InterfaceAdapter,
            true,
            reverse,
        )
    }
    fn build(fault: Fault, reason: SynthesisReason, type_only: bool, reverse: bool) -> Self {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let spelling = if type_only {
            "Generated"
        } else {
            "ErrorFixture"
        };
        let root_registration = GeneratedType {
            name: spelling.into(),
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            origin: GeneratedOrigin::Synthesized(if type_only {
                SynthesisReason::PackageEntryPoint
            } else {
                SynthesisReason::InterfaceAdapter
            }),
            source: source(),
        };
        let (root, types) = if reverse {
            let error = register(&mut builder, "Error", JavaDeclarationKind::Enum);
            let success = register(&mut builder, "Success", JavaDeclarationKind::Record);
            let interface = register(
                &mut builder,
                "Outcome",
                JavaDeclarationKind::SealedInterface,
            );
            let root = builder.generated_type(root_registration);
            (
                root,
                JavaScalarResultTypes {
                    interface,
                    success,
                    error,
                },
            )
        } else {
            let root = builder.generated_type(root_registration);
            (
                root,
                JavaScalarResultTypes {
                    interface: register(
                        &mut builder,
                        "Outcome",
                        JavaDeclarationKind::SealedInterface,
                    ),
                    success: register(&mut builder, "Success", JavaDeclarationKind::Record),
                    error: register(&mut builder, "Error", JavaDeclarationKind::Enum),
                },
            )
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
        let kinds = if reverse {
            let not_a_power_of_two = constant("NOT_A_POWER_OF_TWO");
            let zero = constant("ZERO");
            let negative_overflow = constant("NEG_OVERFLOW");
            let positive_overflow = constant("POS_OVERFLOW");
            let invalid_digit = constant("INVALID_DIGIT");
            let empty = constant("EMPTY");
            JavaErrorKindValues {
                empty,
                invalid_digit,
                positive_overflow,
                negative_overflow,
                zero,
                not_a_power_of_two,
            }
        } else {
            JavaErrorKindValues {
                empty: constant("EMPTY"),
                invalid_digit: constant("INVALID_DIGIT"),
                positive_overflow: constant("POS_OVERFLOW"),
                negative_overflow: constant("NEG_OVERFLOW"),
                zero: constant("ZERO"),
                not_a_power_of_two: constant("NOT_A_POWER_OF_TWO"),
            }
        };
        if reverse {
            error.members.reverse();
        }
        let mut facade = declaration(root, spelling, JavaDeclarationKind::FinalClass);
        if type_only {
            facade
                .members
                .push(JavaMember::Constructor(JavaConstructor {
                    modifiers: vec![JavaModifier::Private],
                    name: name(spelling),
                    parameters: vec![],
                    body: JavaBlock::new(vec![]),
                }));
        } else {
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
        }
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
