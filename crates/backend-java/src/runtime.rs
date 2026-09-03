use crate::{ast::*, dialect::*};

pub fn helper_items(helper: JavaRuntimeHelper) -> Vec<JavaFileItem> {
    let members = match helper {
        JavaRuntimeHelper::Core => core_members(),
        JavaRuntimeHelper::TaggedValues => tagged_members(),
        JavaRuntimeHelper::CheckedIntegers => runtime_methods(helper),
        JavaRuntimeHelper::FloatBits => runtime_methods(helper),
        JavaRuntimeHelper::Unicode => runtime_methods(helper),
        JavaRuntimeHelper::Bytes => bytes_members(),
        JavaRuntimeHelper::ImmutableLists => runtime_methods(helper),
        JavaRuntimeHelper::StringOperations => runtime_methods(helper),
        JavaRuntimeHelper::Interfaces => vec![static_method(
            vec![],
            JavaType::primitive(JavaPrimitive::Boolean),
            "interfaceSupport",
            vec![],
            vec![JavaStmt::Return(Some(bool_literal(true)))],
        )],
    };
    vec![JavaFileItem::RuntimeMembers { helper, members }]
}

fn core_members() -> Vec<JavaMember> {
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
            members: vec![JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Structural,
                annotations: vec![],
                modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
                type_parameters: vec![],
                return_type: JavaType::primitive(JavaPrimitive::Boolean),
                name: identifier("semanticEquals"),
                parameters: vec![parameter(JavaType::known(JavaKnownType::Object), "other")],
                body: None,
            })],
        }),
        JavaMember::NestedType(record(JavaKnownType::RuntimeUnit, "Unit", vec![], vec![])),
        JavaMember::NestedType(scalar_type()),
        JavaMember::NestedType(validated_error_type()),
        JavaMember::NestedType(validated_result_type()),
        static_method(
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
        static_method(
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
        semantic_equality_method(),
        static_method(
            vec![],
            JavaType::primitive(JavaPrimitive::Boolean),
            "deepEqual",
            vec![
                parameter(JavaType::known(JavaKnownType::Object), "left"),
                parameter(JavaType::known(JavaKnownType::Object), "right"),
            ],
            vec![JavaStmt::Return(Some(known_call(
                JavaKnownCallable::ObjectsDeepEquals,
                vec![
                    local(JavaType::known(JavaKnownType::Object), "left"),
                    local(JavaType::known(JavaKnownType::Object), "right"),
                ],
            )))],
        ),
    ]
    .into_iter()
    .chain([
        runtime_method(JavaRuntimeCallable::RequireScalarString),
        runtime_method(JavaRuntimeCallable::CompareScalarStrings),
    ])
    .collect()
}

fn scalar_type() -> JavaTypeDeclaration {
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

fn validated_error_type() -> JavaTypeDeclaration {
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

fn validated_result_type() -> JavaTypeDeclaration {
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
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier("PolyResult"),
        type_parameters: vec![identifier("T")],
        record_components: vec![
            component(boolean.clone(), "ok", JavaRuntimeMember::ResultOk),
            component(t.clone(), "value", JavaRuntimeMember::ResultValue),
            component(error.clone(), "error", JavaRuntimeMember::ResultError),
        ],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Public],
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
                    assign_component(owner.clone(), "ok", boolean, ok),
                    assign_component(owner.clone(), "value", t.clone(), value),
                    assign_component(owner.clone(), "error", error.clone(), failure),
                ]),
            }),
            guarded_accessor(
                owner.clone(),
                "value",
                t,
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
                owner,
                "error",
                error,
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
        ],
    }
}

fn semantic_equality_method() -> JavaMember {
    let object = JavaType::known(JavaKnownType::Object);
    let double = JavaType::Boxed(JavaPrimitive::Double);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let wildcard = JavaType::Wildcard { bound: None };
    let list = generic(JavaKnownType::List, vec![wildcard.clone()]);
    let semantic = JavaType::known(JavaKnownType::RuntimeSemanticValue);
    let left_list = local(list.clone(), "leftList");
    let right_list = local(list.clone(), "rightList");
    let index = local(int.clone(), "index");
    let size = known_method_call(
        JavaKnownMethod::ListSize,
        left_list.clone(),
        vec![],
        int.clone(),
    );
    static_method(
        vec![],
        boolean.clone(),
        JavaRuntimeCallable::SemanticEqual.name(),
        vec![
            parameter(object.clone(), "left"),
            parameter(object.clone(), "right"),
        ],
        vec![
            JavaStmt::If {
                condition: binary(
                    JavaBinaryOperator::LogicalAnd,
                    instance_of(
                        local(object.clone(), "left"),
                        double.clone(),
                        Some(identifier("leftDouble")),
                    ),
                    instance_of(
                        local(object.clone(), "right"),
                        double.clone(),
                        Some(identifier("rightDouble")),
                    ),
                    boolean.clone(),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(binary(
                    JavaBinaryOperator::Equal,
                    cast(
                        JavaType::primitive(JavaPrimitive::Double),
                        local(double.clone(), "leftDouble"),
                    ),
                    cast(
                        JavaType::primitive(JavaPrimitive::Double),
                        local(double, "rightDouble"),
                    ),
                    boolean.clone(),
                )))]),
                else_block: None,
            },
            JavaStmt::If {
                condition: binary(
                    JavaBinaryOperator::LogicalAnd,
                    instance_of(
                        local(object.clone(), "left"),
                        list.clone(),
                        Some(identifier("leftList")),
                    ),
                    instance_of(
                        local(object.clone(), "right"),
                        list.clone(),
                        Some(identifier("rightList")),
                    ),
                    boolean.clone(),
                ),
                then_block: JavaBlock::new(vec![
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::NotEqual,
                            size.clone(),
                            known_method_call(
                                JavaKnownMethod::ListSize,
                                right_list.clone(),
                                vec![],
                                int.clone(),
                            ),
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(bool_literal(
                            false,
                        )))]),
                        else_block: None,
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Mutable,
                        ty: int.clone(),
                        name: identifier("index"),
                        value: Some(int_literal(0)),
                    },
                    JavaStmt::While {
                        condition: binary(
                            JavaBinaryOperator::Less,
                            index.clone(),
                            size,
                            boolean.clone(),
                        ),
                        body: JavaBlock::new(vec![
                            JavaStmt::If {
                                condition: unary(
                                    JavaUnaryOperator::Not,
                                    runtime_call(
                                        JavaRuntimeCallable::SemanticEqual,
                                        vec![
                                            known_method_call(
                                                JavaKnownMethod::ListGet,
                                                left_list.clone(),
                                                vec![index.clone()],
                                                object.clone(),
                                            ),
                                            known_method_call(
                                                JavaKnownMethod::ListGet,
                                                right_list,
                                                vec![index.clone()],
                                                object.clone(),
                                            ),
                                        ],
                                        boolean.clone(),
                                    ),
                                    boolean.clone(),
                                ),
                                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                                    bool_literal(false),
                                ))]),
                                else_block: None,
                            },
                            JavaStmt::Assign {
                                target: index.clone(),
                                value: binary(JavaBinaryOperator::Add, index, int_literal(1), int),
                            },
                        ]),
                    },
                    JavaStmt::Return(Some(bool_literal(true))),
                ]),
                else_block: None,
            },
            JavaStmt::If {
                condition: instance_of(
                    local(object.clone(), "left"),
                    semantic.clone(),
                    Some(identifier("semanticValue")),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(member_call(
                    local(semantic, "semanticValue"),
                    JavaRuntimeMember::SemanticEquals,
                    vec![local(object.clone(), "right")],
                    boolean,
                )))]),
                else_block: None,
            },
            JavaStmt::Return(Some(known_call(
                JavaKnownCallable::ObjectsDeepEquals,
                vec![local(object.clone(), "left"), local(object, "right")],
            ))),
        ],
    )
}

fn tagged_members() -> Vec<JavaMember> {
    let t = type_variable("T");
    let e = type_variable("E");
    let option_t = generic(JavaKnownType::RuntimeOption, vec![t.clone()]);
    let value_result = generic(
        JavaKnownType::RuntimeValueResult,
        vec![t.clone(), e.clone()],
    );
    vec![
        JavaMember::NestedType(validated_option_type()),
        JavaMember::NestedType(validated_value_result_type()),
        static_method(
            vec![identifier("T")],
            option_t.clone(),
            "optionNone",
            vec![],
            vec![JavaStmt::Return(Some(new_known(
                JavaKnownConstructor::RuntimeOption,
                option_t.clone(),
                vec![bool_literal(false), null_literal(t.clone())],
            )))],
        ),
        static_method(
            vec![identifier("T")],
            option_t.clone(),
            "optionSome",
            vec![parameter(t.clone(), "value")],
            vec![JavaStmt::Return(Some(new_known(
                JavaKnownConstructor::RuntimeOption,
                option_t.clone(),
                vec![bool_literal(true), local(t.clone(), "value")],
            )))],
        ),
        static_method(
            vec![identifier("T")],
            JavaType::primitive(JavaPrimitive::Boolean),
            "optionIsSome",
            vec![parameter(option_t.clone(), "value")],
            vec![JavaStmt::Return(Some(member_call(
                local(option_t.clone(), "value"),
                JavaRuntimeMember::OptionSome,
                vec![],
                JavaType::primitive(JavaPrimitive::Boolean),
            )))],
        ),
        static_method(
            vec![identifier("T")],
            t.clone(),
            "optionValue",
            vec![parameter(option_t.clone(), "value")],
            vec![JavaStmt::Return(Some(member_call(
                local(option_t, "value"),
                JavaRuntimeMember::OptionValue,
                vec![],
                t.clone(),
            )))],
        ),
        static_method(
            vec![identifier("T"), identifier("E")],
            value_result.clone(),
            "valueResultOk",
            vec![parameter(t.clone(), "value")],
            vec![JavaStmt::Return(Some(new_known(
                JavaKnownConstructor::RuntimeValueResult,
                value_result.clone(),
                vec![
                    bool_literal(true),
                    local(t.clone(), "value"),
                    null_literal(e.clone()),
                ],
            )))],
        ),
        static_method(
            vec![identifier("T"), identifier("E")],
            value_result.clone(),
            "valueResultErr",
            vec![parameter(e.clone(), "error")],
            vec![JavaStmt::Return(Some(new_known(
                JavaKnownConstructor::RuntimeValueResult,
                value_result.clone(),
                vec![
                    bool_literal(false),
                    null_literal(t.clone()),
                    local(e.clone(), "error"),
                ],
            )))],
        ),
        static_method(
            vec![identifier("T"), identifier("E")],
            JavaType::primitive(JavaPrimitive::Boolean),
            "valueResultIsOk",
            vec![parameter(value_result.clone(), "value")],
            vec![JavaStmt::Return(Some(member_call(
                local(value_result.clone(), "value"),
                JavaRuntimeMember::ValueResultOk,
                vec![],
                JavaType::primitive(JavaPrimitive::Boolean),
            )))],
        ),
        static_method(
            vec![identifier("T"), identifier("E")],
            t.clone(),
            "valueResultValue",
            vec![parameter(value_result.clone(), "value")],
            vec![JavaStmt::Return(Some(member_call(
                local(value_result.clone(), "value"),
                JavaRuntimeMember::ValueResultValue,
                vec![],
                t,
            )))],
        ),
        static_method(
            vec![identifier("T"), identifier("E")],
            e.clone(),
            "valueResultError",
            vec![parameter(value_result.clone(), "value")],
            vec![JavaStmt::Return(Some(member_call(
                local(value_result, "value"),
                JavaRuntimeMember::ValueResultError,
                vec![],
                e,
            )))],
        ),
    ]
}

