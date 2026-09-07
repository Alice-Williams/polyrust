//! Typed runtime construction: tagged values.
use super::declaration_builders::{generic, identifier, parameter, type_variable};
use super::member_builders::{package_static_method, static_method};

use super::call_builders::{member_call, new_known};
use super::expression_builders::{bool_literal, local, null_literal};
use super::option::validated_option_type;
use super::value_result::validated_value_result_type;
use crate::ast::{JavaKnownType, JavaMember, JavaPrimitive, JavaRuntimeMember, JavaStmt, JavaType};
use crate::dialect::JavaKnownConstructor;

pub(super) fn tagged_members() -> Vec<JavaMember> {
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
        package_static_method(
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
        package_static_method(
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
        package_static_method(
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
        package_static_method(
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
