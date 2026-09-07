//! Typed runtime construction: string transform.

use super::*;

pub(super) fn string_truncate_method(value: JavaRuntimeCallable) -> JavaMember {
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

pub(super) fn string_trim_method(value: JavaRuntimeCallable) -> JavaMember {
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
