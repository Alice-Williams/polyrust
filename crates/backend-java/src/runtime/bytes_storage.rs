//! Typed runtime construction: bytes storage.
use super::declaration_builders::{component, generic, identifier, parameter};
use super::member_builders::static_method;

use super::call_builders::{known_generic_call, known_method_call, new_known};
use super::dispatch::runtime_method;
use super::equality::runtime_record_equality_method;
use super::expression_builders::{
    array_index, array_length, binary, cast, fresh_copy_to_boundary, int_literal, local, new_array,
    structural_field, this_value,
};
use super::statement_builders::{assign_component, illegal_argument};
use crate::ast::{
    JavaArrayOwnership, JavaBinaryOperator, JavaBlock, JavaConstructor, JavaDeclarationKind,
    JavaHeritage, JavaKnownType, JavaLocalFinality, JavaMember, JavaMethod, JavaMethodDeclaration,
    JavaModifier, JavaPrimitive, JavaRuntimeMember, JavaStmt, JavaType, JavaTypeDeclaration,
    JavaVisibility,
};
use crate::dialect::{
    JavaKnownCallable, JavaKnownConstructor, JavaKnownMethod, JavaRuntimeCallable,
    JavaRuntimeHelper,
};

pub(super) fn bytes_members() -> Vec<JavaMember> {
    let integer = JavaType::Boxed(JavaPrimitive::Int);
    let list = generic(JavaKnownType::List, vec![integer.clone()]);
    let byte = JavaType::primitive(JavaPrimitive::Byte);
    let byte_array = JavaType::Array {
        component: Box::new(byte.clone()),
        ownership: JavaArrayOwnership::DefensiveCopyBoundary,
    };
    let mutable_byte_array = JavaType::Array {
        component: Box::new(byte.clone()),
        ownership: JavaArrayOwnership::InternalMutable,
    };
    let bytes = JavaType::known(JavaKnownType::RuntimeBytes);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let components = vec![component(
        byte_array.clone(),
        "values",
        JavaRuntimeMember::BytesValues,
    )];
    let source = local(byte_array.clone(), "source");
    let copied = local(mutable_byte_array.clone(), "copy");
    let index = local(int.clone(), "index");
    let field = structural_field(this_value(bytes.clone()), "values", byte_array.clone());
    vec![
        JavaMember::NestedType(JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Public,
            modifiers: vec![JavaModifier::Static],
            name: identifier("Bytes"),
            type_parameters: vec![],
            record_components: components.clone(),
            heritage: JavaHeritage::Interfaces(vec![JavaType::known(
                JavaKnownType::RuntimeSemanticValue,
            )]),
            permits: vec![],
            members: vec![
                JavaMember::Constructor(JavaConstructor {
                    modifiers: vec![JavaModifier::Public],
                    name: identifier("Bytes"),
                    parameters: vec![parameter(byte_array.clone(), "values")],
                    body: JavaBlock::new(vec![
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Final,
                            ty: byte_array.clone(),
                            name: identifier("source"),
                            value: Some(known_generic_call(
                                JavaKnownCallable::ObjectsRequireNonNull,
                                vec![local(byte_array.clone(), "values")],
                                byte_array.clone(),
                            )),
                        },
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Final,
                            ty: mutable_byte_array.clone(),
                            name: identifier("copy"),
                            value: Some(new_array(byte.clone(), array_length(source.clone()))),
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
                                array_length(source.clone()),
                                boolean.clone(),
                            ),
                            body: JavaBlock::new(vec![
                                JavaStmt::Assign {
                                    target: array_index(
                                        copied.clone(),
                                        index.clone(),
                                        byte.clone(),
                                    ),
                                    value: array_index(source.clone(), index.clone(), byte.clone()),
                                },
                                JavaStmt::Assign {
                                    target: index.clone(),
                                    value: binary(
                                        JavaBinaryOperator::Add,
                                        index.clone(),
                                        int_literal(1),
                                        int.clone(),
                                    ),
                                },
                            ]),
                        },
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Final,
                            ty: byte_array.clone(),
                            name: identifier("frozen"),
                            value: Some(fresh_copy_to_boundary(copied.clone(), byte_array.clone())),
                        },
                        assign_component(
                            bytes.clone(),
                            "values",
                            byte_array.clone(),
                            local(byte_array.clone(), "frozen"),
                        ),
                    ]),
                }),
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Structural,
                    annotations: vec![],
                    modifiers: vec![JavaModifier::Public],
                    type_parameters: vec![],
                    return_type: byte_array.clone(),
                    name: identifier("values"),
                    parameters: vec![],
                    body: Some(JavaBlock::new(vec![
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Final,
                            ty: mutable_byte_array.clone(),
                            name: identifier("copy"),
                            value: Some(new_array(byte.clone(), array_length(field.clone()))),
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
                                array_length(field.clone()),
                                boolean.clone(),
                            ),
                            body: JavaBlock::new(vec![
                                JavaStmt::Assign {
                                    target: array_index(
                                        copied.clone(),
                                        index.clone(),
                                        byte.clone(),
                                    ),
                                    value: array_index(field.clone(), index.clone(), byte.clone()),
                                },
                                JavaStmt::Assign {
                                    target: index.clone(),
                                    value: binary(
                                        JavaBinaryOperator::Add,
                                        index.clone(),
                                        int_literal(1),
                                        int.clone(),
                                    ),
                                },
                            ]),
                        },
                        JavaStmt::Local {
                            finality: JavaLocalFinality::Final,
                            ty: byte_array.clone(),
                            name: identifier("frozen"),
                            value: Some(fresh_copy_to_boundary(copied.clone(), byte_array.clone())),
                        },
                        JavaStmt::Return(Some(local(byte_array.clone(), "frozen"))),
                    ])),
                }),
                runtime_record_equality_method(
                    bytes.clone(),
                    bytes.clone(),
                    &components,
                    JavaRuntimeCallable::SemanticEqual,
                    JavaRuntimeMember::SemanticEquals,
                ),
                runtime_record_equality_method(
                    bytes.clone(),
                    bytes.clone(),
                    &components,
                    JavaRuntimeCallable::DeepEqual,
                    JavaRuntimeMember::DeepEquals,
                ),
            ],
        }),
        static_method(
            vec![],
            bytes.clone(),
            "bytesOf",
            vec![parameter(list.clone(), "values")],
            {
                let immutable = local(list.clone(), "immutable");
                let item = local(integer.clone(), "item");
                vec![
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: list.clone(),
                        name: identifier("immutable"),
                        value: Some(known_generic_call(
                            JavaKnownCallable::ListCopyOf,
                            vec![local(list.clone(), "values")],
                            list.clone(),
                        )),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: mutable_byte_array.clone(),
                        name: identifier("copy"),
                        value: Some(new_array(
                            byte.clone(),
                            known_method_call(
                                JavaKnownMethod::ListSize,
                                immutable.clone(),
                                vec![],
                                int.clone(),
                            ),
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
                            known_method_call(
                                JavaKnownMethod::ListSize,
                                immutable.clone(),
                                vec![],
                                int.clone(),
                            ),
                            boolean.clone(),
                        ),
                        body: JavaBlock::new(vec![
                            JavaStmt::Local {
                                finality: JavaLocalFinality::Final,
                                ty: integer.clone(),
                                name: identifier("item"),
                                value: Some(known_method_call(
                                    JavaKnownMethod::ListGet,
                                    immutable.clone(),
                                    vec![index.clone()],
                                    integer.clone(),
                                )),
                            },
                            JavaStmt::If {
                                condition: binary(
                                    JavaBinaryOperator::LogicalOr,
                                    binary(
                                        JavaBinaryOperator::Less,
                                        cast(int.clone(), item.clone()),
                                        int_literal(0),
                                        boolean.clone(),
                                    ),
                                    binary(
                                        JavaBinaryOperator::Greater,
                                        cast(int.clone(), item.clone()),
                                        int_literal(255),
                                        boolean.clone(),
                                    ),
                                    boolean.clone(),
                                ),
                                then_block: JavaBlock::new(vec![illegal_argument(
                                    "byte value is outside 0..255",
                                )]),
                                else_block: None,
                            },
                            JavaStmt::Assign {
                                target: array_index(copied.clone(), index.clone(), byte.clone()),
                                value: cast(byte.clone(), cast(int.clone(), item)),
                            },
                            JavaStmt::Assign {
                                target: index.clone(),
                                value: binary(
                                    JavaBinaryOperator::Add,
                                    index.clone(),
                                    int_literal(1),
                                    int.clone(),
                                ),
                            },
                        ]),
                    },
                    JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: byte_array.clone(),
                        name: identifier("frozen"),
                        value: Some(fresh_copy_to_boundary(copied.clone(), byte_array.clone())),
                    },
                    JavaStmt::Return(Some(new_known(
                        JavaKnownConstructor::RuntimeBytes,
                        bytes.clone(),
                        vec![local(byte_array.clone(), "frozen")],
                    ))),
                ]
            },
        ),
    ]
    .into_iter()
    .chain(
        JavaRuntimeCallable::ALL
            .into_iter()
            .filter(|value| {
                value.helper() == JavaRuntimeHelper::Bytes && *value != JavaRuntimeCallable::BytesOf
            })
            .map(runtime_method),
    )
    .collect()
}
