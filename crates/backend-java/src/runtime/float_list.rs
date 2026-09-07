//! Typed runtime construction: float list.

use super::*;

pub(super) fn float_method(value: JavaRuntimeCallable) -> JavaMember {
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

pub(super) fn list_method(value: JavaRuntimeCallable) -> JavaMember {
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

pub(super) fn bytes_to_list_method(value: JavaRuntimeCallable) -> JavaMember {
    let byte = JavaType::primitive(JavaPrimitive::Byte);
    let byte_array = JavaType::Array {
        component: Box::new(byte.clone()),
        ownership: JavaArrayOwnership::DefensiveCopyBoundary,
    };
    let integer = JavaType::Boxed(JavaPrimitive::Int);
    let list = generic(JavaKnownType::List, vec![integer.clone()]);
    let array_list = generic(JavaKnownType::ArrayList, vec![integer]);
    let bytes = JavaType::known(JavaKnownType::RuntimeBytes);
    let raw = local(byte_array.clone(), "raw");
    let output = local(array_list.clone(), "output");
    static_method(
        vec![],
        list.clone(),
        value.name(),
        vec![parameter(bytes.clone(), "value")],
        vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: byte_array.clone(),
                name: identifier("raw"),
                value: Some(bytes_values(local(bytes, "value"), byte_array)),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: array_list.clone(),
                name: identifier("output"),
                value: Some(new_known(
                    JavaKnownConstructor::ArrayList,
                    array_list,
                    vec![],
                )),
            },
            JavaStmt::ForEach {
                binding_type: byte.clone(),
                binding: identifier("item"),
                iterable: raw,
                body: JavaBlock::new(vec![JavaStmt::Expression(known_method_call(
                    JavaKnownMethod::ArrayListAdd,
                    output.clone(),
                    vec![known_call(
                        JavaKnownCallable::ByteToUnsignedInt,
                        vec![local(byte, "item")],
                    )],
                    JavaType::primitive(JavaPrimitive::Boolean),
                ))]),
            },
            JavaStmt::Return(Some(known_generic_call(
                JavaKnownCallable::ListCopyOf,
                vec![output],
                list,
            ))),
        ],
    )
}
