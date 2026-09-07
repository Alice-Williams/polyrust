//! Typed runtime construction: equality.

use super::*;

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
