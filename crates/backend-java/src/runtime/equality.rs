//! Typed runtime construction: equality.
use super::declaration_builders::{generic, identifier, parameter};
use super::member_builders::static_method;

use super::call_builders::{known_call, known_method_call, member_call, runtime_call};
use super::expression_builders::{
    binary, bool_literal, cast, instance_of, int_literal, local, this_value, unary,
};
use crate::ast::{
    JavaAnnotation, JavaBinaryOperator, JavaBlock, JavaDeclarationKind, JavaExpr, JavaExprKind,
    JavaHeritage, JavaIdentifier, JavaKnownType, JavaLocalFinality, JavaMember, JavaMethod,
    JavaMethodDeclaration, JavaModifier, JavaPrecedence, JavaPrimitive, JavaRecordComponent,
    JavaRecordComponentOrigin, JavaRuntimeMember, JavaStmt, JavaType, JavaTypeDeclaration,
    JavaUnaryOperator, JavaValueRef, JavaVisibility,
};
use crate::dialect::{JavaKnownCallable, JavaKnownMethod, JavaRuntimeCallable};

pub(super) fn record_with_equality(
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
    let semantic = runtime_record_equality_method(
        self_type.clone(),
        comparison_type.clone(),
        &record_components,
        JavaRuntimeCallable::SemanticEqual,
        JavaRuntimeMember::SemanticEquals,
    );
    let deep = runtime_record_equality_method(
        self_type,
        comparison_type,
        &record_components,
        JavaRuntimeCallable::DeepEqual,
        JavaRuntimeMember::DeepEquals,
    );
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
        members: vec![semantic, deep],
    }
}

pub(super) fn equality_dispatch_method(
    callable: JavaRuntimeCallable,
    value_member: JavaRuntimeMember,
    bit_exact_float: bool,
) -> JavaMember {
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
    let float_equal = if bit_exact_float {
        binary(
            JavaBinaryOperator::Equal,
            known_call(
                JavaKnownCallable::DoubleToRawLongBits,
                vec![local(double.clone(), "leftDouble")],
            ),
            known_call(
                JavaKnownCallable::DoubleToRawLongBits,
                vec![local(double.clone(), "rightDouble")],
            ),
            boolean.clone(),
        )
    } else {
        binary(
            JavaBinaryOperator::Equal,
            cast(
                JavaType::primitive(JavaPrimitive::Double),
                local(double.clone(), "leftDouble"),
            ),
            cast(
                JavaType::primitive(JavaPrimitive::Double),
                local(double.clone(), "rightDouble"),
            ),
            boolean.clone(),
        )
    };
    static_method(
        vec![],
        boolean.clone(),
        callable.name(),
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
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(float_equal))]),
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
                                        callable,
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
                    value_member,
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

pub(super) fn runtime_record_equality_method(
    self_type: JavaType,
    comparison_type: JavaType,
    components: &[JavaRecordComponent],
    callable: JavaRuntimeCallable,
    member: JavaRuntimeMember,
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
            let comparison_component =
                comparison_component_type(&component.ty, &this.ty, &comparison_type);
            binary(
                JavaBinaryOperator::LogicalAnd,
                equal,
                runtime_call(
                    callable,
                    vec![
                        cast(
                            object.clone(),
                            member_call(
                                this.clone(),
                                match component.origin {
                                    JavaRecordComponentOrigin::Runtime(member) => member,
                                    JavaRecordComponentOrigin::Core(_) => unreachable!(
                                        "runtime semantic records have runtime components"
                                    ),
                                },
                                vec![],
                                component.ty.clone(),
                            ),
                        ),
                        cast(
                            object.clone(),
                            member_call(
                                other.clone(),
                                match component.origin {
                                    JavaRecordComponentOrigin::Runtime(member) => member,
                                    JavaRecordComponentOrigin::Core(_) => unreachable!(
                                        "runtime semantic records have runtime components"
                                    ),
                                },
                                vec![],
                                comparison_component,
                            ),
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
        name: identifier(member.name()),
        parameters: vec![parameter(object.clone(), "other")],
        body: Some(JavaBlock::new(vec![
            JavaStmt::If {
                condition: unary(
                    JavaUnaryOperator::Not,
                    instance_of(
                        local(object.clone(), "other"),
                        comparison_type.clone(),
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

pub(super) fn runtime_tagged_equality_method(
    self_type: JavaType,
    comparison_type: JavaType,
    tag_member: JavaRuntimeMember,
    active: (JavaType, JavaRuntimeMember),
    inactive: Option<(JavaType, JavaRuntimeMember)>,
    callable: JavaRuntimeCallable,
    member: JavaRuntimeMember,
) -> JavaMember {
    let object = JavaType::known(JavaKnownType::Object);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let this = this_value(self_type.clone());
    let other = local(comparison_type.clone(), "otherValue");
    let this_tag = member_call(this.clone(), tag_member, vec![], boolean.clone());
    let other_tag = member_call(other.clone(), tag_member, vec![], boolean.clone());
    let compare_payload = |payload: &(JavaType, JavaRuntimeMember)| {
        let comparison_payload =
            comparison_component_type(&payload.0, &self_type, &comparison_type);
        runtime_call(
            callable,
            vec![
                cast(
                    object.clone(),
                    member_call(this.clone(), payload.1, vec![], payload.0.clone()),
                ),
                cast(
                    object.clone(),
                    member_call(other.clone(), payload.1, vec![], comparison_payload),
                ),
            ],
            boolean.clone(),
        )
    };
    let inactive_comparison = inactive
        .as_ref()
        .map_or_else(|| bool_literal(true), compare_payload);
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![JavaAnnotation::Override],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: boolean.clone(),
        name: identifier(member.name()),
        parameters: vec![parameter(object.clone(), "other")],
        body: Some(JavaBlock::new(vec![
            JavaStmt::If {
                condition: unary(
                    JavaUnaryOperator::Not,
                    instance_of(
                        local(object.clone(), "other"),
                        comparison_type.clone(),
                        Some(identifier("otherValue")),
                    ),
                    boolean.clone(),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(bool_literal(false)))]),
                else_block: None,
            },
            JavaStmt::If {
                condition: binary(
                    JavaBinaryOperator::NotEqual,
                    this_tag.clone(),
                    other_tag,
                    boolean.clone(),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(bool_literal(false)))]),
                else_block: None,
            },
            JavaStmt::If {
                condition: this_tag,
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(compare_payload(&active)))]),
                else_block: None,
            },
            JavaStmt::Return(Some(inactive_comparison)),
        ])),
    })
}

fn comparison_component_type(
    component: &JavaType,
    self_type: &JavaType,
    comparison_type: &JavaType,
) -> JavaType {
    let JavaType::TypeVariable(component_name) = component else {
        return component.clone();
    };
    let (
        JavaType::Generic {
            arguments: self_arguments,
            ..
        },
        JavaType::Generic {
            arguments: comparison_arguments,
            ..
        },
    ) = (self_type, comparison_type)
    else {
        return component.clone();
    };
    self_arguments
        .iter()
        .zip(comparison_arguments)
        .find_map(|(self_argument, comparison_argument)| match self_argument {
            JavaType::TypeVariable(name) if name == component_name => {
                Some(comparison_argument.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| component.clone())
}
