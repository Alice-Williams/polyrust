//! Typed runtime construction: bytes operations.

use super::*;

pub(super) fn bytes_method(value: JavaRuntimeCallable) -> JavaMember {
    let integer = JavaType::Boxed(JavaPrimitive::Int);
    let list = generic(JavaKnownType::List, vec![integer.clone()]);
    let array_list = generic(JavaKnownType::ArrayList, vec![integer.clone()]);
    let byte_array = JavaType::Array {
        component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
        ownership: JavaArrayOwnership::DefensiveCopyBoundary,
    };
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
                array_length(bytes_values(local(bytes, "value"), byte_array.clone())),
            )))],
        ),
        JavaRuntimeCallable::BytesIsEmpty => static_method(
            vec![],
            boolean.clone(),
            value.name(),
            vec![parameter(bytes.clone(), "value")],
            vec![JavaStmt::Return(Some(binary(
                JavaBinaryOperator::Equal,
                array_length(bytes_values(local(bytes, "value"), byte_array)),
                int_literal(0),
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
                        runtime_call(
                            JavaRuntimeCallable::BytesToList,
                            vec![local(bytes.clone(), "left")],
                            list.clone(),
                        ),
                        runtime_call(
                            JavaRuntimeCallable::BytesToList,
                            vec![local(bytes, "right")],
                            list.clone(),
                        ),
                    ],
                    list,
                )],
                JavaType::known(JavaKnownType::RuntimeBytes),
            )))],
        ),
        JavaRuntimeCallable::BytesReplaceAll => {
            let source = runtime_call(
                JavaRuntimeCallable::BytesToList,
                vec![local(bytes.clone(), "source")],
                list.clone(),
            );
            let needle = runtime_call(
                JavaRuntimeCallable::BytesToList,
                vec![local(bytes.clone(), "needle")],
                list.clone(),
            );
            let replacement = runtime_call(
                JavaRuntimeCallable::BytesToList,
                vec![local(bytes.clone(), "replacement")],
                list.clone(),
            );
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
