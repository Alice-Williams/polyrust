//! Typed runtime construction: string replace.

use super::*;

pub(super) fn string_replace_all_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let source = local(string.clone(), "source");
    let needle = local(string.clone(), "needle");
    let replacement = local(string.clone(), "replacement");
    let output = local(string.clone(), "output");
    let offset = local(int.clone(), "offset");
    let width = local(int.clone(), "width");
    static_method(
        vec![],
        string.clone(),
        value.name(),
        vec![
            parameter(string.clone(), "source"),
            parameter(string.clone(), "needle"),
            parameter(string.clone(), "replacement"),
        ],
        vec![
            JavaStmt::If {
                condition: unary(
                    JavaUnaryOperator::Not,
                    known_method_call(
                        JavaKnownMethod::StringIsEmpty,
                        needle.clone(),
                        vec![],
                        boolean.clone(),
                    ),
                    boolean.clone(),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(known_method_call(
                    JavaKnownMethod::StringReplace,
                    source.clone(),
                    vec![needle, replacement.clone()],
                    string.clone(),
                )))]),
                else_block: None,
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: string.clone(),
                name: identifier("output"),
                value: Some(replacement.clone()),
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
                    known_method_call(
                        JavaKnownMethod::StringLength,
                        source.clone(),
                        vec![],
                        int.clone(),
                    ),
                    boolean,
                ),
                body: JavaBlock::new(vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("width"),
                        value: Some(known_call(
                            JavaKnownCallable::CharacterCharCount,
                            vec![known_method_call(
                                JavaKnownMethod::StringCodePointAt,
                                source.clone(),
                                vec![offset.clone()],
                                int.clone(),
                            )],
                        )),
                    },
                    JavaStmt::Assign {
                        target: output.clone(),
                        value: binary(
                            JavaBinaryOperator::Add,
                            binary(
                                JavaBinaryOperator::Add,
                                output.clone(),
                                known_method_call(
                                    JavaKnownMethod::StringSubstringRange,
                                    source.clone(),
                                    vec![
                                        offset.clone(),
                                        binary(
                                            JavaBinaryOperator::Add,
                                            offset.clone(),
                                            width.clone(),
                                            int.clone(),
                                        ),
                                    ],
                                    string.clone(),
                                ),
                                string.clone(),
                            ),
                            replacement,
                            string.clone(),
                        ),
                    },
                    JavaStmt::Assign {
                        target: offset.clone(),
                        value: binary(JavaBinaryOperator::Add, offset, width, int),
                    },
                ]),
            },
            JavaStmt::Return(Some(output)),
        ],
    )
}

pub(super) fn string_replace_many_method(value: JavaRuntimeCallable) -> JavaMember {
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
