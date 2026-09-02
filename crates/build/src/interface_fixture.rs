use portable_ir::v0::Document;

use crate::{
    EnumFieldId, EnumId, EnumVariantId, Expected, FunctionId, ImplementationId,
    ImplementationMethodId, InterfaceId, InterfaceMethodId, Invocation, ModuleBuilder, Operation,
    Parameter, RecordFieldId, RecordId, Type, TypedValue, Value, Visibility,
};

/// Stable handles into the canonical M34A interface/composition corpus.
#[derive(Clone, Debug)]
pub struct InterfaceFixture {
    pub document: Document,
    pub label: RecordId,
    pub label_text: RecordFieldId,
    pub labelled: InterfaceId,
    pub label_method: InterfaceMethodId,
    pub label_implementation: ImplementationId,
    pub label_implementation_method: ImplementationMethodId,
    pub measured: InterfaceId,
    pub measured_implementation: ImplementationId,
    pub static_dispatch: FunctionId,
    pub dynamic_dispatch: FunctionId,
    pub return_interface: FunctionId,
    pub local_dispatch: FunctionId,
    pub list_dispatch: FunctionId,
    pub option_dispatch: FunctionId,
    pub result_dispatch: FunctionId,
    pub composition_dispatch: FunctionId,
    pub enum_dispatch: FunctionId,
    pub measured_dispatch: FunctionId,
    pub service: RecordId,
    pub service_renderer: RecordFieldId,
    pub envelope: EnumId,
    pub wrapped: EnumVariantId,
    pub wrapped_renderer: EnumFieldId,
}