fn validated_option_type() -> JavaTypeDeclaration {
    let t = type_variable("T");
    let owner = generic(JavaKnownType::RuntimeOption, vec![t.clone()]);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let some = local(boolean.clone(), "some");
    let value = local(t.clone(), "value");
    let value_is_null = binary(
        JavaBinaryOperator::Equal,
        value.clone(),
        null_literal(t.clone()),
        boolean.clone(),
    );
    let invalid = binary(
        JavaBinaryOperator::LogicalOr,
        binary(
            JavaBinaryOperator::LogicalAnd,
            some.clone(),
            value_is_null.clone(),
            boolean.clone(),
        ),
        binary(
            JavaBinaryOperator::LogicalAnd,
            unary(JavaUnaryOperator::Not, some.clone(), boolean.clone()),
            unary(JavaUnaryOperator::Not, value_is_null, boolean.clone()),
            boolean.clone(),
        ),
        boolean.clone(),
    );
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier("PolyOption"),
        type_parameters: vec![identifier("T")],
        record_components: vec![
            component(boolean.clone(), "some", JavaRuntimeMember::OptionSome),
            component(t.clone(), "value", JavaRuntimeMember::OptionValue),
        ],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Public],
                name: identifier("PolyOption"),
                parameters: vec![
                    parameter(boolean.clone(), "some"),
                    parameter(t.clone(), "value"),
                ],
                body: JavaBlock::new(vec![
                    JavaStmt::If {
                        condition: invalid,
                        then_block: JavaBlock::new(vec![illegal_argument(
                            "PolyOption tag and payload disagree",
                        )]),
                        else_block: None,
                    },
                    assign_component(owner.clone(), "some", boolean.clone(), some),
                    assign_component(owner.clone(), "value", t.clone(), value),
                ]),
            }),
            guarded_accessor(
                owner.clone(),
                "value",
                t,
                unary(
                    JavaUnaryOperator::Not,
                    structural_field(this_value(owner), "some", boolean.clone()),
                    boolean,
                ),
                "cannot read value from None",
            ),
        ],
    }
}

fn validated_value_result_type() -> JavaTypeDeclaration {
    let t = type_variable("T");
    let e = type_variable("E");
    let owner = generic(
        JavaKnownType::RuntimeValueResult,
        vec![t.clone(), e.clone()],
    );
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let ok = local(boolean.clone(), "ok");
    let value = local(t.clone(), "value");
    let error = local(e.clone(), "error");
    let value_is_null = binary(
        JavaBinaryOperator::Equal,
        value.clone(),
        null_literal(t.clone()),
        boolean.clone(),
    );
    let error_is_null = binary(
        JavaBinaryOperator::Equal,
        error.clone(),
        null_literal(e.clone()),
        boolean.clone(),
    );
    let invalid = binary(
        JavaBinaryOperator::LogicalOr,
        binary(
            JavaBinaryOperator::LogicalAnd,
            ok.clone(),
            binary(
                JavaBinaryOperator::LogicalOr,
                value_is_null,
                unary(
                    JavaUnaryOperator::Not,
                    error_is_null.clone(),
                    boolean.clone(),
                ),
                boolean.clone(),
            ),
            boolean.clone(),
        ),
        binary(
            JavaBinaryOperator::LogicalAnd,
            unary(JavaUnaryOperator::Not, ok.clone(), boolean.clone()),
            binary(
                JavaBinaryOperator::LogicalOr,
                unary(
                    JavaUnaryOperator::Not,
                    binary(
                        JavaBinaryOperator::Equal,
                        value.clone(),
                        null_literal(t.clone()),
                        boolean.clone(),
                    ),
                    boolean.clone(),
                ),
                error_is_null,
                boolean.clone(),
            ),
            boolean.clone(),
        ),
        boolean.clone(),
    );
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier("PolyValueResult"),
        type_parameters: vec![identifier("T"), identifier("E")],
        record_components: vec![
            component(boolean.clone(), "ok", JavaRuntimeMember::ValueResultOk),
            component(t.clone(), "value", JavaRuntimeMember::ValueResultValue),
            component(e.clone(), "error", JavaRuntimeMember::ValueResultError),
        ],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Public],
                name: identifier("PolyValueResult"),
                parameters: vec![
                    parameter(boolean.clone(), "ok"),
                    parameter(t.clone(), "value"),
                    parameter(e.clone(), "error"),
                ],
                body: JavaBlock::new(vec![
                    JavaStmt::If {
                        condition: invalid,
                        then_block: JavaBlock::new(vec![illegal_argument(
                            "PolyValueResult tag and payloads disagree",
                        )]),
                        else_block: None,
                    },
                    assign_component(owner.clone(), "ok", boolean.clone(), ok),
                    assign_component(owner.clone(), "value", t.clone(), value),
                    assign_component(owner.clone(), "error", e.clone(), error),
                ]),
            }),
            guarded_accessor(
                owner.clone(),
                "value",
                t,
                unary(
                    JavaUnaryOperator::Not,
                    structural_field(this_value(owner.clone()), "ok", boolean.clone()),
                    boolean.clone(),
                ),
                "cannot read value from Err",
            ),
            guarded_accessor(
                owner.clone(),
                "error",
                e,
                structural_field(this_value(owner), "ok", boolean),
                "cannot read error from Ok",
            ),
        ],
    }
}

fn bytes_members() -> Vec<JavaMember> {
    let integer = JavaType::Boxed(JavaPrimitive::Int);
    let list = generic(JavaKnownType::List, vec![integer.clone()]);
    let bytes = JavaType::known(JavaKnownType::RuntimeBytes);
    let copied = local(list.clone(), "copy");
    vec![
        JavaMember::NestedType(JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Public,
            modifiers: vec![JavaModifier::Static],
            name: identifier("Bytes"),
            type_parameters: vec![],
            record_components: vec![component(
                list.clone(),
                "values",
                JavaRuntimeMember::BytesValues,
            )],
            heritage: JavaHeritage::Interfaces(vec![JavaType::known(
                JavaKnownType::RuntimeSemanticValue,
            )]),
            permits: vec![],
            members: vec![
                JavaMember::Constructor(JavaConstructor {
                    modifiers: vec![JavaModifier::Public],
                    name: identifier("Bytes"),
                    parameters: vec![parameter(list.clone(), "values")],
                    body: JavaBlock::new(vec![
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Final,
                            ty: list.clone(),
                            name: identifier("copy"),
                            value: Some(known_generic_call(
                                JavaKnownCallable::ListCopyOf,
                                vec![local(list.clone(), "values")],
                                list.clone(),
                            )),
                        },
                        JavaStmt::ForEach {
                            binding_type: integer.clone(),
                            binding: identifier("item"),
                            iterable: copied.clone(),
                            body: JavaBlock::new(vec![JavaStmt::If {
                                condition: binary(
                                    JavaBinaryOperator::LogicalOr,
                                    binary(
                                        JavaBinaryOperator::Less,
                                        cast(
                                            JavaType::primitive(JavaPrimitive::Int),
                                            local(integer.clone(), "item"),
                                        ),
                                        int_literal(0),
                                        JavaType::primitive(JavaPrimitive::Boolean),
                                    ),
                                    binary(
                                        JavaBinaryOperator::Greater,
                                        cast(
                                            JavaType::primitive(JavaPrimitive::Int),
                                            local(integer.clone(), "item"),
                                        ),
                                        int_literal(255),
                                        JavaType::primitive(JavaPrimitive::Boolean),
                                    ),
                                    JavaType::primitive(JavaPrimitive::Boolean),
                                ),
                                then_block: JavaBlock::new(vec![illegal_argument(
                                    "byte value is outside 0..255",
                                )]),
                                else_block: None,
                            }]),
                        },
                        assign_component(bytes.clone(), "values", list.clone(), copied),
                    ]),
                }),
                semantic_method(
                    bytes.clone(),
                    bytes.clone(),
                    &[component(
                        list.clone(),
                        "values",
                        JavaRuntimeMember::BytesValues,
                    )],
                ),
            ],
        }),
        static_method(
            vec![],
            bytes.clone(),
            "bytesOf",
            vec![parameter(list.clone(), "values")],
            vec![JavaStmt::Return(Some(new_known(
                JavaKnownConstructor::RuntimeBytes,
                bytes.clone(),
                vec![local(list, "values")],
            )))],
        ),
    ]
    .into_iter()
    .chain(
        JavaRuntimeCallable::ALL
            .into_iter()
            .filter(|value| {
                value.helper() == JavaRuntimeHelper::Bytes && *value != JavaRuntimeCallable::BytesOf
            })
            .map(runtime_method),
    )
    .collect()
}

fn runtime_methods(helper: JavaRuntimeHelper) -> Vec<JavaMember> {
    JavaRuntimeCallable::ALL
        .into_iter()
        .filter(|value| value.helper() == helper)
        .map(runtime_method)
        .collect()
}

fn runtime_method(value: JavaRuntimeCallable) -> JavaMember {
    match value {
        JavaRuntimeCallable::CheckedNegI32
        | JavaRuntimeCallable::CheckedNegI64
        | JavaRuntimeCallable::CheckedAddI32
        | JavaRuntimeCallable::CheckedAddI64
        | JavaRuntimeCallable::CheckedSubI32
        | JavaRuntimeCallable::CheckedSubI64
        | JavaRuntimeCallable::CheckedMulI32
        | JavaRuntimeCallable::CheckedMulI64
        | JavaRuntimeCallable::CheckedDivI32
        | JavaRuntimeCallable::CheckedDivI64
        | JavaRuntimeCallable::CheckedRemI32
        | JavaRuntimeCallable::CheckedRemI64
        | JavaRuntimeCallable::CheckedShiftLeftI32
        | JavaRuntimeCallable::CheckedShiftLeftI64
        | JavaRuntimeCallable::CheckedShiftRightI32
        | JavaRuntimeCallable::CheckedShiftRightI64
        | JavaRuntimeCallable::NarrowI64ToI32 => checked_integer_method(value),
        JavaRuntimeCallable::FloatTrunc
        | JavaRuntimeCallable::FloatIsNegativeZero
        | JavaRuntimeCallable::FloatAbs => float_method(value),
        JavaRuntimeCallable::RequireScalarString
        | JavaRuntimeCallable::CompareScalarStrings
        | JavaRuntimeCallable::ScalarLength
        | JavaRuntimeCallable::StringIndexOfLiteral
        | JavaRuntimeCallable::StringSliceScalars
        | JavaRuntimeCallable::StringToUtf8
        | JavaRuntimeCallable::StringFromUtf8 => unicode_method(value),
        JavaRuntimeCallable::BytesLength
        | JavaRuntimeCallable::BytesIsEmpty
        | JavaRuntimeCallable::BytesConcat
        | JavaRuntimeCallable::BytesReplaceAll => bytes_method(value),
        JavaRuntimeCallable::ListCopy
        | JavaRuntimeCallable::ListLength
        | JavaRuntimeCallable::ListIsEmpty
        | JavaRuntimeCallable::ListGet
        | JavaRuntimeCallable::ListAppend
        | JavaRuntimeCallable::ListConcat
        | JavaRuntimeCallable::ListContains
        | JavaRuntimeCallable::ListIndexOf => list_method(value),
        JavaRuntimeCallable::StringReplaceMany
        | JavaRuntimeCallable::StringTruncateUtf8Bytes
        | JavaRuntimeCallable::StringTrimStart
        | JavaRuntimeCallable::StringTrimEnd => string_method(value),
        JavaRuntimeCallable::Ok
        | JavaRuntimeCallable::Fail
        | JavaRuntimeCallable::DeepEqual
        | JavaRuntimeCallable::SemanticEqual
        | JavaRuntimeCallable::OptionNone
        | JavaRuntimeCallable::OptionSome
        | JavaRuntimeCallable::OptionIsSome
        | JavaRuntimeCallable::OptionValue
        | JavaRuntimeCallable::ValueResultOk
        | JavaRuntimeCallable::ValueResultErr
        | JavaRuntimeCallable::ValueResultIsOk
        | JavaRuntimeCallable::ValueResultValue
        | JavaRuntimeCallable::ValueResultError
        | JavaRuntimeCallable::BytesOf => {
            unreachable!("{value:?} has a dedicated typed declaration")
        }
    }
}

