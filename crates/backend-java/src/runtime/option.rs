//! Typed runtime construction: option.
use super::declaration_builders::{generic, identifier, parameter, type_variable};
use super::member_builders::{field_accessor, guarded_accessor, private_final_field};

use super::call_builders::known_generic_call;
use super::equality::runtime_tagged_equality_method;
use super::expression_builders::{
    binary, conditional, local, null_literal, structural_field, this_value, unary,
};
use super::statement_builders::{assign_component, illegal_argument};
use crate::ast::{
    JavaBinaryOperator, JavaBlock, JavaConstructor, JavaDeclarationKind, JavaHeritage,
    JavaKnownType, JavaMember, JavaModifier, JavaPrimitive, JavaRuntimeMember, JavaStmt, JavaType,
    JavaTypeDeclaration, JavaUnaryOperator, JavaVisibility,
};
use crate::dialect::{JavaKnownCallable, JavaRuntimeCallable};

pub(super) fn validated_option_type() -> JavaTypeDeclaration {
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
    let comparison_type = generic(
        JavaKnownType::RuntimeOption,
        vec![JavaType::Wildcard { bound: None }],
    );
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![JavaModifier::Static],
        name: identifier("PolyOption"),
        type_parameters: vec![identifier("T")],
        record_components: vec![],
        heritage: JavaHeritage::Interfaces(vec![JavaType::known(
            JavaKnownType::RuntimeSemanticValue,
        )]),
        permits: vec![],
        members: vec![
            private_final_field(boolean.clone(), "some"),
            private_final_field(t.clone(), "value"),
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Private],
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
                    assign_component(owner.clone(), "some", boolean.clone(), some.clone()),
                    assign_component(
                        owner.clone(),
                        "value",
                        t.clone(),
                        conditional(
                            some.clone(),
                            known_generic_call(
                                JavaKnownCallable::ObjectsRequireNonNull,
                                vec![value.clone()],
                                t.clone(),
                            ),
                            value,
                            t.clone(),
                        ),
                    ),
                ]),
            }),
            field_accessor(
                owner.clone(),
                "some",
                boolean.clone(),
                JavaRuntimeMember::OptionSome,
            ),
            guarded_accessor(
                owner.clone(),
                "value",
                t.clone(),
                unary(
                    JavaUnaryOperator::Not,
                    structural_field(this_value(owner.clone()), "some", boolean.clone()),
                    boolean,
                ),
                "cannot read value from None",
            ),
            runtime_tagged_equality_method(
                owner.clone(),
                comparison_type.clone(),
                JavaRuntimeMember::OptionSome,
                (t.clone(), JavaRuntimeMember::OptionValue),
                None,
                JavaRuntimeCallable::SemanticEqual,
                JavaRuntimeMember::SemanticEquals,
            ),
            runtime_tagged_equality_method(
                owner,
                comparison_type,
                JavaRuntimeMember::OptionSome,
                (t, JavaRuntimeMember::OptionValue),
                None,
                JavaRuntimeCallable::DeepEqual,
                JavaRuntimeMember::DeepEquals,
            ),
        ],
    }
}