/// Builds the single source of truth used to prove portable interface and
/// composition semantics in the checker, CoreIR, evaluator, and every target.
pub fn interface_composition_fixture() -> InterfaceFixture {
    let mut module = ModuleBuilder::new("interface_composition");

    let (labelled, label_method) = module.interface(
        "Labelled",
        Visibility::Public,
        vec!["A flat immutable label interface.".into()],
        |interface| {
            interface.method(
                "label",
                vec![],
                vec![Parameter::new("prefix", Type::string())],
                Some(Type::string()),
            )
        },
    );
    let (measured, measure_method) = module.interface(
        "Measured",
        Visibility::Public,
        vec!["An independent flat interface implemented by Label.".into()],
        |interface| interface.method("measure", vec![], vec![], Some(Type::i64())),
    );
    let (label, label_text) = module.record(
        "Label",
        Visibility::Public,
        vec!["An immutable label value.".into()],
        |record| record.field("text", Type::string(), vec![]),
    );
    let (service, service_renderer) = module.record(
        "Service",
        Visibility::Public,
        vec!["Composition through an explicitly named interface field.".into()],
        |record| record.field("renderer", Type::interface(labelled), vec![]),
    );
    let (envelope, (wrapped, wrapped_renderer)) = module.enumeration(
        "Envelope",
        Visibility::Public,
        vec!["Tagged interface payload coverage.".into()],
        |enumeration| {
            enumeration.variant("Wrapped", vec![], |variant| {
                variant.field("renderer", Type::interface(labelled), vec![])
            })
        },
    );

    let (label_implementation, (label_implementation_method, ())) = module.implementation(
        "LabelLabelled",
        Visibility::Package,
        vec![],
        labelled,
        label,
        |implementation| {
            implementation.method("label", label_method, vec![], |method| {
                method.parameter(Parameter::new("prefix", Type::string()));
                method.returns(Type::string());
                method.body(|body| {
                    let receiver = body.self_value();
                    let text = body.field(receiver, label_text);
                    let prefix = body.local("prefix");
                    let value = body.intrinsic(Operation::StringConcat, [prefix, text]);
                    body.block([], Some(value))
                });
            })
        },
    );
    let (measured_implementation, (_measured_implementation_method, ())) = module.implementation(
        "LabelMeasured",
        Visibility::Package,
        vec![],
        measured,
        label,
        |implementation| {
            implementation.method("measure", measure_method, vec![], |method| {
                method.returns(Type::i64());
                method.body(|body| {
                    let receiver = body.self_value();
                    let text = body.field(receiver, label_text);
                    let value = body.intrinsic(Operation::StringScalarLength, [text]);
                    body.block([], Some(value))
                });
            })
        },
    );

    let static_dispatch =
        module.function("static_dispatch", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::named(label)));
            function.parameter(Parameter::new("prefix", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let receiver = body.local("value");
                let prefix = body.local("prefix");
                let value = body.concrete_method(
                    receiver,
                    label_implementation,
                    label_implementation_method,
                    [prefix],
                );
                body.block([], Some(value))
            });
        });
    let dynamic_dispatch =
        module.function("dynamic_dispatch", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::interface(labelled)));
            function.parameter(Parameter::new("prefix", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let receiver = body.local("value");
                let prefix = body.local("prefix");
                let value = body.interface_method(receiver, labelled, label_method, [prefix]);
                body.block([], Some(value))
            });
        });
    let return_interface =
        module.function("return_interface", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::named(label)));
            function.returns(Type::interface(labelled));
            function.body(|body| {
                let value = body.local("value");
                let value = body.interface_value(label_implementation, value);
                body.block([], Some(value))
            });
        });
    let local_dispatch =
        module.function("local_dispatch", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::named(label)));
            function.returns(Type::string());
            function.body(|body| {
                let value = body.local("value");
                let value = body.interface_value(label_implementation, value);
                let bind = body.let_statement("renderer", Some(Type::interface(labelled)), value);
                let renderer = body.local("renderer");
                let prefix = body.literal(Value::string("local:"));
                let result = body.interface_method(renderer, labelled, label_method, [prefix]);
                body.block([bind], Some(result))
            });
        });
    let list_dispatch = module.function("list_dispatch", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::named(label)));
        function.returns(Type::string());
        function.body(|body| {
            let value = body.local("value");
            let value = body.interface_value(label_implementation, value);
            let values = body.list(Type::interface(labelled), [value]);
            let zero = body.literal(Value::i64(0));
            let renderer = body.intrinsic(Operation::ListGetChecked, [values, zero]);
            let prefix = body.literal(Value::string("list:"));
            let result = body.interface_method(renderer, labelled, label_method, [prefix]);
            body.block([], Some(result))
        });
    });
    let option_dispatch =
        module.function("option_dispatch", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::named(label)));
            function.returns(Type::string());
            function.body(|body| {
                let value = body.local("value");
                let value = body.interface_value(label_implementation, value);
                let option = body.some(value);
                let some_pattern = body.some_pattern("renderer");
                let renderer = body.local("renderer");
                let prefix = body.literal(Value::string("option:"));
                let some_result = body.interface_method(renderer, labelled, label_method, [prefix]);
                let some_block = body.block([], Some(some_result));
                let some_arm = body.match_arm(some_pattern, some_block);
                let none_pattern = body.none_pattern();
                let none_result = body.literal(Value::string("none"));
                let none_block = body.block([], Some(none_result));
                let none_arm = body.match_arm(none_pattern, none_block);
                let result = body.match_value(option, [some_arm, none_arm]);
                body.block([], Some(result))
            });
        });
    let result_dispatch =
        module.function("result_dispatch", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::named(label)));
            function.returns(Type::string());
            function.body(|body| {
                let value = body.local("value");
                let value = body.interface_value(label_implementation, value);
                let result_value = body.ok(value, Type::string());
                let ok_pattern = body.ok_pattern("renderer");
                let renderer = body.local("renderer");
                let prefix = body.literal(Value::string("result:"));
                let ok_result = body.interface_method(renderer, labelled, label_method, [prefix]);
                let ok_block = body.block([], Some(ok_result));
                let ok_arm = body.match_arm(ok_pattern, ok_block);
                let err_pattern = body.err_pattern("message");
                let message = body.local("message");
                let err_block = body.block([], Some(message));
                let err_arm = body.match_arm(err_pattern, err_block);
                let result = body.match_value(result_value, [ok_arm, err_arm]);
                body.block([], Some(result))
            });
        });
    let composition_dispatch = module.function(
        "composition_dispatch",
        Visibility::Public,
        vec![],
        |function| {
            function.parameter(Parameter::new("value", Type::named(label)));
            function.returns(Type::string());
            function.body(|body| {
                let value = body.local("value");
                let renderer = body.interface_value(label_implementation, value);
                let service_value = body.record(service, [(service_renderer, renderer)]);
                let renderer = body.field(service_value, service_renderer);
                let prefix = body.literal(Value::string("composition:"));
                let result = body.interface_method(renderer, labelled, label_method, [prefix]);
                body.block([], Some(result))
            });
        },
    );
    let enum_dispatch = module.function("enum_dispatch", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::named(label)));
        function.returns(Type::string());
        function.body(|body| {
            let value = body.local("value");
            let renderer = body.interface_value(label_implementation, value);
            let envelope_value =
                body.enumeration(envelope, wrapped, [(wrapped_renderer, renderer)]);
            let pattern = body.enum_pattern(
                envelope,
                wrapped,
                [(wrapped_renderer, "renderer".to_owned())],
            );
            let renderer = body.local("renderer");
            let prefix = body.literal(Value::string("enum:"));
            let result = body.interface_method(renderer, labelled, label_method, [prefix]);
            let arm_block = body.block([], Some(result));
            let arm = body.match_arm(pattern, arm_block);
            let result = body.match_value(envelope_value, [arm]);
            body.block([], Some(result))
        });
    });
    let measured_dispatch = module.function(
        "measured_dispatch",
        Visibility::Public,
        vec![],
        |function| {
            function.parameter(Parameter::new("value", Type::named(label)));
            function.returns(Type::i64());
            function.body(|body| {
                let value = body.local("value");
                let value = body.interface_value(measured_implementation, value);
                let result = body.interface_method(value, measured, measure_method, []);
                body.block([], Some(result))
            });
        },
    );

    let label_value = || {
        TypedValue::new(
            Type::named(label),
            Value::record(label, [(label_text, Value::string("value"))]),
        )
    };
    for (name, function, expected) in [
        ("static", static_dispatch, "static:value"),
        ("dynamic", dynamic_dispatch, "dynamic:value"),
    ] {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                function,
                [
                    label_value(),
                    TypedValue::new(Type::string(), Value::string(format!("{name}:"))),
                ],
            ),
            Expected::value(TypedValue::new(Type::string(), Value::string(expected))),
        );
    }
    for (name, function, expected) in [
        ("local", local_dispatch, "local:value"),
        ("list", list_dispatch, "list:value"),
        ("option", option_dispatch, "option:value"),
        ("result", result_dispatch, "result:value"),
        ("composition", composition_dispatch, "composition:value"),
        ("enum", enum_dispatch, "enum:value"),
    ] {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(function, [label_value()]),
            Expected::value(TypedValue::new(Type::string(), Value::string(expected))),
        );
    }
    module.portable_test(
        "multiple_conformance",
        Visibility::Package,
        vec![],
        Invocation::function(measured_dispatch, [label_value()]),
        Expected::value(TypedValue::new(Type::i64(), Value::i64(5))),
    );

    InterfaceFixture {
        document: module
            .finish_unchecked()
            .expect("canonical interface fixture is structurally complete"),
        label,
        label_text,
        labelled,
        label_method,
        label_implementation,
        label_implementation_method,
        measured,
        measured_implementation,
        static_dispatch,
        dynamic_dispatch,
        return_interface,
        local_dispatch,
        list_dispatch,
        option_dispatch,
        result_dispatch,
        composition_dispatch,
        enum_dispatch,
        measured_dispatch,
        service,
        service_renderer,
        envelope,
        wrapped,
        wrapped_renderer,
    }
}