fn string_method(value: JavaRuntimeCallable) -> JavaMember {
    match value {
        JavaRuntimeCallable::StringReplaceMany => string_replace_many_method(value),
        JavaRuntimeCallable::StringTruncateUtf8Bytes => string_truncate_method(value),
        JavaRuntimeCallable::StringTrimStart | JavaRuntimeCallable::StringTrimEnd => {
            string_trim_method(value)
        }
        _ => unreachable!(),
    }
}

fn string_replace_many_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let replacements = generic(JavaKnownType::List, vec![string.clone()]);
    let source = local(string.clone(), "source");
    let mappings = local(replacements.clone(), "replacements");
    let output = local(string.clone(), "output");
    let offset = local(int.clone(), "offset");
    let mapping = local(int.clone(), "mapping");
    let remaining = local(string.clone(), "remaining");
    let matched = local(boolean.clone(), "matched");
    let needle = local(string.clone(), "needle");
    let width = local(int.clone(), "width");
    let append_one_scalar = vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: int.clone(),
            name: identifier("width"),
            value: Some(known_call(
                JavaKnownCallable::CharacterCharCount,
                vec![known_method_call(
                    JavaKnownMethod::StringCodePointAt,
                    remaining.clone(),
                    vec![int_literal(0)],
                    int.clone(),
                )],
            )),
        },
        JavaStmt::Assign {
            target: output.clone(),
            value: binary(
                JavaBinaryOperator::Add,
                output.clone(),
                known_method_call(
                    JavaKnownMethod::StringSubstringRange,
                    remaining.clone(),
                    vec![int_literal(0), width.clone()],
                    string.clone(),
                ),
                string.clone(),
            ),
        },
        JavaStmt::Assign {
            target: offset.clone(),
            value: binary(JavaBinaryOperator::Add, offset.clone(), width, int.clone()),
        },
    ];
    static_method(
        vec![],
        string.clone(),
        value.name(),
        vec![
            parameter(string.clone(), "source"),
            parameter(replacements.clone(), "replacements"),
        ],
        vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: string.clone(),
                name: identifier("output"),
                value: Some(string_literal("")),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("offset"),
                value: Some(int_literal(0)),
            },
            JavaStmt::While {
                condition: bool_literal(true),
                body: JavaBlock::new(
                    vec![
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Final,
                            ty: string.clone(),
                            name: identifier("remaining"),
                            value: Some(known_method_call(
                                JavaKnownMethod::StringSubstringFrom,
                                source,
                                vec![offset.clone()],
                                string.clone(),
                            )),
                        },
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Mutable,
                            ty: boolean.clone(),
                            name: identifier("matched"),
                            value: Some(bool_literal(false)),
                        },
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Mutable,
                            ty: int.clone(),
                            name: identifier("mapping"),
                            value: Some(int_literal(0)),
                        },
                        JavaStmt::While {
                            condition: binary(
                                JavaBinaryOperator::Less,
                                mapping.clone(),
                                known_method_call(
                                    JavaKnownMethod::ListSize,
                                    mappings.clone(),
                                    vec![],
                                    int.clone(),
                                ),
                                boolean.clone(),
                            ),
                            body: JavaBlock::new(vec![
                                JavaStmt::Local {
                                    finality: JavaLocalFinality::Final,
                                    ty: string.clone(),
                                    name: identifier("needle"),
                                    value: Some(known_method_call(
                                        JavaKnownMethod::ListGet,
                                        mappings.clone(),
                                        vec![mapping.clone()],
                                        string.clone(),
                                    )),
                                },
                                JavaStmt::Local {
                                    finality: JavaLocalFinality::Final,
                                    ty: string.clone(),
                                    name: identifier("replacement"),
                                    value: Some(known_method_call(
                                        JavaKnownMethod::ListGet,
                                        mappings.clone(),
                                        vec![binary(
                                            JavaBinaryOperator::Add,
                                            mapping.clone(),
                                            int_literal(1),
                                            int.clone(),
                                        )],
                                        string.clone(),
                                    )),
                                },
                                JavaStmt::If {
                                    condition: known_method_call(
                                        JavaKnownMethod::StringStartsWith,
                                        remaining.clone(),
                                        vec![needle.clone()],
                                        boolean.clone(),
                                    ),
                                    then_block: JavaBlock::new(vec![
                                        JavaStmt::Assign {
                                            target: output.clone(),
                                            value: binary(
                                                JavaBinaryOperator::Add,
                                                output.clone(),
                                                local(string.clone(), "replacement"),
                                                string.clone(),
                                            ),
                                        },
                                        JavaStmt::If {
                                            condition: known_method_call(
                                                JavaKnownMethod::StringIsEmpty,
                                                needle.clone(),
                                                vec![],
                                                boolean.clone(),
                                            ),
                                            then_block: JavaBlock::new(
                                                vec![JavaStmt::If {
                                                    condition: known_method_call(
                                                        JavaKnownMethod::StringIsEmpty,
                                                        remaining.clone(),
                                                        vec![],
                                                        boolean.clone(),
                                                    ),
                                                    then_block: JavaBlock::new(vec![
                                                        JavaStmt::Return(Some(output.clone())),
                                                    ]),
                                                    else_block: None,
                                                }]
                                                .into_iter()
                                                .chain(append_one_scalar.clone())
                                                .collect(),
                                            ),
                                            else_block: Some(JavaBlock::new(vec![
                                                JavaStmt::Assign {
                                                    target: offset.clone(),
                                                    value: binary(
                                                        JavaBinaryOperator::Add,
                                                        offset.clone(),
                                                        known_method_call(
                                                            JavaKnownMethod::StringLength,
                                                            needle,
                                                            vec![],
                                                            int.clone(),
                                                        ),
                                                        int.clone(),
                                                    ),
                                                },
                                            ])),
                                        },
                                        JavaStmt::Assign {
                                            target: matched.clone(),
                                            value: bool_literal(true),
                                        },
                                        JavaStmt::Break,
                                    ]),
                                    else_block: None,
                                },
                                JavaStmt::Assign {
                                    target: mapping.clone(),
                                    value: binary(
                                        JavaBinaryOperator::Add,
                                        mapping,
                                        int_literal(2),
                                        int.clone(),
                                    ),
                                },
                            ]),
                        },
                        JavaStmt::If {
                            condition: matched,
                            then_block: JavaBlock::new(vec![JavaStmt::Continue]),
                            else_block: None,
                        },
                        JavaStmt::If {
                            condition: known_method_call(
                                JavaKnownMethod::StringIsEmpty,
                                remaining.clone(),
                                vec![],
                                boolean,
                            ),
                            then_block: JavaBlock::new(vec![JavaStmt::Break]),
                            else_block: None,
                        },
                    ]
                    .into_iter()
                    .chain(append_one_scalar)
                    .collect(),
                ),
            },
            JavaStmt::Return(Some(output)),
        ],
    )
}

fn string_truncate_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let double = JavaType::primitive(JavaPrimitive::Double);
    let source = local(string.clone(), "source");
    let offset = local(int.clone(), "offset");
    let consumed = local(int.clone(), "consumed");
    let width = local(int.clone(), "width");
    let end = local(int.clone(), "end");
    let code_point = local(int.clone(), "codePoint");
    let utf8_width = conditional(
        binary(
            JavaBinaryOperator::LessEqual,
            code_point.clone(),
            int_literal(0x7f),
            boolean.clone(),
        ),
        int_literal(1),
        conditional(
            binary(
                JavaBinaryOperator::LessEqual,
                code_point.clone(),
                int_literal(0x7ff),
                boolean.clone(),
            ),
            int_literal(2),
            conditional(
                binary(
                    JavaBinaryOperator::LessEqual,
                    code_point,
                    int_literal(0xffff),
                    boolean.clone(),
                ),
                int_literal(3),
                int_literal(4),
                int.clone(),
            ),
            int.clone(),
        ),
        int.clone(),
    );
    let next_consumed = binary(
        JavaBinaryOperator::Add,
        consumed.clone(),
        width,
        int.clone(),
    );
    static_method(
        vec![],
        string.clone(),
        value.name(),
        vec![
            parameter(string.clone(), "source"),
            parameter(double.clone(), "budget"),
        ],
        vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("offset"),
                value: Some(int_literal(0)),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("consumed"),
                value: Some(int_literal(0)),
            },
            JavaStmt::While {
                condition: binary(
                    JavaBinaryOperator::Less,
                    offset.clone(),
                    known_method_call(
                        JavaKnownMethod::StringLength,
                        source.clone(),
                        vec![],
                        int.clone(),
                    ),
                    boolean.clone(),
                ),
                body: JavaBlock::new(vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("codePoint"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCodePointAt,
                            source.clone(),
                            vec![offset.clone()],
                            int.clone(),
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("end"),
                        value: Some(binary(
                            JavaBinaryOperator::Add,
                            offset.clone(),
                            known_call(
                                JavaKnownCallable::CharacterCharCount,
                                vec![local(int.clone(), "codePoint")],
                            ),
                            int.clone(),
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("width"),
                        value: Some(utf8_width),
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Equal,
                            cast(double.clone(), next_consumed.clone()),
                            local(double.clone(), "budget"),
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                            known_method_call(
                                JavaKnownMethod::StringSubstringRange,
                                source.clone(),
                                vec![int_literal(0), end.clone()],
                                string.clone(),
                            ),
                        ))]),
                        else_block: None,
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Greater,
                            cast(double, next_consumed.clone()),
                            local(JavaType::primitive(JavaPrimitive::Double), "budget"),
                            boolean,
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                            known_method_call(
                                JavaKnownMethod::StringSubstringRange,
                                source.clone(),
                                vec![int_literal(0), offset.clone()],
                                string.clone(),
                            ),
                        ))]),
                        else_block: None,
                    },
                    JavaStmt::Assign {
                        target: consumed,
                        value: next_consumed,
                    },
                    JavaStmt::Assign {
                        target: offset,
                        value: end,
                    },
                ]),
            },
            JavaStmt::Return(Some(source)),
        ],
    )
}

fn string_trim_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let source = local(string.clone(), "source");
    let characters = local(string.clone(), "characters");
    let offset = local(int.clone(), "offset");
    let code_point = local(int.clone(), "codePoint");
    let trim_start = value == JavaRuntimeCallable::StringTrimStart;
    let next_code_point = known_method_call(
        if trim_start {
            JavaKnownMethod::StringCodePointAt
        } else {
            JavaKnownMethod::StringCodePointBefore
        },
        source.clone(),
        vec![offset.clone()],
        int.clone(),
    );
    let advance = binary(
        if trim_start {
            JavaBinaryOperator::Add
        } else {
            JavaBinaryOperator::Subtract
        },
        offset.clone(),
        known_call(
            JavaKnownCallable::CharacterCharCount,
            vec![code_point.clone()],
        ),
        int.clone(),
    );
    static_method(
        vec![],
        string.clone(),
        value.name(),
        vec![
            parameter(string.clone(), "source"),
            parameter(string.clone(), "characters"),
        ],
        vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("offset"),
                value: Some(if trim_start {
                    int_literal(0)
                } else {
                    known_method_call(
                        JavaKnownMethod::StringLength,
                        source.clone(),
                        vec![],
                        int.clone(),
                    )
                }),
            },
            JavaStmt::While {
                condition: if trim_start {
                    binary(
                        JavaBinaryOperator::Less,
                        offset.clone(),
                        known_method_call(
                            JavaKnownMethod::StringLength,
                            source.clone(),
                            vec![],
                            int.clone(),
                        ),
                        boolean.clone(),
                    )
                } else {
                    binary(
                        JavaBinaryOperator::Greater,
                        offset.clone(),
                        int_literal(0),
                        boolean.clone(),
                    )
                },
                body: JavaBlock::new(vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("codePoint"),
                        value: Some(next_code_point),
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Less,
                            known_method_call(
                                JavaKnownMethod::StringIndexOfCodePoint,
                                characters,
                                vec![code_point],
                                int.clone(),
                            ),
                            int_literal(0),
                            boolean,
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Break]),
                        else_block: None,
                    },
                    JavaStmt::Assign {
                        target: offset,
                        value: advance,
                    },
                ]),
            },
            JavaStmt::Return(Some(if trim_start {
                known_method_call(
                    JavaKnownMethod::StringSubstringFrom,
                    source,
                    vec![local(int, "offset")],
                    string,
                )
            } else {
                known_method_call(
                    JavaKnownMethod::StringSubstringRange,
                    source,
                    vec![int_literal(0), local(int, "offset")],
                    string,
                )
            })),
        ],
    )
}

