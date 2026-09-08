use crate::{
    Bool, I32, ModuleBuilder, Operation, Parameter, Requirements, Type, TypedEquatable, TypedType,
    Unit, Visibility, field, interface_method, list_type, method_binding, option_type, parameter,
    portable_name, typed_list, typed_program,
};
use portable_diagnostics::DiagnosticCode;

fn assert_equatable<T: TypedEquatable, R: Requirements>(_: TypedType<T, R>) {}

#[test]
fn unit_and_transparent_alias_fields_keep_record_equality() {
    let program = typed_program(portable_name!("alias_record_equality"), |builder| {
        builder.alias(portable_name!("Count"), I32::TYPE, |builder, count| {
            builder.record(
                portable_name!("Holder"),
                typed_list![
                    field(portable_name!("unit"), Unit::TYPE),
                    field(portable_name!("count"), count.ty())
                ],
                |builder, holder| {
                    assert_equatable(holder.ty());
                    builder
                        .function(
                            portable_name!("compare"),
                            typed_list![
                                parameter(portable_name!("left"), holder.ty()),
                                parameter(portable_name!("right"), holder.ty())
                            ],
                            Bool::TYPE,
                            |body, args| {
                                let left = body.read(args.head);
                                let right = body.read(args.tail.head);
                                body.equal(left, right)
                            },
                        )
                        .builder
                },
            )
        })
    });
    let core = portable_core_ir::lower_checked(program.checked_program()).unwrap();
    portable_core_ir::verify_core(&core).unwrap();
}

#[test]
fn concrete_method_receiver_retains_positive_field_equality() {
    let program = typed_program(portable_name!("receiver_equality"), |builder| {
        builder.record(
            portable_name!("Holder"),
            typed_list![field(portable_name!("value"), I32::TYPE)],
            |builder, holder| {
                builder.interface(
                    portable_name!("Compare"),
                    typed_list![interface_method(
                        portable_name!("compare"),
                        typed_list![parameter(portable_name!("other"), holder.ty())],
                        Bool::TYPE
                    )],
                    |builder, compare| {
                        let method = method_binding(
                            &holder,
                            &compare.methods().head,
                            portable_name!("compare"),
                            |body, receiver, args| {
                                let other = body.read(args.head);
                                body.equal(receiver, other)
                            },
                        );
                        builder.implementation(
                            portable_name!("CompareHolder"),
                            &compare,
                            &holder,
                            typed_list![method],
                            |builder, _| builder,
                        )
                    },
                )
            },
        )
    });
    let core = portable_core_ir::lower_checked(program.checked_program()).unwrap();
    portable_core_ir::verify_core(&core).unwrap();
}

#[test]
fn record_equality_is_derived_recursively_including_empty_records() {
    let program = typed_program(portable_name!("nested_equatable_records"), |builder| {
        builder.record(portable_name!("Empty"), typed_list![], |builder, empty| {
            assert_equatable(empty.ty());
            builder.record(
                portable_name!("Inner"),
                typed_list![field(
                    portable_name!("values"),
                    option_type(list_type(I32::TYPE))
                )],
                |builder, inner| {
                    builder.record(
                        portable_name!("Outer"),
                        typed_list![
                            field(portable_name!("inner"), inner.ty()),
                            field(portable_name!("empty"), empty.ty())
                        ],
                        |builder, outer| {
                            assert_equatable(outer.ty());
                            builder
                                .function(
                                    portable_name!("compare"),
                                    typed_list![
                                        parameter(portable_name!("left"), outer.ty()),
                                        parameter(portable_name!("right"), outer.ty())
                                    ],
                                    Bool::TYPE,
                                    |body, args| {
                                        let left = body.read(args.head);
                                        let right = body.read(args.tail.head);
                                        body.equal(left, right)
                                    },
                                )
                                .builder
                        },
                    )
                },
            )
        })
    });
    let core = portable_core_ir::lower_checked(program.checked_program()).unwrap();
    portable_core_ir::verify_core(&core).unwrap();
}

#[test]
fn nested_interface_storage_and_projection_stay_admitted() {
    let program = typed_program(portable_name!("interface_storage"), |builder| {
        builder.interface(portable_name!("View"), typed_list![], |builder, view| {
            builder.record(
                portable_name!("Holder"),
                typed_list![field(
                    portable_name!("views"),
                    option_type(list_type(view.ty()))
                )],
                |builder, holder| {
                    builder
                        .function(
                            portable_name!("project"),
                            typed_list![parameter(portable_name!("holder"), holder.ty())],
                            option_type(list_type(view.ty())),
                            |body, args| {
                                let value = body.read(args.head);
                                body.field(value, holder.fields().head)
                            },
                        )
                        .builder
                },
            )
        })
    });
    let core = portable_core_ir::lower_checked(program.checked_program()).unwrap();
    portable_core_ir::verify_core(&core).unwrap();
}

#[test]
fn dynamic_checker_rejects_all_recursive_interface_equality_operations() {
    for operation in [
        Operation::Equal,
        Operation::NotEqual,
        Operation::ListContains,
        Operation::ListIndexOf,
    ] {
        let mut module = ModuleBuilder::new("dynamic_interface_equality");
        let (view, ()) = module.interface("View", Visibility::Public, vec![], |_| {});
        let (inner, _) = module.record("Inner", Visibility::Public, vec![], |record| {
            record.field(
                "views",
                Type::option(Type::list(Type::interface(view))),
                vec![],
            )
        });
        let (outer, _) = module.record("Outer", Visibility::Public, vec![], |record| {
            record.field("inner", Type::named(inner), vec![])
        });
        module.function("compare", Visibility::Public, vec![], |function| {
            let (left, result) = match operation {
                Operation::Equal | Operation::NotEqual => (Type::named(outer), Type::bool()),
                Operation::ListContains => (Type::list(Type::named(outer)), Type::bool()),
                Operation::ListIndexOf => {
                    (Type::list(Type::named(outer)), Type::option(Type::i64()))
                }
                _ => unreachable!("closed test operation matrix"),
            };
            function.parameter(Parameter::new("left", left));
            function.parameter(Parameter::new("right", Type::named(outer)));
            function.returns(result);
            function.body(|body| {
                let left = body.local("left");
                let right = body.local("right");
                let result = body.intrinsic(operation, [left, right]);
                body.block([], Some(result))
            });
        });
        let errors = module
            .finish()
            .expect_err("interface equality must be rejected");
        assert!(
            errors
                .iter()
                .any(|error| error.code == DiagnosticCode::InvalidInterfacePosition),
            "{operation:?}: {errors:?}"
        );
    }
}
