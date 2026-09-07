//! Typed runtime construction: declaration builders.

use super::*;

pub(super) fn identifier(value: &str) -> JavaIdentifier {
    JavaIdentifier::from_portable(value)
}
pub(super) fn type_variable(value: &str) -> JavaType {
    JavaType::TypeVariable(identifier(value))
}
pub(super) fn generic(raw: JavaKnownType, arguments: Vec<JavaType>) -> JavaType {
    JavaType::generic(raw, arguments)
}
pub(super) fn component(
    ty: JavaType,
    name: &str,
    runtime_member: JavaRuntimeMember,
) -> JavaRecordComponent {
    JavaRecordComponent {
        origin: JavaRecordComponentOrigin::Runtime(runtime_member),
        ty,
        name: identifier(name),
    }
}
pub(super) fn parameter(ty: JavaType, name: &str) -> JavaParameter {
    JavaParameter {
        ty,
        name: identifier(name),
        final_parameter: true,
    }
}
pub(super) fn record(
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

pub(super) fn comparison_component_type(
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