fn unicode_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let byte = JavaType::primitive(JavaPrimitive::Byte);
    let character = JavaType::primitive(JavaPrimitive::Char);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let long = JavaType::primitive(JavaPrimitive::Long);
    let integer = JavaType::Boxed(JavaPrimitive::Int);
    let bytes = JavaType::known(JavaKnownType::RuntimeBytes);
    let byte_array = JavaType::Array {
        component: Box::new(byte.clone()),
        ownership: JavaArrayOwnership::InternalMutable,
    };
    let integer_list = generic(JavaKnownType::List, vec![integer.clone()]);
    let integer_array_list = generic(JavaKnownType::ArrayList, vec![integer.clone()]);
    let result_i64 = generic(
        JavaKnownType::RuntimeResult,
        vec![JavaType::Boxed(JavaPrimitive::Long)],
    );
    let result_string = generic(JavaKnownType::RuntimeResult, vec![string.clone()]);
    match value {
        JavaRuntimeCallable::RequireScalarString => require_scalar_string_method(value),
        JavaRuntimeCallable::CompareScalarStrings => compare_scalar_strings_method(value),
        JavaRuntimeCallable::ScalarLength => {
            let source = local(string.clone(), "value");
            let length = local(int.clone(), "length");
            let index = local(int.clone(), "index");
            let unit = local(character.clone(), "unit");
            let next_index = binary(
                JavaBinaryOperator::Add,
                index.clone(),
                int_literal(1),
                int.clone(),
            );
            let next_unit = known_method_call(
                JavaKnownMethod::StringCharAt,
                source.clone(),
                vec![next_index.clone()],
                character.clone(),
            );
            let invalid_pair = binary(
                JavaBinaryOperator::LogicalOr,
                binary(
                    JavaBinaryOperator::GreaterEqual,
                    next_index,
                    length.clone(),
                    boolean.clone(),
                ),
                unary(
                    JavaUnaryOperator::Not,
                    known_call(JavaKnownCallable::CharacterIsLowSurrogate, vec![next_unit]),
                    boolean.clone(),
                ),
                boolean.clone(),
            );
            static_method(
                vec![],
                result_i64.clone(),
                value.name(),
                vec![parameter(string.clone(), "value")],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("length"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringLength,
                            source.clone(),
                            vec![],
                            int.clone(),
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Mutable,
                        ty: int.clone(),
                        name: identifier("index"),
                        value: Some(int_literal(0)),
                    },
                    JavaStmt::While {
                        condition: binary(
                            JavaBinaryOperator::Less,
                            index.clone(),
                            length.clone(),
                            boolean.clone(),
                        ),
                        body: JavaBlock::new(vec![
                            JavaStmt::Local {
                                finality: JavaLocalFinality::Final,
                                ty: character.clone(),
                                name: identifier("unit"),
                                value: Some(known_method_call(
                                    JavaKnownMethod::StringCharAt,
                                    source.clone(),
                                    vec![index.clone()],
                                    character.clone(),
                                )),
                            },
                            JavaStmt::If {
                                condition: known_call(
                                    JavaKnownCallable::CharacterIsHighSurrogate,
                                    vec![unit.clone()],
                                ),
                                then_block: JavaBlock::new(vec![
                                    JavaStmt::If {
                                        condition: invalid_pair,
                                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                                            runtime_fail(
                                                result_i64.clone(),
                                                JavaRuntimeFailure::InvalidUnicodeScalar,
                                            ),
                                        ))]),
                                        else_block: None,
                                    },
                                    JavaStmt::Assign {
                                        target: index.clone(),
                                        value: binary(
                                            JavaBinaryOperator::Add,
                                            index.clone(),
                                            int_literal(2),
                                            int.clone(),
                                        ),
                                    },
                                    JavaStmt::Continue,
                                ]),
                                else_block: None,
                            },
                            JavaStmt::If {
                                condition: known_call(
                                    JavaKnownCallable::CharacterIsLowSurrogate,
                                    vec![unit],
                                ),
                                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                                    runtime_fail(
                                        result_i64.clone(),
                                        JavaRuntimeFailure::InvalidUnicodeScalar,
                                    ),
                                ))]),
                                else_block: None,
                            },
                            JavaStmt::Assign {
                                target: index.clone(),
                                value: binary(
                                    JavaBinaryOperator::Add,
                                    index,
                                    int_literal(1),
                                    int.clone(),
                                ),
                            },
                        ]),
                    },
                    JavaStmt::Return(Some(runtime_ok(
                        result_i64,
                        cast(
                            long,
                            known_method_call(
                                JavaKnownMethod::StringCodePointCount,
                                source,
                                vec![int_literal(0), length],
                                int,
                            ),
                        ),
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::StringIndexOfLiteral => {
            let source = local(string.clone(), "source");
            let offset = local(int.clone(), "offset");
            let option_long = generic(
                JavaKnownType::RuntimeOption,
                vec![JavaType::Boxed(JavaPrimitive::Long)],
            );
            static_method(
                vec![],
                option_long.clone(),
                value.name(),
                vec![
                    parameter(string.clone(), "source"),
                    parameter(string.clone(), "needle"),
                ],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("offset"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringIndexOfString,
                            source.clone(),
                            vec![local(string.clone(), "needle")],
                            int.clone(),
                        )),
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Less,
                            offset.clone(),
                            int_literal(0),
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_call(
                            JavaRuntimeCallable::OptionNone,
                            vec![],
                            option_long.clone(),
                        )))]),
                        else_block: None,
                    },
                    JavaStmt::Return(Some(runtime_call(
                        JavaRuntimeCallable::OptionSome,
                        vec![cast(
                            long,
                            known_method_call(
                                JavaKnownMethod::StringCodePointCount,
                                source,
                                vec![int_literal(0), offset],
                                int,
                            ),
                        )],
                        option_long,
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::StringSliceScalars => {
            let source = local(string.clone(), "source");
            let scalar_length = local(int.clone(), "scalarLength");
            let scalar_length_long = cast(long.clone(), scalar_length.clone());
            let clamp = |name: &str| {
                let operand = local(long.clone(), name);
                conditional(
                    binary(
                        JavaBinaryOperator::Less,
                        operand.clone(),
                        long_literal(0),
                        boolean.clone(),
                    ),
                    long_literal(0),
                    conditional(
                        binary(
                            JavaBinaryOperator::Greater,
                            operand.clone(),
                            scalar_length_long.clone(),
                            boolean.clone(),
                        ),
                        scalar_length_long.clone(),
                        operand,
                        long.clone(),
                    ),
                    long.clone(),
                )
            };
            let clamped_start = local(long.clone(), "clampedStart");
            let clamped_end = local(long.clone(), "clampedEnd");
            static_method(
                vec![],
                string.clone(),
                value.name(),
                vec![
                    parameter(string.clone(), "source"),
                    parameter(long.clone(), "start"),
                    parameter(long.clone(), "end"),
                ],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("scalarLength"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCodePointCount,
                            source.clone(),
                            vec![
                                int_literal(0),
                                known_method_call(
                                    JavaKnownMethod::StringLength,
                                    source.clone(),
                                    vec![],
                                    int.clone(),
                                ),
                            ],
                            int.clone(),
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: long.clone(),
                        name: identifier("clampedStart"),
                        value: Some(clamp("start")),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: long.clone(),
                        name: identifier("clampedEnd"),
                        value: Some(clamp("end")),
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::GreaterEqual,
                            clamped_start.clone(),
                            clamped_end.clone(),
                            boolean,
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(string_literal(
                            "",
                        )))]),
                        else_block: None,
                    },
                    JavaStmt::Return(Some(known_method_call(
                        JavaKnownMethod::StringSubstringRange,
                        source.clone(),
                        vec![
                            known_method_call(
                                JavaKnownMethod::StringOffsetByCodePoints,
                                source.clone(),
                                vec![int_literal(0), cast(int.clone(), clamped_start)],
                                int.clone(),
                            ),
                            known_method_call(
                                JavaKnownMethod::StringOffsetByCodePoints,
                                source,
                                vec![int_literal(0), cast(int.clone(), clamped_end)],
                                int,
                            ),
                        ],
                        string,
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::StringToUtf8 => {
            let raw = local(byte_array.clone(), "raw");
            let output = local(integer_array_list.clone(), "output");
            static_method(
                vec![],
                bytes.clone(),
                value.name(),
                vec![parameter(string.clone(), "value")],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: byte_array.clone(),
                        name: identifier("raw"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringGetBytes,
                            local(string, "value"),
                            vec![known_field(JavaKnownField::StandardCharsetsUtf8)],
                            byte_array,
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: integer_array_list.clone(),
                        name: identifier("output"),
                        value: Some(new_known(
                            JavaKnownConstructor::ArrayList,
                            integer_array_list,
                            vec![],
                        )),
                    },
                    JavaStmt::ForEach {
                        binding_type: byte,
                        binding: identifier("item"),
                        iterable: raw,
                        body: JavaBlock::new(vec![JavaStmt::Expression(known_method_call(
                            JavaKnownMethod::ArrayListAdd,
                            output.clone(),
                            vec![known_call(
                                JavaKnownCallable::ByteToUnsignedInt,
                                vec![local(JavaType::primitive(JavaPrimitive::Byte), "item")],
                            )],
                            JavaType::primitive(JavaPrimitive::Boolean),
                        ))]),
                    },
                    JavaStmt::Return(Some(runtime_call(
                        JavaRuntimeCallable::BytesOf,
                        vec![output],
                        bytes,
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::StringFromUtf8 => string_from_utf8_method(
            value,
            string,
            bytes,
            byte_array,
            integer_list,
            result_string,
        ),
        _ => unreachable!(),
    }
}

fn require_scalar_string_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let character = JavaType::primitive(JavaPrimitive::Char);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let source = local(string.clone(), "value");
    let length = local(int.clone(), "length");
    let index = local(int.clone(), "index");
    let unit = local(character.clone(), "unit");
    let next = binary(
        JavaBinaryOperator::Add,
        index.clone(),
        int_literal(1),
        int.clone(),
    );
    let invalid_pair = binary(
        JavaBinaryOperator::LogicalOr,
        binary(
            JavaBinaryOperator::GreaterEqual,
            next.clone(),
            length.clone(),
            boolean.clone(),
        ),
        unary(
            JavaUnaryOperator::Not,
            known_call(
                JavaKnownCallable::CharacterIsLowSurrogate,
                vec![known_method_call(
                    JavaKnownMethod::StringCharAt,
                    source.clone(),
                    vec![next],
                    character.clone(),
                )],
            ),
            boolean.clone(),
        ),
        boolean.clone(),
    );
    let throw_invalid = || {
        JavaStmt::Throw(new_known(
            JavaKnownConstructor::IllegalArgumentExceptionString,
            JavaType::known(JavaKnownType::IllegalArgumentException),
            vec![string_literal(
                "string contains an unpaired UTF-16 surrogate",
            )],
        ))
    };
    static_method(
        vec![],
        string.clone(),
        value.name(),
        vec![parameter(string.clone(), "value")],
        vec![
            JavaStmt::Expression(known_generic_call(
                JavaKnownCallable::ObjectsRequireNonNull,
                vec![source.clone()],
                string.clone(),
            )),
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: int.clone(),
                name: identifier("length"),
                value: Some(known_method_call(
                    JavaKnownMethod::StringLength,
                    source.clone(),
                    vec![],
                    int.clone(),
                )),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("index"),
                value: Some(int_literal(0)),
            },
            JavaStmt::While {
                condition: binary(
                    JavaBinaryOperator::Less,
                    index.clone(),
                    length,
                    boolean.clone(),
                ),
                body: JavaBlock::new(vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: character.clone(),
                        name: identifier("unit"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCharAt,
                            source.clone(),
                            vec![index.clone()],
                            character,
                        )),
                    },
                    JavaStmt::If {
                        condition: known_call(
                            JavaKnownCallable::CharacterIsHighSurrogate,
                            vec![unit.clone()],
                        ),
                        then_block: JavaBlock::new(vec![
                            JavaStmt::If {
                                condition: invalid_pair,
                                then_block: JavaBlock::new(vec![throw_invalid()]),
                                else_block: None,
                            },
                            JavaStmt::Assign {
                                target: index.clone(),
                                value: binary(
                                    JavaBinaryOperator::Add,
                                    index.clone(),
                                    int_literal(2),
                                    int.clone(),
                                ),
                            },
                            JavaStmt::Continue,
                        ]),
                        else_block: None,
                    },
                    JavaStmt::If {
                        condition: known_call(
                            JavaKnownCallable::CharacterIsLowSurrogate,
                            vec![unit],
                        ),
                        then_block: JavaBlock::new(vec![throw_invalid()]),
                        else_block: None,
                    },
                    JavaStmt::Assign {
                        target: index.clone(),
                        value: binary(JavaBinaryOperator::Add, index, int_literal(1), int),
                    },
                ]),
            },
            JavaStmt::Return(Some(source)),
        ],
    )
}

fn compare_scalar_strings_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let left = local(string.clone(), "left");
    let right = local(string.clone(), "right");
    let left_index = local(int.clone(), "leftIndex");
    let right_index = local(int.clone(), "rightIndex");
    let left_scalar = local(int.clone(), "leftScalar");
    let right_scalar = local(int.clone(), "rightScalar");
    let length = |receiver: JavaExpr| {
        known_method_call(JavaKnownMethod::StringLength, receiver, vec![], int.clone())
    };
    static_method(
        vec![],
        int.clone(),
        value.name(),
        vec![
            parameter(string.clone(), "left"),
            parameter(string.clone(), "right"),
        ],
        vec![
            JavaStmt::Expression(runtime_call(
                JavaRuntimeCallable::RequireScalarString,
                vec![left.clone()],
                string.clone(),
            )),
            JavaStmt::Expression(runtime_call(
                JavaRuntimeCallable::RequireScalarString,
                vec![right.clone()],
                string.clone(),
            )),
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("leftIndex"),
                value: Some(int_literal(0)),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("rightIndex"),
                value: Some(int_literal(0)),
            },
            JavaStmt::While {
                condition: binary(
                    JavaBinaryOperator::LogicalAnd,
                    binary(
                        JavaBinaryOperator::Less,
                        left_index.clone(),
                        length(left.clone()),
                        boolean.clone(),
                    ),
                    binary(
                        JavaBinaryOperator::Less,
                        right_index.clone(),
                        length(right.clone()),
                        boolean.clone(),
                    ),
                    boolean.clone(),
                ),
                body: JavaBlock::new(vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("leftScalar"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCodePointAt,
                            left.clone(),
                            vec![left_index.clone()],
                            int.clone(),
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("rightScalar"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCodePointAt,
                            right.clone(),
                            vec![right_index.clone()],
                            int.clone(),
                        )),
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Less,
                            left_scalar.clone(),
                            right_scalar.clone(),
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(int_literal(-1)))]),
                        else_block: None,
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Greater,
                            left_scalar.clone(),
                            right_scalar.clone(),
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(int_literal(1)))]),
                        else_block: None,
                    },
                    JavaStmt::Assign {
                        target: left_index.clone(),
                        value: binary(
                            JavaBinaryOperator::Add,
                            left_index.clone(),
                            known_call(
                                JavaKnownCallable::CharacterCharCount,
                                vec![left_scalar.clone()],
                            ),
                            int.clone(),
                        ),
                    },
                    JavaStmt::Assign {
                        target: right_index.clone(),
                        value: binary(
                            JavaBinaryOperator::Add,
                            right_index.clone(),
                            known_call(JavaKnownCallable::CharacterCharCount, vec![right_scalar]),
                            int.clone(),
                        ),
                    },
                ]),
            },
            JavaStmt::If {
                condition: binary(
                    JavaBinaryOperator::Equal,
                    left_index.clone(),
                    length(left),
                    boolean.clone(),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(conditional(
                    binary(
                        JavaBinaryOperator::Equal,
                        right_index,
                        length(right),
                        boolean,
                    ),
                    int_literal(0),
                    int_literal(-1),
                    int.clone(),
                )))]),
                else_block: None,
            },
            JavaStmt::Return(Some(int_literal(1))),
        ],
    )
}

