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
