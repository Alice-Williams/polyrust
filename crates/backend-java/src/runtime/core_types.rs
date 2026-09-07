//! Typed runtime construction: core types.

use super::*;

pub(super) fn core_members() -> Vec<JavaMember> {
    let t = type_variable("T");
    let error = JavaType::known(JavaKnownType::RuntimeError);
    let result_t = generic(JavaKnownType::RuntimeResult, vec![t.clone()]);
    vec![
        JavaMember::NestedType(JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::Interface,
            visibility: JavaVisibility::Public,
            modifiers: vec![JavaModifier::Static],
            name: identifier("SemanticValue"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: [
                JavaRuntimeMember::SemanticEquals,
                JavaRuntimeMember::DeepEquals,
            ]
            .into_iter()
            .map(|member| {
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Structural,
                    annotations: vec![],
                    modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
                    type_parameters: vec![],
                    return_type: JavaType::primitive(JavaPrimitive::Boolean),
                    name: identifier(member.name()),
                    parameters: vec![parameter(JavaType::known(JavaKnownType::Object), "other")],
                    body: None,
                })
            })
            .collect(),
        }),
        JavaMember::NestedType(record(JavaKnownType::RuntimeUnit, "Unit", vec![], vec![])),
        JavaMember::NestedType(scalar_type()),
        JavaMember::NestedType(validated_error_type()),
        JavaMember::NestedType(validated_result_type()),
        package_static_method(
            vec![identifier("T")],
            result_t.clone(),
            "ok",
            vec![parameter(t.clone(), "value")],
            vec![JavaStmt::Return(Some(new_known(
                JavaKnownConstructor::RuntimeResult,
                result_t.clone(),
                vec![
                    bool_literal(true),
                    local(t.clone(), "value"),
                    null_literal(error.clone()),
                ],
            )))],
        ),
        package_static_method(
            vec![identifier("T")],
            result_t.clone(),
            "fail",
            vec![
                parameter(JavaType::known(JavaKnownType::String), "code"),
                parameter(JavaType::known(JavaKnownType::String), "message"),
            ],
            vec![JavaStmt::Return(Some(new_known(
                JavaKnownConstructor::RuntimeResult,
                result_t,
                vec![
                    bool_literal(false),
                    null_literal(t),
                    new_known(
                        JavaKnownConstructor::RuntimeError,
                        error,
                        vec![
                            local(JavaType::known(JavaKnownType::String), "code"),
                            local(JavaType::known(JavaKnownType::String), "message"),
                        ],
                    ),
                ],
            )))],
        ),
        equality_dispatch_method(
            JavaRuntimeCallable::SemanticEqual,
            JavaRuntimeMember::SemanticEquals,
            false,
        ),
        equality_dispatch_method(
            JavaRuntimeCallable::DeepEqual,
            JavaRuntimeMember::DeepEquals,
            true,
        ),
    ]
    .into_iter()
    .chain([
        runtime_method(JavaRuntimeCallable::RequireScalarString),
        runtime_method(JavaRuntimeCallable::CompareScalarStrings),
    ])
    .collect()
}

pub(super) fn scalar_type() -> JavaTypeDeclaration {
    let scalar = JavaType::known(JavaKnownType::RuntimeScalar);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let value = local(int.clone(), "value");
    let surrogate = binary(
        JavaBinaryOperator::LogicalAnd,
        binary(
            JavaBinaryOperator::GreaterEqual,
            value.clone(),
            int_literal(0xd800),
            boolean.clone(),
        ),
        binary(
            JavaBinaryOperator::LessEqual,
            value.clone(),
            int_literal(0xdfff),
            boolean.clone(),
        ),
        boolean.clone(),
    );
    let invalid = binary(
        JavaBinaryOperator::LogicalOr,
        binary(
            JavaBinaryOperator::LogicalOr,
            binary(
                JavaBinaryOperator::Less,
                value.clone(),
                int_literal(0),
                boolean.clone(),
            ),
            binary(
                JavaBinaryOperator::Greater,
                value.clone(),
                int_literal(0x10ffff),
                boolean.clone(),
            ),
            boolean.clone(),
        ),
        surrogate,
        boolean,
    );
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier("Scalar"),
        type_parameters: vec![],
        record_components: vec![component(
            int.clone(),
            "value",
            JavaRuntimeMember::ScalarValue,
        )],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Public],
            name: identifier("Scalar"),
            parameters: vec![parameter(int.clone(), "value")],
            body: JavaBlock::new(vec![
                JavaStmt::If {
                    condition: invalid,
                    then_block: JavaBlock::new(vec![JavaStmt::Throw(new_known(
                        JavaKnownConstructor::IllegalArgumentExceptionString,
                        JavaType::known(JavaKnownType::IllegalArgumentException),
                        vec![string_literal("value is not a Unicode scalar")],
                    ))]),
                    else_block: None,
                },
                JavaStmt::Assign {
                    target: structural_field(
                        JavaExpr {
                            ty: scalar,
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::Value(JavaValueRef::This),
                        },
                        "value",
                        int,
                    ),
                    value,
                },
            ]),
        })],
    }
}