fn string_from_utf8_method(
    value: JavaRuntimeCallable,
    string: JavaType,
    bytes: JavaType,
    byte_array: JavaType,
    integer_list: JavaType,
    result_string: JavaType,
) -> JavaMember {
    let byte = JavaType::primitive(JavaPrimitive::Byte);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let decoder = JavaType::known(JavaKnownType::CharsetDecoder);
    let char_buffer = JavaType::known(JavaKnownType::CharBuffer);
    let values = local(integer_list.clone(), "values");
    let raw = local(byte_array.clone(), "raw");
    let index = local(int.clone(), "index");
    let decoder_value = local(decoder.clone(), "decoder");
    let report = known_field(JavaKnownField::CodingErrorReport);
    static_method(
        vec![],
        result_string.clone(),
        value.name(),
        vec![parameter(bytes.clone(), "value")],
        vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: integer_list.clone(),
                name: identifier("values"),
                value: Some(bytes_values(local(bytes, "value"), integer_list.clone())),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: byte_array.clone(),
                name: identifier("raw"),
                value: Some(new_array(
                    byte.clone(),
                    known_method_call(
                        JavaKnownMethod::ListSize,
                        values.clone(),
                        vec![],
                        int.clone(),
                    ),
                )),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("index"),
                value: Some(int_literal(0)),
            },
            JavaStmt::While {
                condition: binary(
                    JavaBinaryOperator::Less,
                    index.clone(),
                    known_method_call(
                        JavaKnownMethod::ListSize,
                        values.clone(),
                        vec![],
                        int.clone(),
                    ),
                    boolean,
                ),
                body: JavaBlock::new(vec![
                    JavaStmt::Assign {
                        target: array_index(raw.clone(), index.clone(), byte.clone()),
                        value: cast(
                            byte,
                            cast(
                                int.clone(),
                                known_method_call(
                                    JavaKnownMethod::ListGet,
                                    values,
                                    vec![index.clone()],
                                    JavaType::Boxed(JavaPrimitive::Int),
                                ),
                            ),
                        ),
                    },
                    JavaStmt::Assign {
                        target: index.clone(),
                        value: binary(JavaBinaryOperator::Add, index, int_literal(1), int),
                    },
                ]),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: decoder.clone(),
                name: identifier("decoder"),
                value: Some(known_method_call(
                    JavaKnownMethod::CharsetNewDecoder,
                    known_field(JavaKnownField::StandardCharsetsUtf8),
                    vec![],
                    decoder.clone(),
                )),
            },
            JavaStmt::Expression(known_method_call(
                JavaKnownMethod::DecoderOnMalformedInput,
                decoder_value.clone(),
                vec![report.clone()],
                decoder.clone(),
            )),
            JavaStmt::Expression(known_method_call(
                JavaKnownMethod::DecoderOnUnmappableCharacter,
                decoder_value.clone(),
                vec![report],
                decoder.clone(),
            )),
            JavaStmt::TryCatch {
                try_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_ok(
                    result_string.clone(),
                    known_method_call(
                        JavaKnownMethod::CharBufferToString,
                        known_method_call(
                            JavaKnownMethod::DecoderDecode,
                            decoder_value,
                            vec![known_call(JavaKnownCallable::ByteBufferWrap, vec![raw])],
                            char_buffer,
                        ),
                        vec![],
                        string,
                    ),
                )))]),
                catches: vec![JavaCatch {
                    exception_type: JavaType::known(JavaKnownType::CharacterCodingException),
                    binding: identifier("failure"),
                    body: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                        result_string,
                        JavaRuntimeFailure::InvalidUtf8,
                    )))]),
                }],
            },
        ],
    )
}

