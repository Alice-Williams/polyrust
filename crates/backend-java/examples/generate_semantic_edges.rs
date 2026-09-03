use std::path::PathBuf;

use portable_backend_java::JavaBackend;
use portable_build::{
    Expected, Invocation, ModuleBuilder, Operation, Parameter, Type, TypedValue, Value, Visibility,
};
use portable_codegen::{Backend, BackendOptions, OutputContents};

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let output = PathBuf::from(arguments.next().expect("output path"));
    assert!(arguments.next().is_none(), "unexpected argument");

    let mut module = ModuleBuilder::new("java_semantic_edges");
    let nested_string_constant = module.constant(
        "NESTED_STRING_CONSTANT",
        Visibility::Public,
        vec![],
        Type::string(),
        |body| {
            let first = body.constant_literal(Value::string("a"));
            let second = body.constant_literal(Value::string("b"));
            let nested = body.constant_intrinsic(Operation::StringConcat, [first, second]);
            let third = body.constant_literal(Value::string("c"));
            body.constant_intrinsic(Operation::StringConcat, [nested, third])
        },
    );
    let compound_bytes_constant = module.constant(
        "COMPOUND_BYTES_CONSTANT",
        Visibility::Public,
        vec![],
        Type::bytes(),
        |body| {
            let first = body.constant_literal(Value::bytes([1, 2]));
            let second = body.constant_literal(Value::bytes([3, 4]));
            body.constant_intrinsic(Operation::BytesConcat, [first, second])
        },
    );
    let z_dependency = module.constant(
        "Z_DEPENDENCY",
        Visibility::Public,
        vec![],
        Type::i64(),
        |body| body.constant_literal(Value::i64(7)),
    );
    let a_dependent = module.constant(
        "A_DEPENDENT",
        Visibility::Public,
        vec![],
        Type::i64(),
        |body| body.constant_reference(z_dependency),
    );
    let read_nested_string = module.function(
        "read_nested_string",
        Visibility::Public,
        vec![],
        |function| {
            function.returns(Type::string());
            function.body(|body| {
                let value = body.constant(nested_string_constant);
                body.block([], Some(value))
            });
        },
    );
    let read_compound_bytes = module.function(
        "read_compound_bytes",
        Visibility::Public,
        vec![],
        |function| {
            function.returns(Type::bytes());
            function.body(|body| {
                let value = body.constant(compound_bytes_constant);
                body.block([], Some(value))
            });
        },
    );
    let read_dependent_constant = module.function(
        "read_dependent_constant",
        Visibility::Public,
        vec![],
        |function| {
            function.returns(Type::i64());
            function.body(|body| {
                let value = body.constant(a_dependent);
                body.block([], Some(value))
            });
        },
    );
    let string_less = module.function("string_less", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("left", Type::string()));
        function.parameter(Parameter::new("right", Type::string()));
        function.returns(Type::bool());
        function.body(|body| {
            let left = body.local("left");
            let right = body.local("right");
            let value = body.intrinsic(Operation::Less, [left, right]);
            body.block([], Some(value))
        });
    });
    let char_less = module.function("char_less", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("left", Type::char()));
        function.parameter(Parameter::new("right", Type::char()));
        function.returns(Type::bool());
        function.body(|body| {
            let left = body.local("left");
            let right = body.local("right");
            let value = body.intrinsic(Operation::Less, [left, right]);
            body.block([], Some(value))
        });
    });
    let overflow_inner =
        module.function("overflow_inner", Visibility::Package, vec![], |function| {
            function.returns(Type::i32());
            function.body(|body| {
                let maximum = body.literal(Value::i32(i32::MAX));
                let one = body.literal(Value::i32(1));
                let value = body.intrinsic(Operation::IntAddChecked, [maximum, one]);
                body.block([], Some(value))
            });
        });
    let overflow_outer =
        module.function("overflow_outer", Visibility::Public, vec![], |function| {
            function.returns(Type::i32());
            function.body(|body| {
                let value = body.call(overflow_inner, []);
                body.block([], Some(value))
            });
        });
    let short_circuit = module.function("short_circuit", Visibility::Public, vec![], |function| {
        function.returns(Type::bool());
        function.body(|body| {
            let left = body.literal(Value::bool(false));
            let maximum = body.literal(Value::i32(i32::MAX));
            let one = body.literal(Value::i32(1));
            let overflow = body.intrinsic(Operation::IntAddChecked, [maximum, one]);
            let zero = body.literal(Value::i32(0));
            let right = body.intrinsic(Operation::Equal, [overflow, zero]);
            let value = body.intrinsic(Operation::BoolAnd, [left, right]);
            body.block([], Some(value))
        });
    });
    module.function("echo_nested_list", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new(
            "values",
            Type::list(Type::list(Type::string())),
        ));
        function.returns(Type::list(Type::list(Type::string())));
        function.body(|body| {
            let value = body.local("values");
            body.block([], Some(value))
        });
    });
    module.function(
        "echo_nested_option",
        Visibility::Public,
        vec![],
        |function| {
            function.parameter(Parameter::new(
                "value",
                Type::option(Type::list(Type::list(Type::string()))),
            ));
            function.returns(Type::option(Type::list(Type::list(Type::string()))));
            function.body(|body| {
                let value = body.local("value");
                body.block([], Some(value))
            });
        },
    );
    module.function(
        "echo_nested_result",
        Visibility::Public,
        vec![],
        |function| {
            function.parameter(Parameter::new(
                "value",
                Type::result(
                    Type::list(Type::list(Type::string())),
                    Type::list(Type::list(Type::string())),
                ),
            ));
            function.returns(Type::result(
                Type::list(Type::list(Type::string())),
                Type::list(Type::list(Type::string())),
            ));
            function.body(|body| {
                let value = body.local("value");
                body.block([], Some(value))
            });
        },
    );
    module.function("echo_f64_option", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::option(Type::f64())));
        function.returns(Type::option(Type::f64()));
        function.body(|body| {
            let value = body.local("value");
            body.block([], Some(value))
        });
    });
    module.function(
        "echo_nested_f64_result",
        Visibility::Public,
        vec![],
        |function| {
            function.parameter(Parameter::new(
                "value",
                Type::result(Type::list(Type::option(Type::f64())), Type::string()),
            ));
            function.returns(Type::result(
                Type::list(Type::option(Type::f64())),
                Type::string(),
            ));
            function.body(|body| {
                let value = body.local("value");
                body.block([], Some(value))
            });
        },
    );
    let checked_rem_i32 =
        module.function("checked_rem_i32", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("left", Type::i32()));
            function.parameter(Parameter::new("right", Type::i32()));
            function.returns(Type::i32());
            function.body(|body| {
                let left = body.local("left");
                let right = body.local("right");
                let value = body.intrinsic(Operation::IntRemChecked, [left, right]);
                body.block([], Some(value))
            });
        });
    let checked_rem_i64 =
        module.function("checked_rem_i64", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("left", Type::i64()));
            function.parameter(Parameter::new("right", Type::i64()));
            function.returns(Type::i64());
            function.body(|body| {
                let left = body.local("left");
                let right = body.local("right");
                let value = body.intrinsic(Operation::IntRemChecked, [left, right]);
                body.block([], Some(value))
            });
        });
    let replace_all = module.function("replace_all", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("source", Type::string()));
        function.parameter(Parameter::new("needle", Type::string()));
        function.parameter(Parameter::new("replacement", Type::string()));
        function.returns(Type::string());
        function.body(|body| {
            let source = body.local("source");
            let needle = body.local("needle");
            let replacement = body.local("replacement");
            let value = body.intrinsic(Operation::StringReplaceAll, [source, needle, replacement]);
            body.block([], Some(value))
        });
    });

    let astral = "\u{10000}";
    let bmp = "\u{e000}";
    module.portable_test(
        "string_order_uses_scalars",
        Visibility::Package,
        vec![],
        Invocation::function(
            string_less,
            [
                TypedValue::new(Type::string(), Value::string(astral)),
                TypedValue::new(Type::string(), Value::string(bmp)),
            ],
        ),
        Expected::value(TypedValue::new(Type::bool(), Value::bool(false))),
    );
    module.portable_test(
        "char_order_uses_scalars",
        Visibility::Package,
        vec![],
        Invocation::function(
            char_less,
            [
                TypedValue::new(Type::char(), Value::char('\u{10000}')),
                TypedValue::new(Type::char(), Value::char('\u{e000}')),
            ],
        ),
        Expected::value(TypedValue::new(Type::bool(), Value::bool(false))),
    );
    module.portable_test(
        "nested_call_preserves_error",
        Visibility::Package,
        vec![],
        Invocation::function(overflow_outer, []),
        Expected::error(TypedValue::new(
            Type::string(),
            Value::string("checked_overflow"),
        )),
    );
    module.portable_test(
        "boolean_and_short_circuits",
        Visibility::Package,
        vec![],
        Invocation::function(short_circuit, []),
        Expected::value(TypedValue::new(Type::bool(), Value::bool(false))),
    );
    module.portable_test(
        "nested_infallible_constant_intrinsics",
        Visibility::Package,
        vec![],
        Invocation::function(read_nested_string, []),
        Expected::value(TypedValue::new(Type::string(), Value::string("abc"))),
    );
    module.portable_test(
        "compound_constant_intrinsic_operands",
        Visibility::Package,
        vec![],
        Invocation::function(read_compound_bytes, []),
        Expected::value(TypedValue::new(Type::bytes(), Value::bytes([1, 2, 3, 4]))),
    );
    module.portable_test(
        "constant_dependencies_are_emitted_before_dependents",
        Visibility::Package,
        vec![],
        Invocation::function(read_dependent_constant, []),
        Expected::value(TypedValue::new(Type::i64(), Value::i64(7))),
    );
    module.portable_test(
        "i32_min_remainder_negative_one_overflows",
        Visibility::Package,
        vec![],
        Invocation::function(
            checked_rem_i32,
            [
                TypedValue::new(Type::i32(), Value::i32(i32::MIN)),
                TypedValue::new(Type::i32(), Value::i32(-1)),
            ],
        ),
        Expected::error(TypedValue::new(
            Type::string(),
            Value::string("checked_overflow"),
        )),
    );
    module.portable_test(
        "i64_min_remainder_negative_one_overflows",
        Visibility::Package,
        vec![],
        Invocation::function(
            checked_rem_i64,
            [
                TypedValue::new(Type::i64(), Value::i64(i64::MIN)),
                TypedValue::new(Type::i64(), Value::i64(-1)),
            ],
        ),
        Expected::error(TypedValue::new(
            Type::string(),
            Value::string("checked_overflow"),
        )),
    );
    module.portable_test(
        "empty_needle_replacement_uses_unicode_scalar_boundaries",
        Visibility::Package,
        vec![],
        Invocation::function(
            replace_all,
            [
                TypedValue::new(Type::string(), Value::string("a🦀")),
                TypedValue::new(Type::string(), Value::string("")),
                TypedValue::new(Type::string(), Value::string("-")),
            ],
        ),
        Expected::value(TypedValue::new(Type::string(), Value::string("-a-🦀-"))),
    );
    module.portable_test(
        "empty_source_has_one_empty_needle_boundary",
        Visibility::Package,
        vec![],
        Invocation::function(
            replace_all,
            [
                TypedValue::new(Type::string(), Value::string("")),
                TypedValue::new(Type::string(), Value::string("")),
                TypedValue::new(Type::string(), Value::string("-")),
            ],
        ),
        Expected::value(TypedValue::new(Type::string(), Value::string("-"))),
    );
    module.portable_test(
        "empty_replacement_preserves_source",
        Visibility::Package,
        vec![],
        Invocation::function(
            replace_all,
            [
                TypedValue::new(Type::string(), Value::string("a🦀")),
                TypedValue::new(Type::string(), Value::string("")),
                TypedValue::new(Type::string(), Value::string("")),
            ],
        ),
        Expected::value(TypedValue::new(Type::string(), Value::string("a🦀"))),
    );

    let checked = module.finish().expect("semantic edge fixture checks");
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("generate semantic edge fixture");
    for file in manifest.files() {
        let path = output.join(file.path());
        std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        match file.contents() {
            OutputContents::Text(text) => std::fs::write(path, text),
            OutputContents::Bytes(bytes) => std::fs::write(path, bytes),
        }
        .expect("write output");
    }
}
