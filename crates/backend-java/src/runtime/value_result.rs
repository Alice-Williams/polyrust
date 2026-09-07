//! Typed runtime construction: value result.

use super::*;

pub(super) fn validated_value_result_type() -> JavaTypeDeclaration {
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
    let comparison_type = generic(
        JavaKnownType::RuntimeValueResult,
        vec![
            JavaType::Wildcard { bound: None },
            JavaType::Wildcard { bound: None },
        ],
    );
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier("PolyValueResult"),
        type_parameters: vec![identifier("T"), identifier("E")],
        record_components: vec![],
        heritage: JavaHeritage::Interfaces(vec![JavaType::known(
            JavaKnownType::RuntimeSemanticValue,
        )]),
        permits: vec![],
        members: vec![
            private_final_field(boolean.clone(), "ok"),
            private_final_field(t.clone(), "value"),
            private_final_field(e.clone(), "error"),
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Private],
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
                    assign_component(
                        owner.clone(),
                        "error",
                        e.clone(),
                        conditional(
                            ok.clone(),
                            error.clone(),
                            known_generic_call(
                                JavaKnownCallable::ObjectsRequireNonNull,
                                vec![error],
                                e.clone(),
                            ),
                            e.clone(),
                        ),
                    ),
                ]),
            }),
            field_accessor(
                owner.clone(),
                "ok",
                boolean.clone(),
                JavaRuntimeMember::ValueResultOk,
            ),
            guarded_accessor(
                owner.clone(),
                "value",
                t.clone(),
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
                e.clone(),
                structural_field(this_value(owner.clone()), "ok", boolean),
                "cannot read error from Ok",
            ),
            runtime_tagged_equality_method(
                owner.clone(),
                comparison_type.clone(),
                JavaRuntimeMember::ValueResultOk,
                (t.clone(), JavaRuntimeMember::ValueResultValue),
                Some((e.clone(), JavaRuntimeMember::ValueResultError)),
                JavaRuntimeCallable::SemanticEqual,
                JavaRuntimeMember::SemanticEquals,
            ),
            runtime_tagged_equality_method(
                owner,
                comparison_type,
                JavaRuntimeMember::ValueResultOk,
                (t, JavaRuntimeMember::ValueResultValue),
                Some((e, JavaRuntimeMember::ValueResultError)),
                JavaRuntimeCallable::DeepEqual,
                JavaRuntimeMember::DeepEquals,
            ),
        ],
    }
}