fn checked_integer_method(value: JavaRuntimeCallable) -> JavaMember {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let long = JavaType::primitive(JavaPrimitive::Long);
    let bigint = JavaType::known(JavaKnownType::BigInteger);
    let wide = matches!(
        value,
        JavaRuntimeCallable::CheckedNegI64
            | JavaRuntimeCallable::CheckedAddI64
            | JavaRuntimeCallable::CheckedSubI64
            | JavaRuntimeCallable::CheckedMulI64
            | JavaRuntimeCallable::CheckedDivI64
            | JavaRuntimeCallable::CheckedRemI64
            | JavaRuntimeCallable::CheckedShiftLeftI64
            | JavaRuntimeCallable::CheckedShiftRightI64
    );
    let operand = if wide { long.clone() } else { int.clone() };
    let boxed = if wide {
        JavaType::Boxed(JavaPrimitive::Long)
    } else {
        JavaType::Boxed(JavaPrimitive::Int)
    };
    let result = generic(JavaKnownType::RuntimeResult, vec![boxed]);

    if value == JavaRuntimeCallable::NarrowI64ToI32 {
        let narrow_result = generic(
            JavaKnownType::RuntimeResult,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        );
        return static_method(
            vec![],
            narrow_result.clone(),
            value.name(),
            vec![parameter(long.clone(), "value")],
            checked_bounds_statements(
                local(long, "value"),
                narrow_result,
                JavaKnownField::IntegerMinValue,
                JavaKnownField::IntegerMaxValue,
                int,
                JavaRuntimeFailure::NarrowingOutOfRange,
            ),
        );
    }

    if matches!(
        value,
        JavaRuntimeCallable::CheckedShiftLeftI32
            | JavaRuntimeCallable::CheckedShiftLeftI64
            | JavaRuntimeCallable::CheckedShiftRightI32
            | JavaRuntimeCallable::CheckedShiftRightI64
    ) {
        let amount = operand.clone();
        let right = local(amount.clone(), "right");
        let invalid = binary(
            JavaBinaryOperator::LogicalOr,
            binary(
                JavaBinaryOperator::Less,
                right.clone(),
                if wide {
                    long_literal(0)
                } else {
                    int_literal(0)
                },
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            binary(
                JavaBinaryOperator::GreaterEqual,
                right.clone(),
                if wide {
                    long_literal(64)
                } else {
                    int_literal(32)
                },
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            JavaType::primitive(JavaPrimitive::Boolean),
        );
        let shift_amount = if wide {
            cast(int.clone(), right)
        } else {
            right
        };
        let shifted = binary(
            if matches!(
                value,
                JavaRuntimeCallable::CheckedShiftLeftI32 | JavaRuntimeCallable::CheckedShiftLeftI64
            ) {
                JavaBinaryOperator::ShiftLeft
            } else {
                JavaBinaryOperator::ShiftRight
            },
            local(operand.clone(), "left"),
            shift_amount,
            operand.clone(),
        );
        return static_method(
            vec![],
            result.clone(),
            value.name(),
            vec![
                parameter(operand.clone(), "left"),
                parameter(amount, "right"),
            ],
            vec![
                JavaStmt::If {
                    condition: invalid,
                    then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                        result.clone(),
                        JavaRuntimeFailure::InvalidShift,
                    )))]),
                    else_block: None,
                },
                JavaStmt::Return(Some(runtime_ok(result, shifted))),
            ],
        );
    }

    if matches!(
        value,
        JavaRuntimeCallable::CheckedDivI32
            | JavaRuntimeCallable::CheckedDivI64
            | JavaRuntimeCallable::CheckedRemI32
            | JavaRuntimeCallable::CheckedRemI64
    ) {
        let left = local(operand.clone(), "left");
        let right = local(operand.clone(), "right");
        let zero = if wide {
            long_literal(0)
        } else {
            int_literal(0)
        };
        let minus_one = if wide {
            long_literal(-1)
        } else {
            int_literal(-1)
        };
        let minimum = known_field(if wide {
            JavaKnownField::LongMinValue
        } else {
            JavaKnownField::IntegerMinValue
        });
        let division = matches!(
            value,
            JavaRuntimeCallable::CheckedDivI32 | JavaRuntimeCallable::CheckedDivI64
        );
        let mut statements = vec![JavaStmt::If {
            condition: binary(
                JavaBinaryOperator::Equal,
                right.clone(),
                zero,
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                result.clone(),
                if division {
                    JavaRuntimeFailure::DivisionByZero
                } else {
                    JavaRuntimeFailure::RemainderByZero
                },
            )))]),
            else_block: None,
        }];
        if division {
            statements.push(JavaStmt::If {
                condition: binary(
                    JavaBinaryOperator::LogicalAnd,
                    binary(
                        JavaBinaryOperator::Equal,
                        left.clone(),
                        minimum,
                        JavaType::primitive(JavaPrimitive::Boolean),
                    ),
                    binary(
                        JavaBinaryOperator::Equal,
                        right.clone(),
                        minus_one,
                        JavaType::primitive(JavaPrimitive::Boolean),
                    ),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                    result.clone(),
                    JavaRuntimeFailure::CheckedOverflow,
                )))]),
                else_block: None,
            });
        }
        statements.push(JavaStmt::Return(Some(runtime_ok(
            result.clone(),
            binary(
                if division {
                    JavaBinaryOperator::Divide
                } else {
                    JavaBinaryOperator::Remainder
                },
                left,
                right,
                operand.clone(),
            ),
        ))));
        return static_method(
            vec![],
            result,
            value.name(),
            vec![
                parameter(operand.clone(), "left"),
                parameter(operand, "right"),
            ],
            statements,
        );
    }

    let unary_operation = matches!(
        value,
        JavaRuntimeCallable::CheckedNegI32 | JavaRuntimeCallable::CheckedNegI64
    );
    let result_expression = if wide {
        let left = known_call(
            JavaKnownCallable::BigIntegerValueOf,
            vec![local(long.clone(), "value")],
        );
        if unary_operation {
            known_method_call(
                JavaKnownMethod::BigIntegerNegate,
                left,
                vec![],
                bigint.clone(),
            )
        } else {
            let right = known_call(
                JavaKnownCallable::BigIntegerValueOf,
                vec![local(long.clone(), "right")],
            );
            known_method_call(
                match value {
                    JavaRuntimeCallable::CheckedAddI64 => JavaKnownMethod::BigIntegerAdd,
                    JavaRuntimeCallable::CheckedSubI64 => JavaKnownMethod::BigIntegerSubtract,
                    JavaRuntimeCallable::CheckedMulI64 => JavaKnownMethod::BigIntegerMultiply,
                    _ => unreachable!(),
                },
                known_call(
                    JavaKnownCallable::BigIntegerValueOf,
                    vec![local(long.clone(), "left")],
                ),
                vec![right],
                bigint.clone(),
            )
        }
    } else {
        let long_left = cast(
            long.clone(),
            local(int.clone(), if unary_operation { "value" } else { "left" }),
        );
        if unary_operation {
            unary(JavaUnaryOperator::Negate, long_left, long.clone())
        } else {
            binary(
                match value {
                    JavaRuntimeCallable::CheckedAddI32 => JavaBinaryOperator::Add,
                    JavaRuntimeCallable::CheckedSubI32 => JavaBinaryOperator::Subtract,
                    JavaRuntimeCallable::CheckedMulI32 => JavaBinaryOperator::Multiply,
                    _ => unreachable!(),
                },
                long_left,
                cast(long.clone(), local(int.clone(), "right")),
                long.clone(),
            )
        }
    };
    let statements = if wide {
        checked_bigint_statements(result_expression, result.clone(), long.clone())
    } else {
        checked_bounds_statements(
            result_expression,
            result.clone(),
            JavaKnownField::IntegerMinValue,
            JavaKnownField::IntegerMaxValue,
            int.clone(),
            JavaRuntimeFailure::CheckedOverflow,
        )
    };
    let parameters = if unary_operation {
        vec![parameter(operand, "value")]
    } else {
        vec![
            parameter(operand.clone(), "left"),
            parameter(operand, "right"),
        ]
    };
    static_method(vec![], result, value.name(), parameters, statements)
}

fn checked_bounds_statements(
    candidate: JavaExpr,
    result: JavaType,
    minimum: JavaKnownField,
    maximum: JavaKnownField,
    output: JavaType,
    failure: JavaRuntimeFailure,
) -> Vec<JavaStmt> {
    let candidate_type = candidate.ty.clone();
    vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: candidate_type.clone(),
            name: identifier("candidate"),
            value: Some(candidate),
        },
        JavaStmt::If {
            condition: binary(
                JavaBinaryOperator::LogicalOr,
                binary(
                    JavaBinaryOperator::Less,
                    local(candidate_type.clone(), "candidate"),
                    cast(candidate_type.clone(), known_field(minimum)),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                binary(
                    JavaBinaryOperator::Greater,
                    local(candidate_type.clone(), "candidate"),
                    cast(candidate_type.clone(), known_field(maximum)),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                result.clone(),
                failure,
            )))]),
            else_block: None,
        },
        JavaStmt::Return(Some(runtime_ok(
            result,
            cast(output, local(candidate_type, "candidate")),
        ))),
    ]
}

fn checked_bigint_statements(
    candidate: JavaExpr,
    result: JavaType,
    output: JavaType,
) -> Vec<JavaStmt> {
    let bigint = JavaType::known(JavaKnownType::BigInteger);
    let minimum = known_call(
        JavaKnownCallable::BigIntegerValueOf,
        vec![known_field(JavaKnownField::LongMinValue)],
    );
    let maximum = known_call(
        JavaKnownCallable::BigIntegerValueOf,
        vec![known_field(JavaKnownField::LongMaxValue)],
    );
    vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: bigint.clone(),
            name: identifier("candidate"),
            value: Some(candidate),
        },
        JavaStmt::If {
            condition: binary(
                JavaBinaryOperator::LogicalOr,
                binary(
                    JavaBinaryOperator::Less,
                    known_method_call(
                        JavaKnownMethod::BigIntegerCompareTo,
                        local(bigint.clone(), "candidate"),
                        vec![minimum],
                        JavaType::primitive(JavaPrimitive::Int),
                    ),
                    int_literal(0),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                binary(
                    JavaBinaryOperator::Greater,
                    known_method_call(
                        JavaKnownMethod::BigIntegerCompareTo,
                        local(bigint.clone(), "candidate"),
                        vec![maximum],
                        JavaType::primitive(JavaPrimitive::Int),
                    ),
                    int_literal(0),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                result.clone(),
                JavaRuntimeFailure::CheckedOverflow,
            )))]),
            else_block: None,
        },
        JavaStmt::Return(Some(runtime_ok(
            result,
            known_method_call(
                JavaKnownMethod::BigIntegerLongValue,
                local(bigint, "candidate"),
                vec![],
                output,
            ),
        ))),
    ]
}

fn float_method(value: JavaRuntimeCallable) -> JavaMember {
    let double = JavaType::primitive(JavaPrimitive::Double);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let operand = local(double.clone(), "value");
    let returned = match value {
        JavaRuntimeCallable::FloatTrunc => conditional(
            binary(
                JavaBinaryOperator::Greater,
                operand.clone(),
                double_literal(0),
                boolean.clone(),
            ),
            known_call(JavaKnownCallable::MathFloor, vec![operand.clone()]),
            known_call(JavaKnownCallable::MathCeil, vec![operand.clone()]),
            double.clone(),
        ),
        JavaRuntimeCallable::FloatIsNegativeZero => binary(
            JavaBinaryOperator::Equal,
            known_call(
                JavaKnownCallable::DoubleToRawLongBits,
                vec![operand.clone()],
            ),
            known_field(JavaKnownField::LongMinValue),
            boolean.clone(),
        ),
        JavaRuntimeCallable::FloatAbs => known_call(
            JavaKnownCallable::DoubleFromLongBits,
            vec![binary(
                JavaBinaryOperator::BitAnd,
                known_call(JavaKnownCallable::DoubleToRawLongBits, vec![operand]),
                known_field(JavaKnownField::LongMaxValue),
                JavaType::primitive(JavaPrimitive::Long),
            )],
        ),
        _ => unreachable!(),
    };
    static_method(
        vec![],
        returned.ty.clone(),
        value.name(),
        vec![parameter(double, "value")],
        vec![JavaStmt::Return(Some(returned))],
    )
}

