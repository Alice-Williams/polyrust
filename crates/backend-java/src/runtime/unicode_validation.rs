//! Typed runtime construction: unicode validation.

use super::*;

pub(super) fn require_scalar_string_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let character = JavaType::primitive(JavaPrimitive::Char);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let source = local(string.clone(), "value");
    let length = local(int.clone(), "length");
    let index = local(int.clone(), "index");
    let unit = local(character.clone(), "unit");
    let next = binary(
        JavaBinaryOperator::Add,
        index.clone(),
        int_literal(1),
        int.clone(),
    );
    let invalid_pair = binary(
        JavaBinaryOperator::LogicalOr,
        binary(
            JavaBinaryOperator::GreaterEqual,
            next.clone(),
            length.clone(),
            boolean.clone(),
        ),
        unary(
            JavaUnaryOperator::Not,
            known_call(
                JavaKnownCallable::CharacterIsLowSurrogate,
                vec![known_method_call(
                    JavaKnownMethod::StringCharAt,
                    source.clone(),
                    vec![next],
                    character.clone(),
                )],
            ),
            boolean.clone(),
        ),
        boolean.clone(),
    );
    let throw_invalid = || {
        JavaStmt::Throw(new_known(
            JavaKnownConstructor::IllegalArgumentExceptionString,
            JavaType::known(JavaKnownType::IllegalArgumentException),
            vec![string_literal(
                "string contains an unpaired UTF-16 surrogate",
            )],
        ))
    };
    static_method(
        vec![],
        string.clone(),
        value.name(),
        vec![parameter(string.clone(), "value")],
        vec![
            JavaStmt::Expression(known_generic_call(
                JavaKnownCallable::ObjectsRequireNonNull,
                vec![source.clone()],
                string.clone(),
            )),
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: int.clone(),
                name: identifier("length"),
                value: Some(known_method_call(
                    JavaKnownMethod::StringLength,
                    source.clone(),
                    vec![],
                    int.clone(),
                )),
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
                    length,
                    boolean.clone(),
                ),
                body: JavaBlock::new(vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: character.clone(),
                        name: identifier("unit"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCharAt,
                            source.clone(),
                            vec![index.clone()],
                            character,
                        )),
                    },
                    JavaStmt::If {
                        condition: known_call(
                            JavaKnownCallable::CharacterIsHighSurrogate,
                            vec![unit.clone()],
                        ),
                        then_block: JavaBlock::new(vec![
                            JavaStmt::If {
                                condition: invalid_pair,
                                then_block: JavaBlock::new(vec![throw_invalid()]),
                                else_block: None,
                            },
                            JavaStmt::Assign {
                                target: index.clone(),
                                value: binary(
                                    JavaBinaryOperator::Add,
                                    index.clone(),
                                    int_literal(2),
                                    int.clone(),
                                ),
                            },
                            JavaStmt::Continue,
                        ]),
                        else_block: None,
                    },
                    JavaStmt::If {
                        condition: known_call(
                            JavaKnownCallable::CharacterIsLowSurrogate,
                            vec![unit],
                        ),
                        then_block: JavaBlock::new(vec![throw_invalid()]),
                        else_block: None,
                    },
                    JavaStmt::Assign {
                        target: index.clone(),
                        value: binary(JavaBinaryOperator::Add, index, int_literal(1), int),
                    },
                ]),
            },
            JavaStmt::Return(Some(source)),
        ],
    )
}

