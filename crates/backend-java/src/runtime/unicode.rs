//! Typed runtime construction: unicode.
use super::declaration_builders::{generic, identifier, parameter};
use super::member_builders::static_method;

use super::call_builders::{
    JavaRuntimeFailure, known_call, known_field, known_method_call, new_known, runtime_call,
    runtime_fail, runtime_ok,
};
use super::expression_builders::{
    binary, cast, conditional, fresh_copy_to_boundary, int_literal, local, long_literal,
    string_literal, unary,
};
use super::unicode_validation::{
    compare_scalar_strings_method, require_scalar_string_method, string_from_utf8_method,
};
use crate::ast::{
    JavaArrayOwnership, JavaBinaryOperator, JavaBlock, JavaKnownType, JavaLocalFinality,
    JavaMember, JavaPrimitive, JavaStmt, JavaType, JavaUnaryOperator,
};
use crate::dialect::{
    JavaKnownCallable, JavaKnownConstructor, JavaKnownField, JavaKnownMethod, JavaRuntimeCallable,
};

pub(super) fn unicode_method(value: JavaRuntimeCallable) -> JavaMember {
    let string = JavaType::known(JavaKnownType::String);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let byte = JavaType::primitive(JavaPrimitive::Byte);
    let character = JavaType::primitive(JavaPrimitive::Char);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let long = JavaType::primitive(JavaPrimitive::Long);
    let bytes = JavaType::known(JavaKnownType::RuntimeBytes);
    let byte_array = JavaType::Array {
        component: Box::new(byte.clone()),
        ownership: JavaArrayOwnership::InternalMutable,
    };
    let public_byte_array = JavaType::Array {
        component: Box::new(byte.clone()),
        ownership: JavaArrayOwnership::DefensiveCopyBoundary,
    };
    let result_i64 = generic(
        JavaKnownType::RuntimeResult,
        vec![JavaType::Boxed(JavaPrimitive::Long)],
    );
    let result_string = generic(JavaKnownType::RuntimeResult, vec![string.clone()]);
    match value {
        JavaRuntimeCallable::RequireScalarString => require_scalar_string_method(value),
        JavaRuntimeCallable::CompareScalarStrings => compare_scalar_strings_method(value),
        JavaRuntimeCallable::ScalarLength => {
            let source = local(string.clone(), "value");
            let length = local(int.clone(), "length");
            let index = local(int.clone(), "index");
            let unit = local(character.clone(), "unit");
            let next_index = binary(
                JavaBinaryOperator::Add,
                index.clone(),
                int_literal(1),
                int.clone(),
            );
            let next_unit = known_method_call(
                JavaKnownMethod::StringCharAt,
                source.clone(),
                vec![next_index.clone()],
                character.clone(),
            );
            let invalid_pair = binary(
                JavaBinaryOperator::LogicalOr,
                binary(
                    JavaBinaryOperator::GreaterEqual,
                    next_index,
                    length.clone(),
                    boolean.clone(),
                ),
                unary(
                    JavaUnaryOperator::Not,
                    known_call(JavaKnownCallable::CharacterIsLowSurrogate, vec![next_unit]),
                    boolean.clone(),
                ),
                boolean.clone(),
            );
            static_method(
                vec![],
                result_i64.clone(),
                value.name(),
                vec![parameter(string.clone(), "value")],
                vec![
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
                            length.clone(),
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
                                    character.clone(),
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
                                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                                            runtime_fail(
                                                result_i64.clone(),
                                                JavaRuntimeFailure::InvalidUnicodeScalar,
                                            ),
                                        ))]),
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
                                then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                                    runtime_fail(
                                        result_i64.clone(),
                                        JavaRuntimeFailure::InvalidUnicodeScalar,
                                    ),
                                ))]),
                                else_block: None,
                            },
                            JavaStmt::Assign {
                                target: index.clone(),
                                value: binary(
                                    JavaBinaryOperator::Add,
                                    index,
                                    int_literal(1),
                                    int.clone(),
                                ),
                            },
                        ]),
                    },
                    JavaStmt::Return(Some(runtime_ok(
                        result_i64,
                        cast(
                            long,
                            known_method_call(
                                JavaKnownMethod::StringCodePointCount,
                                source,
                                vec![int_literal(0), length],
                                int,
                            ),
                        ),
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::StringIndexOfLiteral => {
            let source = local(string.clone(), "source");
            let offset = local(int.clone(), "offset");
            let option_long = generic(
                JavaKnownType::RuntimeOption,
                vec![JavaType::Boxed(JavaPrimitive::Long)],
            );
            static_method(
                vec![],
                option_long.clone(),
                value.name(),
                vec![
                    parameter(string.clone(), "source"),
                    parameter(string.clone(), "needle"),
                ],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("offset"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringIndexOfString,
                            source.clone(),
                            vec![local(string.clone(), "needle")],
                            int.clone(),
                        )),
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::Less,
                            offset.clone(),
                            int_literal(0),
                            boolean.clone(),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_call(
                            JavaRuntimeCallable::OptionNone,
                            vec![],
                            option_long.clone(),
                        )))]),
                        else_block: None,
                    },
                    JavaStmt::Return(Some(runtime_call(
                        JavaRuntimeCallable::OptionSome,
                        vec![cast(
                            long,
                            known_method_call(
                                JavaKnownMethod::StringCodePointCount,
                                source,
                                vec![int_literal(0), offset],
                                int,
                            ),
                        )],
                        option_long,
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::StringSliceScalars => {
            let source = local(string.clone(), "source");
            let scalar_length = local(int.clone(), "scalarLength");
            let scalar_length_long = cast(long.clone(), scalar_length.clone());
            let clamp = |name: &str| {
                let operand = local(long.clone(), name);
                conditional(
                    binary(
                        JavaBinaryOperator::Less,
                        operand.clone(),
                        long_literal(0),
                        boolean.clone(),
                    ),
                    long_literal(0),
                    conditional(
                        binary(
                            JavaBinaryOperator::Greater,
                            operand.clone(),
                            scalar_length_long.clone(),
                            boolean.clone(),
                        ),
                        scalar_length_long.clone(),
                        operand,
                        long.clone(),
                    ),
                    long.clone(),
                )
            };
            let clamped_start = local(long.clone(), "clampedStart");
            let clamped_end = local(long.clone(), "clampedEnd");
            static_method(
                vec![],
                string.clone(),
                value.name(),
                vec![
                    parameter(string.clone(), "source"),
                    parameter(long.clone(), "start"),
                    parameter(long.clone(), "end"),
                ],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: int.clone(),
                        name: identifier("scalarLength"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringCodePointCount,
                            source.clone(),
                            vec![
                                int_literal(0),
                                known_method_call(
                                    JavaKnownMethod::StringLength,
                                    source.clone(),
                                    vec![],
                                    int.clone(),
                                ),
                            ],
                            int.clone(),
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: long.clone(),
                        name: identifier("clampedStart"),
                        value: Some(clamp("start")),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: long.clone(),
                        name: identifier("clampedEnd"),
                        value: Some(clamp("end")),
                    },
                    JavaStmt::If {
                        condition: binary(
                            JavaBinaryOperator::GreaterEqual,
                            clamped_start.clone(),
                            clamped_end.clone(),
                            boolean,
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(string_literal(
                            "",
                        )))]),
                        else_block: None,
                    },
                    JavaStmt::Return(Some(known_method_call(
                        JavaKnownMethod::StringSubstringRange,
                        source.clone(),
                        vec![
                            known_method_call(
                                JavaKnownMethod::StringOffsetByCodePoints,
                                source.clone(),
                                vec![int_literal(0), cast(int.clone(), clamped_start)],
                                int.clone(),
                            ),
                            known_method_call(
                                JavaKnownMethod::StringOffsetByCodePoints,
                                source,
                                vec![int_literal(0), cast(int.clone(), clamped_end)],
                                int,
                            ),
                        ],
                        string,
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::StringToUtf8 => {
            let raw = local(byte_array.clone(), "raw");
            static_method(
                vec![],
                bytes.clone(),
                value.name(),
                vec![parameter(string.clone(), "value")],
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: byte_array.clone(),
                        name: identifier("raw"),
                        value: Some(known_method_call(
                            JavaKnownMethod::StringGetBytes,
                            local(string, "value"),
                            vec![known_field(JavaKnownField::StandardCharsetsUtf8)],
                            byte_array,
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: public_byte_array.clone(),
                        name: identifier("encoded"),
                        value: Some(fresh_copy_to_boundary(raw, public_byte_array.clone())),
                    },
                    JavaStmt::Return(Some(new_known(
                        JavaKnownConstructor::RuntimeBytes,
                        bytes,
                        vec![local(public_byte_array.clone(), "encoded")],
                    ))),
                ],
            )
        }
        JavaRuntimeCallable::StringFromUtf8 => {
            string_from_utf8_method(value, string, bytes, public_byte_array, result_string)
        }
        _ => unreachable!(),
    }
}