pub(super) fn validated_error_type() -> JavaTypeDeclaration {
    let owner = JavaType::known(JavaKnownType::RuntimeError);
    let string = JavaType::known(JavaKnownType::String);
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier("PolyError"),
        type_parameters: vec![],
        record_components: vec![
            component(string.clone(), "code", JavaRuntimeMember::ErrorCode),
            component(string.clone(), "message", JavaRuntimeMember::ErrorMessage),
        ],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Public],
            name: identifier("PolyError"),
            parameters: vec![
                parameter(string.clone(), "code"),
                parameter(string.clone(), "message"),
            ],
            body: JavaBlock::new(vec![
                assign_component(
                    owner.clone(),
                    "code",
                    string.clone(),
                    runtime_call(
                        JavaRuntimeCallable::RequireScalarString,
                        vec![local(string.clone(), "code")],
                        string.clone(),
                    ),
                ),
                assign_component(
                    owner,
                    "message",
                    string.clone(),
                    runtime_call(
                        JavaRuntimeCallable::RequireScalarString,
                        vec![local(string.clone(), "message")],
                        string,
                    ),
                ),
            ]),
        })],
    }
}

pub(super) fn validated_result_type() -> JavaTypeDeclaration {
    let t = type_variable("T");
    let owner = generic(JavaKnownType::RuntimeResult, vec![t.clone()]);
    let error = JavaType::known(JavaKnownType::RuntimeError);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let ok = local(boolean.clone(), "ok");
    let value = local(t.clone(), "value");
    let failure = local(error.clone(), "error");
    let invalid_success = binary(
        JavaBinaryOperator::LogicalAnd,
        ok.clone(),
        binary(
            JavaBinaryOperator::LogicalOr,
            binary(
                JavaBinaryOperator::Equal,
                value.clone(),
                null_literal(t.clone()),
                boolean.clone(),
            ),
            binary(
                JavaBinaryOperator::NotEqual,
                failure.clone(),
                null_literal(error.clone()),
                boolean.clone(),
            ),
            boolean.clone(),
        ),
        boolean.clone(),
    );
    let invalid_failure = binary(
        JavaBinaryOperator::LogicalAnd,
        unary(JavaUnaryOperator::Not, ok.clone(), boolean.clone()),
        binary(
            JavaBinaryOperator::LogicalOr,
            binary(
                JavaBinaryOperator::NotEqual,
                value.clone(),
                null_literal(t.clone()),
                boolean.clone(),
            ),
            binary(
                JavaBinaryOperator::Equal,
                failure.clone(),
                null_literal(error.clone()),
                boolean.clone(),
            ),
            boolean.clone(),
        ),
        boolean.clone(),
    );
    let comparison_type = generic(
        JavaKnownType::RuntimeResult,
        vec![JavaType::Wildcard { bound: None }],
    );
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier("PolyResult"),
        type_parameters: vec![identifier("T")],
        record_components: vec![],
        heritage: JavaHeritage::Interfaces(vec![JavaType::known(
            JavaKnownType::RuntimeSemanticValue,
        )]),
        permits: vec![],
        members: vec![
            private_final_field(boolean.clone(), "ok"),
            private_final_field(t.clone(), "value"),
            private_final_field(error.clone(), "error"),
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Private],
                name: identifier("PolyResult"),
                parameters: vec![
                    parameter(boolean.clone(), "ok"),
                    parameter(t.clone(), "value"),
                    parameter(error.clone(), "error"),
                ],
                body: JavaBlock::new(vec![
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::LogicalOr,
                            invalid_success,
                            invalid_failure,
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![illegal_argument(
                            "PolyResult tag and payloads disagree",
                        )]),
                        else_block: None,
                    },
                    assign_component(owner.clone(), "ok", boolean.clone(), ok.clone()),
                    assign_component(
                        owner.clone(),
                        "value",
                        t.clone(),
                        conditional(
                            ok.clone(),
                            known_generic_call(
                                JavaKnownCallable::ObjectsRequireNonNull,
                                vec![value.clone()],
                                t.clone(),
                            ),
                            value,
                            t.clone(),
                        ),
                    ),
                    assign_component(owner.clone(), "error", error.clone(), failure),
                ]),
            }),
            field_accessor(owner.clone(), "ok", boolean, JavaRuntimeMember::ResultOk),
            guarded_accessor(
                owner.clone(),
                "value",
                t.clone(),
                unary(
                    JavaUnaryOperator::Not,
                    structural_field(
                        this_value(owner.clone()),
                        "ok",
                        JavaType::primitive(JavaPrimitive::Boolean),
                    ),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                "cannot read value from a failed PolyResult",
            ),
            guarded_accessor(
                owner.clone(),
                "error",
                error.clone(),
                structural_field(
                    this_value(generic(
                        JavaKnownType::RuntimeResult,
                        vec![type_variable("T")],
                    )),
                    "ok",
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                "cannot read error from a successful PolyResult",
            ),
            runtime_tagged_equality_method(
                owner.clone(),
                comparison_type.clone(),
                JavaRuntimeMember::ResultOk,
                (t.clone(), JavaRuntimeMember::ResultValue),
                Some((error.clone(), JavaRuntimeMember::ResultError)),
                JavaRuntimeCallable::SemanticEqual,
                JavaRuntimeMember::SemanticEquals,
            ),
            runtime_tagged_equality_method(
                owner,
                comparison_type,
                JavaRuntimeMember::ResultOk,
                (t, JavaRuntimeMember::ResultValue),
                Some((error, JavaRuntimeMember::ResultError)),
                JavaRuntimeCallable::DeepEqual,
                JavaRuntimeMember::DeepEquals,
            ),
        ],
    }
}