fn list_method(value: JavaRuntimeCallable) -> JavaMember {
    let t = type_variable("T");
    let list = generic(JavaKnownType::List, vec![t.clone()]);
    let array_list = generic(JavaKnownType::ArrayList, vec![t.clone()]);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let long = JavaType::primitive(JavaPrimitive::Long);
    let option_long = generic(
        JavaKnownType::RuntimeOption,
        vec![JavaType::Boxed(JavaPrimitive::Long)],
    );
    let result_t = generic(JavaKnownType::RuntimeResult, vec![t.clone()]);
    let values = local(list.clone(), "values");
    match value {
        JavaRuntimeCallable::ListCopy => static_method(
            vec![identifier("T")],
            list.clone(),
            value.name(),
            vec![parameter(list.clone(), "values")],
            vec![JavaStmt::Return(Some(known_generic_call(
                JavaKnownCallable::ListCopyOf,
                vec![values],
                list,
            )))],
        ),
        JavaRuntimeCallable::ListLength => static_method(
            vec![identifier("T")],
            long.clone(),
            value.name(),
            vec![parameter(list.clone(), "values")],
            vec![JavaStmt::Return(Some(cast(
                long,
                known_method_call(JavaKnownMethod::ListSize, values, vec![], int),
            )))],
        ),
        JavaRuntimeCallable::ListIsEmpty => static_method(
            vec![identifier("T")],
            boolean.clone(),
            value.name(),
            vec![parameter(list.clone(), "values")],
            vec![JavaStmt::Return(Some(known_method_call(
                JavaKnownMethod::ListIsEmpty,
                values,
                vec![],
                boolean,
            )))],
        ),
        JavaRuntimeCallable::ListGet => {
            let index = local(long.clone(), "index");
            let size = known_method_call(
                JavaKnownMethod::ListSize,
                values.clone(),
                vec![],
                int.clone(),
            );
            let out_of_bounds = binary(
                JavaBinaryOperator::LogicalOr,
                binary(
                    JavaBinaryOperator::Less,
                    index.clone(),
                    long_literal(0),
                    boolean.clone(),
                ),
                binary(
                    JavaBinaryOperator::GreaterEqual,
                    index.clone(),
                    cast(long.clone(), size),
                    boolean.clone(),
                ),
                boolean,
            );
            static_method(
                vec![identifier("T")],
                result_t.clone(),
                value.name(),
                vec![parameter(list.clone(), "values"), parameter(long, "index")],
                vec![
                    JavaStmt::If {
                        condition: out_of_bounds,
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                            result_t.clone(),
                            JavaRuntimeFailure::IndexOutOfBounds,
                        )))]),
                        else_block: None,
                    },
                    JavaStmt::Return(Some(runtime_ok(
                        result_t,
                        known_method_call(
                            JavaKnownMethod::ListGet,
                            values,
                            vec![cast(int, index)],
                            t,
                        ),
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::ListAppend | JavaRuntimeCallable::ListConcat => {
            let second_name = if value == JavaRuntimeCallable::ListAppend {
                "item"
            } else {
                "right"
            };
            let second_type = if value == JavaRuntimeCallable::ListAppend {
                t.clone()
            } else {
                list.clone()
            };
            let mutating_method = if value == JavaRuntimeCallable::ListAppend {
                JavaKnownMethod::ArrayListAdd
            } else {
                JavaKnownMethod::ArrayListAddAll
            };
            static_method(
                vec![identifier("T")],
                list.clone(),
                value.name(),
                vec![
                    parameter(
                        list.clone(),
                        if value == JavaRuntimeCallable::ListAppend {
                            "values"
                        } else {
                            "left"
                        },
                    ),
                    parameter(second_type.clone(), second_name),
                ],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: array_list.clone(),
                        name: identifier("result"),
                        value: Some(new_known(
                            JavaKnownConstructor::ArrayListFromList,
                            array_list.clone(),
                            vec![local(
                                list.clone(),
                                if value == JavaRuntimeCallable::ListAppend {
                                    "values"
                                } else {
                                    "left"
                                },
                            )],
                        )),
                    },
                    JavaStmt::Expression(known_method_call(
                        mutating_method,
                        local(array_list.clone(), "result"),
                        vec![local(second_type, second_name)],
                        boolean.clone(),
                    )),
                    JavaStmt::Return(Some(known_generic_call(
                        JavaKnownCallable::ListCopyOf,
                        vec![local(array_list, "result")],
                        list.clone(),
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::ListContains | JavaRuntimeCallable::ListIndexOf => {
            let index = local(long.clone(), "index");
            let candidate = local(t.clone(), "candidate");
            let matches = runtime_call(
                JavaRuntimeCallable::SemanticEqual,
                vec![candidate, local(t.clone(), "item")],
                boolean.clone(),
            );
            let found = if value == JavaRuntimeCallable::ListContains {
                bool_literal(true)
            } else {
                runtime_call(
                    JavaRuntimeCallable::OptionSome,
                    vec![index.clone()],
                    option_long.clone(),
                )
            };
            let mut body = vec![JavaStmt::If {
                condition: matches,
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(found))]),
                else_block: None,
            }];
            if value == JavaRuntimeCallable::ListIndexOf {
                body.push(JavaStmt::Assign {
                    target: index.clone(),
                    value: binary(
                        JavaBinaryOperator::Add,
                        index,
                        long_literal(1),
                        long.clone(),
                    ),
                });
            }
            let return_type = if value == JavaRuntimeCallable::ListContains {
                boolean.clone()
            } else {
                option_long.clone()
            };
            let mut statements = Vec::new();
            if value == JavaRuntimeCallable::ListIndexOf {
                statements.push(JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty: long,
                    name: identifier("index"),
                    value: Some(long_literal(0)),
                });
            }
            statements.push(JavaStmt::ForEach {
                binding_type: t.clone(),
                binding: identifier("candidate"),
                iterable: values,
                body: JavaBlock::new(body),
            });
            statements.push(JavaStmt::Return(Some(
                if value == JavaRuntimeCallable::ListContains {
                    bool_literal(false)
                } else {
                    runtime_call(JavaRuntimeCallable::OptionNone, vec![], option_long)
                },
            )));
            static_method(
                vec![identifier("T")],
                return_type,
                value.name(),
                vec![parameter(list, "values"), parameter(t, "item")],
                statements,
            )
        }
        _ => unreachable!(),
    }
}

fn bytes_method(value: JavaRuntimeCallable) -> JavaMember {
    let integer = JavaType::Boxed(JavaPrimitive::Int);
    let list = generic(JavaKnownType::List, vec![integer.clone()]);
    let array_list = generic(JavaKnownType::ArrayList, vec![integer.clone()]);
    let bytes = JavaType::known(JavaKnownType::RuntimeBytes);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let long = JavaType::primitive(JavaPrimitive::Long);
    match value {
        JavaRuntimeCallable::BytesLength => static_method(
            vec![],
            long.clone(),
            value.name(),
            vec![parameter(bytes.clone(), "value")],
            vec![JavaStmt::Return(Some(cast(
                long,
                known_method_call(
                    JavaKnownMethod::ListSize,
                    bytes_values(local(bytes, "value"), list),
                    vec![],
                    int,
                ),
            )))],
        ),
        JavaRuntimeCallable::BytesIsEmpty => static_method(
            vec![],
            boolean.clone(),
            value.name(),
            vec![parameter(bytes.clone(), "value")],
            vec![JavaStmt::Return(Some(known_method_call(
                JavaKnownMethod::ListIsEmpty,
                bytes_values(local(bytes, "value"), list),
                vec![],
                boolean,
            )))],
        ),
        JavaRuntimeCallable::BytesConcat => static_method(
            vec![],
            bytes.clone(),
            value.name(),
            vec![
                parameter(bytes.clone(), "left"),
                parameter(bytes.clone(), "right"),
            ],
            vec![JavaStmt::Return(Some(runtime_call(
                JavaRuntimeCallable::BytesOf,
                vec![runtime_call(
                    JavaRuntimeCallable::ListConcat,
                    vec![
                        bytes_values(local(bytes.clone(), "left"), list.clone()),
                        bytes_values(local(bytes, "right"), list.clone()),
                    ],
                    list,
                )],
                JavaType::known(JavaKnownType::RuntimeBytes),
            )))],
        ),
        JavaRuntimeCallable::BytesReplaceAll => {
            let source = bytes_values(local(bytes.clone(), "source"), list.clone());
            let needle = bytes_values(local(bytes.clone(), "needle"), list.clone());
            let replacement = bytes_values(local(bytes.clone(), "replacement"), list.clone());
            let result = local(array_list.clone(), "result");
            let offset = local(int.clone(), "offset");
            let needle_size = known_method_call(
                JavaKnownMethod::ListSize,
                needle.clone(),
                vec![],
                int.clone(),
            );
            let source_size = known_method_call(
                JavaKnownMethod::ListSize,
                source.clone(),
                vec![],
                int.clone(),
            );
            let empty_needle_body = vec![
                JavaStmt::Expression(known_method_call(
                    JavaKnownMethod::ArrayListAddAll,
                    result.clone(),
                    vec![replacement.clone()],
                    boolean.clone(),
                )),
                JavaStmt::ForEach {
                    binding_type: integer.clone(),
                    binding: identifier("item"),
                    iterable: source.clone(),
                    body: JavaBlock::new(vec![
                        JavaStmt::Expression(known_method_call(
                            JavaKnownMethod::ArrayListAdd,
                            result.clone(),
                            vec![local(integer.clone(), "item")],
                            boolean.clone(),
                        )),
                        JavaStmt::Expression(known_method_call(
                            JavaKnownMethod::ArrayListAddAll,
                            result.clone(),
                            vec![replacement.clone()],
                            boolean.clone(),
                        )),
                    ]),
                },
                JavaStmt::Return(Some(runtime_call(
                    JavaRuntimeCallable::BytesOf,
                    vec![result.clone()],
                    bytes.clone(),
                ))),
            ];
            let enough = binary(
                JavaBinaryOperator::LessEqual,
                binary(
                    JavaBinaryOperator::Add,
                    offset.clone(),
                    needle_size.clone(),
                    int.clone(),
                ),
                source_size.clone(),
                boolean.clone(),
            );
            let slice = known_method_call(
                JavaKnownMethod::ListSubList,
                source.clone(),
                vec![
                    offset.clone(),
                    binary(
                        JavaBinaryOperator::Add,
                        offset.clone(),
                        needle_size.clone(),
                        int.clone(),
                    ),
                ],
                list.clone(),
            );
            let equal = known_call(
                JavaKnownCallable::ObjectsDeepEquals,
                vec![slice, needle.clone()],
            );
            let matches = binary(
                JavaBinaryOperator::LogicalAnd,
                enough,
                equal,
                boolean.clone(),
            );
            static_method(
                vec![],
                bytes.clone(),
                value.name(),
                vec![
                    parameter(bytes.clone(), "source"),
                    parameter(bytes.clone(), "needle"),
                    parameter(bytes.clone(), "replacement"),
                ],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: array_list.clone(),
                        name: identifier("result"),
                        value: Some(new_known(
                            JavaKnownConstructor::ArrayList,
                            array_list.clone(),
                            vec![],
                        )),
                    },
                    JavaStmt::If {
                        condition: known_method_call(
                            JavaKnownMethod::ListIsEmpty,
                            needle.clone(),
                            vec![],
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(empty_needle_body),
                        else_block: None,
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Mutable,
                        ty: int.clone(),
                        name: identifier("offset"),
                        value: Some(int_literal(0)),
                    },
                    JavaStmt::While {
                        condition: binary(
                            JavaBinaryOperator::Less,
                            offset.clone(),
                            source_size,
                            boolean.clone(),
                        ),
                        body: JavaBlock::new(vec![JavaStmt::If {
                            condition: matches,
                            then_block: JavaBlock::new(vec![
                                JavaStmt::Expression(known_method_call(
                                    JavaKnownMethod::ArrayListAddAll,
                                    result.clone(),
                                    vec![replacement],
                                    boolean.clone(),
                                )),
                                JavaStmt::Assign {
                                    target: offset.clone(),
                                    value: binary(
                                        JavaBinaryOperator::Add,
                                        offset.clone(),
                                        needle_size,
                                        int.clone(),
                                    ),
                                },
                            ]),
                            else_block: Some(JavaBlock::new(vec![
                                JavaStmt::Expression(known_method_call(
                                    JavaKnownMethod::ArrayListAdd,
                                    result.clone(),
                                    vec![known_method_call(
                                        JavaKnownMethod::ListGet,
                                        source,
                                        vec![offset.clone()],
                                        integer,
                                    )],
                                    boolean,
                                )),
                                JavaStmt::Assign {
                                    target: offset.clone(),
                                    value: binary(
                                        JavaBinaryOperator::Add,
                                        offset,
                                        int_literal(1),
                                        int,
                                    ),
                                },
                            ])),
                        }]),
                    },
                    JavaStmt::Return(Some(runtime_call(
                        JavaRuntimeCallable::BytesOf,
                        vec![result],
                        bytes,
                    ))),
                ],
            )
        }
        _ => unreachable!(),
    }
}