pub(super) fn compare_scalar_strings_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let left = local(string.clone(), "left");
    let right = local(string.clone(), "right");
    let left_index = local(int.clone(), "leftIndex");
    let right_index = local(int.clone(), "rightIndex");
    let left_scalar = local(int.clone(), "leftScalar");
    let right_scalar = local(int.clone(), "rightScalar");
    let length = |receiver: JavaExpr| {
        known_method_call(JavaKnownMethod::StringLength, receiver, vec![], int.clone())
    };
    static_method(
        vec![],
        int.clone(),
        value.name(),
        vec![
            parameter(string.clone(), "left"),
            parameter(string.clone(), "right"),
        ],
        vec![
            JavaStmt::Expression(runtime_call(
                JavaRuntimeCallable::RequireScalarString,
                vec![left.clone()],
                string.clone(),
            )),
            JavaStmt::Expression(runtime_call(
                JavaRuntimeCallable::RequireScalarString,
                vec![right.clone()],
                string.clone(),
            )),
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("leftIndex"),
                value: Some(int_literal(0)),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Mutable,
                ty: int.clone(),
                name: identifier("rightIndex"),
                value: Some(int_literal(0)),
            },
            JavaStmt::While {
                condition: binary(
                    JavaBinaryOperator::LogicalAnd,
                    binary(
                        JavaBinaryOperator::Less,
                        left_index.clone(),
                        length(left.clone()),
                        boolean.clone(),
                    ),
                    binary(
                        JavaBinaryOperator::Less,
                        right_index.clone(),
                        length(right.clone()),
                        boolean.clone(),
                    ),
                    boolean.clone(),
                ),
                body: JavaBlock::new(vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("leftScalar"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCodePointAt,
                            left.clone(),
                            vec![left_index.clone()],
                            int.clone(),
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("rightScalar"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCodePointAt,
                            right.clone(),
                            vec![right_index.clone()],
                            int.clone(),
                        )),
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Less,
                            left_scalar.clone(),
                            right_scalar.clone(),
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(int_literal(-1)))]),
                        else_block: None,
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Greater,
                            left_scalar.clone(),
                            right_scalar.clone(),
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(int_literal(1)))]),
                        else_block: None,
                    },
                    JavaStmt::Assign {
                        target: left_index.clone(),
                        value: binary(
                            JavaBinaryOperator::Add,
                            left_index.clone(),
                            known_call(
                                JavaKnownCallable::CharacterCharCount,
                                vec![left_scalar.clone()],
                            ),
                            int.clone(),
                        ),
                    },
                    JavaStmt::Assign {
                        target: right_index.clone(),
                        value: binary(
                            JavaBinaryOperator::Add,
                            right_index.clone(),
                            known_call(JavaKnownCallable::CharacterCharCount, vec![right_scalar]),
                            int.clone(),
                        ),
                    },
                ]),
            },
            JavaStmt::If {
                condition: binary(
                    JavaBinaryOperator::Equal,
                    left_index.clone(),
                    length(left),
                    boolean.clone(),
                ),
                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(conditional(
                    binary(
                        JavaBinaryOperator::Equal,
                        right_index,
                        length(right),
                        boolean,
                    ),
                    int_literal(0),
                    int_literal(-1),
                    int.clone(),
                )))]),
                else_block: None,
            },
            JavaStmt::Return(Some(int_literal(1))),
        ],
    )
}

pub(super) fn string_from_utf8_method(
    value: JavaRuntimeCallable,
    string: JavaType,
    bytes: JavaType,
    public_byte_array: JavaType,
    result_string: JavaType,
) -> JavaMember {
    let decoder = JavaType::known(JavaKnownType::CharsetDecoder);
    let char_buffer = JavaType::known(JavaKnownType::CharBuffer);
    let raw = local(public_byte_array.clone(), "raw");
    let decoder_value = local(decoder.clone(), "decoder");
    let report = known_field(JavaKnownField::CodingErrorReport);
    static_method(
        vec![],
        result_string.clone(),
        value.name(),
        vec![parameter(bytes.clone(), "value")],
        vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: public_byte_array.clone(),
                name: identifier("raw"),
                value: Some(bytes_values(local(bytes, "value"), public_byte_array)),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: decoder.clone(),
                name: identifier("decoder"),
                value: Some(known_method_call(
                    JavaKnownMethod::CharsetNewDecoder,
                    known_field(JavaKnownField::StandardCharsetsUtf8),
                    vec![],
                    decoder.clone(),
                )),
            },
            JavaStmt::Expression(known_method_call(
                JavaKnownMethod::DecoderOnMalformedInput,
                decoder_value.clone(),
                vec![report.clone()],
                decoder.clone(),
            )),
            JavaStmt::Expression(known_method_call(
                JavaKnownMethod::DecoderOnUnmappableCharacter,
                decoder_value.clone(),
                vec![report],
                decoder.clone(),
            )),
            JavaStmt::TryCatch {
                try_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_ok(
                    result_string.clone(),
                    known_method_call(
                        JavaKnownMethod::CharBufferToString,
                        known_method_call(
                            JavaKnownMethod::DecoderDecode,
                            decoder_value,
                            vec![known_call(JavaKnownCallable::ByteBufferWrap, vec![raw])],
                            char_buffer,
                        ),
                        vec![],
                        string,
                    ),
                )))]),
                catches: vec![JavaCatch {
                    exception_type: JavaType::known(JavaKnownType::CharacterCodingException),
                    binding: identifier("failure"),
                    body: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                        result_string,
                        JavaRuntimeFailure::InvalidUtf8,
                    )))]),
                }],
            },
        ],
    )
}