fn identifier(value: &str) -> JavaIdentifier {
    JavaIdentifier::from_portable(value)
}
fn type_variable(value: &str) -> JavaType {
    JavaType::TypeVariable(identifier(value))
}
fn generic(raw: JavaKnownType, arguments: Vec<JavaType>) -> JavaType {
    JavaType::generic(raw, arguments)
}
fn component(ty: JavaType, name: &str, runtime_member: JavaRuntimeMember) -> JavaRecordComponent {
    JavaRecordComponent {
        origin: JavaRecordComponentOrigin::Runtime(runtime_member),
        ty,
        name: identifier(name),
    }
}
fn parameter(ty: JavaType, name: &str) -> JavaParameter {
    JavaParameter {
        ty,
        name: identifier(name),
        final_parameter: true,
    }
}
fn record(
    owner: JavaKnownType,
    name: &str,
    type_parameters: Vec<JavaIdentifier>,
    record_components: Vec<JavaRecordComponent>,
) -> JavaTypeDeclaration {
    let self_type = if type_parameters.is_empty() {
        JavaType::known(owner)
    } else {
        generic(
            owner,
            type_parameters
                .iter()
                .cloned()
                .map(JavaType::TypeVariable)
                .collect(),
        )
    };
    let comparison_type = if type_parameters.is_empty() {
        JavaType::known(owner)
    } else {
        generic(
            owner,
            type_parameters
                .iter()
                .map(|_| JavaType::Wildcard { bound: None })
                .collect(),
        )
    };
    let semantic = semantic_method(self_type, comparison_type, &record_components);
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier(name),
        type_parameters,
        record_components,
        heritage: JavaHeritage::Interfaces(vec![JavaType::known(
            JavaKnownType::RuntimeSemanticValue,
        )]),
        permits: vec![],
        members: vec![semantic],
    }
}

fn semantic_method(
    self_type: JavaType,
    comparison_type: JavaType,
    components: &[JavaRecordComponent],
) -> JavaMember {
    let object = JavaType::known(JavaKnownType::Object);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let other = local(comparison_type.clone(), "otherValue");
    let this = JavaExpr {
        ty: self_type,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::This),
    };
    let equal = components
        .iter()
        .fold(bool_literal(true), |equal, component| {
            binary(
                JavaBinaryOperator::LogicalAnd,
                equal,
                runtime_call(
                    JavaRuntimeCallable::SemanticEqual,
                    vec![
                        member_call(
                            this.clone(),
                            match component.origin {
                                JavaRecordComponentOrigin::Runtime(member) => member,
                                JavaRecordComponentOrigin::Core(_) => {
                                    unreachable!("runtime semantic records have runtime components")
                                }
                            },
                            vec![],
                            component.ty.clone(),
                        ),
                        member_call(
                            other.clone(),
                            match component.origin {
                                JavaRecordComponentOrigin::Runtime(member) => member,
                                JavaRecordComponentOrigin::Core(_) => {
                                    unreachable!("runtime semantic records have runtime components")
                                }
                            },
                            vec![],
                            component.ty.clone(),
                        ),
                    ],
                    boolean.clone(),
                ),
                boolean.clone(),
            )
        });
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![JavaAnnotation::Override],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: boolean.clone(),
        name: identifier("semanticEquals"),
        parameters: vec![parameter(object.clone(), "other")],
        body: Some(JavaBlock::new(vec![
            JavaStmt::If {
                condition: unary(
                    JavaUnaryOperator::Not,
                    instance_of(
                        local(object, "other"),
                        comparison_type,
                        Some(identifier("otherValue")),
                    ),
                    boolean.clone(),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(bool_literal(false)))]),
                else_block: None,
            },
            JavaStmt::Return(Some(equal)),
        ])),
    })
}
fn static_method(
    type_parameters: Vec<JavaIdentifier>,
    return_type: JavaType,
    name: &str,
    parameters: Vec<JavaParameter>,
    statements: Vec<JavaStmt>,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters,
        return_type,
        name: identifier(name),
        parameters,
        body: Some(JavaBlock::new(statements)),
    })
}
fn local(ty: JavaType, name: &str) -> JavaExpr {
    JavaExpr::local(ty, identifier(name))
}
fn unary(operator: JavaUnaryOperator, operand: JavaExpr, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator,
            operand: Box::new(operand),
        },
    }
}
fn binary(operator: JavaBinaryOperator, left: JavaExpr, right: JavaExpr, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: match operator {
            JavaBinaryOperator::LogicalOr => JavaPrecedence::LogicalOr,
            JavaBinaryOperator::LogicalAnd => JavaPrecedence::LogicalAnd,
            JavaBinaryOperator::BitOr => JavaPrecedence::BitOr,
            JavaBinaryOperator::BitXor => JavaPrecedence::BitXor,
            JavaBinaryOperator::BitAnd => JavaPrecedence::BitAnd,
            JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual => JavaPrecedence::Equality,
            JavaBinaryOperator::Less
            | JavaBinaryOperator::LessEqual
            | JavaBinaryOperator::Greater
            | JavaBinaryOperator::GreaterEqual => JavaPrecedence::Relational,
            JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => JavaPrecedence::Shift,
            JavaBinaryOperator::Add | JavaBinaryOperator::Subtract => JavaPrecedence::Additive,
            JavaBinaryOperator::Multiply
            | JavaBinaryOperator::Divide
            | JavaBinaryOperator::Remainder => JavaPrecedence::Multiplicative,
        },
        kind: JavaExprKind::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
    }
}
fn conditional(
    condition: JavaExpr,
    when_true: JavaExpr,
    when_false: JavaExpr,
    ty: JavaType,
) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        },
    }
}
fn cast(target: JavaType, value: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: target.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target,
            value: Box::new(value),
        },
    }
}
fn instance_of(value: JavaExpr, target: JavaType, binding: Option<JavaIdentifier>) -> JavaExpr {
    JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Boolean),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(value),
            target,
            binding,
        },
    }
}
fn new_array(component: JavaType, length: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: JavaType::Array {
            component: Box::new(component.clone()),
            ownership: JavaArrayOwnership::InternalMutable,
        },
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::NewArray {
            component,
            length: Box::new(length),
        },
    }
}
fn array_index(array: JavaExpr, index: JavaExpr, component: JavaType) -> JavaExpr {
    JavaExpr {
        ty: component,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::ArrayIndex {
            array: Box::new(array),
            index: Box::new(index),
        },
    }
}
fn structural_field(receiver: JavaExpr, name: &str, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty: ty.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Field {
            receiver: Box::new(receiver),
            field: JavaFieldRef::Structural {
                name: identifier(name),
                ty,
            },
        },
    }
}
fn this_value(ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::This),
    }
}
fn assign_component(owner: JavaType, name: &str, ty: JavaType, value: JavaExpr) -> JavaStmt {
    JavaStmt::Assign {
        target: structural_field(this_value(owner), name, ty),
        value,
    }
}
fn illegal_argument(message: &str) -> JavaStmt {
    JavaStmt::Throw(new_known(
        JavaKnownConstructor::IllegalArgumentExceptionString,
        JavaType::known(JavaKnownType::IllegalArgumentException),
        vec![string_literal(message)],
    ))
}
fn illegal_state(message: &str) -> JavaStmt {
    JavaStmt::Throw(new_known(
        JavaKnownConstructor::IllegalStateExceptionString,
        JavaType::known(JavaKnownType::IllegalStateException),
        vec![string_literal(message)],
    ))
}
fn guarded_accessor(
    owner: JavaType,
    name: &str,
    ty: JavaType,
    invalid: JavaExpr,
    message: &str,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: ty.clone(),
        name: identifier(name),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![
            JavaStmt::If {
                condition: invalid,
                then_block: JavaBlock::new(vec![illegal_state(message)]),
                else_block: None,
            },
            JavaStmt::Return(Some(structural_field(this_value(owner), name, ty))),
        ])),
    })
}
fn bytes_values(receiver: JavaExpr, list_type: JavaType) -> JavaExpr {
    member_call(receiver, JavaRuntimeMember::BytesValues, vec![], list_type)
}
fn bool_literal(value: bool) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Boolean),
        JavaLiteral::Boolean(value),
    )
}
fn int_literal(value: i32) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Int),
        JavaLiteral::I32(value),
    )
}
fn long_literal(value: i64) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Long),
        JavaLiteral::I64(value),
    )
}
fn double_literal(bits: u64) -> JavaExpr {
    known_call(
        JavaKnownCallable::DoubleFromLongBits,
        vec![long_literal(bits as i64)],
    )
}
fn string_literal(value: &str) -> JavaExpr {
    JavaExpr::literal(
        JavaType::known(JavaKnownType::String),
        JavaLiteral::String(value.to_owned()),
    )
}
fn null_literal(ty: JavaType) -> JavaExpr {
    JavaExpr::literal(
        ty,
        JavaLiteral::InternalNull(JavaNullPurpose::AbsentTaggedPayload),
    )
}
fn known_call(callable: JavaKnownCallable, arguments: Vec<JavaExpr>) -> JavaExpr {
    let expected = callable.signature();
    let signature = JavaMethodSignature {
        receiver: None,
        parameters: arguments
            .iter()
            .map(|argument| argument.ty.clone())
            .collect(),
        result: expected.result,
        checked_exceptions: expected.checked_exceptions,
        nullable_result: expected.nullable_result,
        pure: expected.pure,
    };
    JavaExpr {
        ty: signature.result.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}
fn known_field(field: JavaKnownField) -> JavaExpr {
    JavaExpr {
        ty: field.ty(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::KnownField(field)),
    }
}
fn known_method_call(
    method: JavaKnownMethod,
    receiver: JavaExpr,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let expected = method.signature();
    let signature = JavaMethodSignature {
        receiver: Some(receiver.ty.clone()),
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: expected.checked_exceptions,
        nullable_result: expected.nullable_result,
        pure: expected.pure,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: receiver.ty.clone(),
                name: identifier(method.name().text()),
                signature,
                origin: JavaMemberOrigin::Known(method),
            },
            receiver: Some(Box::new(receiver)),
            arguments,
        },
    }
}
fn known_generic_call(
    callable: JavaKnownCallable,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: None,
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}
fn runtime_call(
    callable: JavaRuntimeCallable,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: None,
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Runtime {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JavaRuntimeFailure {
    CheckedOverflow,
    DivisionByZero,
    IndexOutOfBounds,
    InvalidShift,
    InvalidUnicodeScalar,
    InvalidUtf8,
    NarrowingOutOfRange,
    RemainderByZero,
}

impl JavaRuntimeFailure {
    const fn name(self) -> &'static str {
        match self {
            Self::CheckedOverflow => "checked_overflow",
            Self::DivisionByZero => "division_by_zero",
            Self::IndexOutOfBounds => "index_out_of_bounds",
            Self::InvalidShift => "invalid_shift",
            Self::InvalidUnicodeScalar => "invalid_unicode_scalar",
            Self::InvalidUtf8 => "invalid_utf8",
            Self::NarrowingOutOfRange => "narrowing_out_of_range",
            Self::RemainderByZero => "remainder_by_zero",
        }
    }
}

fn runtime_fail(result: JavaType, failure: JavaRuntimeFailure) -> JavaExpr {
    runtime_call(
        JavaRuntimeCallable::Fail,
        vec![
            string_literal(failure.name()),
            string_literal(failure.name()),
        ],
        result,
    )
}
fn runtime_ok(result: JavaType, value: JavaExpr) -> JavaExpr {
    runtime_call(JavaRuntimeCallable::Ok, vec![value], result)
}
fn new_known(
    constructor: JavaKnownConstructor,
    owner: JavaType,
    arguments: Vec<JavaExpr>,
) -> JavaExpr {
    JavaExpr {
        ty: owner.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Known {
                constructor,
                owner,
                parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
            },
            arguments,
        },
    }
}
fn member_call(
    receiver: JavaExpr,
    member: JavaRuntimeMember,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: Some(receiver.ty.clone()),
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: receiver.ty.clone(),
                name: identifier(member.name()),
                signature,
                origin: JavaMemberOrigin::Runtime(member),
            },
            receiver: Some(Box::new(receiver)),
            arguments,
        },
    }
}
