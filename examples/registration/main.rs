#![forbid(unsafe_code)]

use portable_backend_go::GoBackend;
use portable_backend_rust::RustBackend;
use portable_build::{
    Contract, Expression, Field, Function, Implementation, MethodSignature, ModuleBuilder,
    Parameter, PortableTest, Record, Type, Value,
};
use portable_codegen::{Backend, GeneratedPackage};

fn main() {
    let artifact = std::env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("expected one of: rust-source, go-source, go-test"));
    let module = registration_module();
    let package = match artifact.as_str() {
        "rust-source" => RustBackend.emit(&module),
        "go-source" | "go-test" => GoBackend.emit(&module),
        other => panic!("unknown artifact `{other}`"),
    };
    let path = match artifact.as_str() {
        "rust-source" => "src/lib.rs",
        "go-source" => "generated.go",
        "go-test" => "generated_test.go",
        _ => unreachable!(),
    };
    print_artifact(&package, path);
}

fn print_artifact(package: &GeneratedPackage, path: &str) {
    print!(
        "{}",
        package
            .file(path)
            .unwrap_or_else(|| panic!("backend did not produce `{path}`"))
    );
}

fn registration_module() -> portable_check::CheckedModule {
    let mut builder = ModuleBuilder::new("registration");
    builder.constant("adult_age", Type::I64, Value::I64(18));
    builder.record(Record {
        name: "User".into(),
        fields: vec![
            Field::new("name", Type::String),
            Field::new("age", Type::I64),
        ],
    });
    builder.record(Record {
        name: "AgeValidator".into(),
        fields: vec![Field::new("minimum", Type::I64)],
    });
    builder.contract(Contract {
        name: "Validator".into(),
        methods: vec![MethodSignature {
            name: "accepts".into(),
            parameters: vec![Parameter::new("user", Type::named("User"))],
            return_type: Type::Bool,
        }],
    });
    builder.implementation(Implementation {
        contract: "Validator".into(),
        record: "AgeValidator".into(),
        methods: vec![Function {
            name: "accepts".into(),
            parameters: vec![Parameter::new("user", Type::named("User"))],
            return_type: Type::Bool,
            body: Expression::greater_than_or_equal(
                Expression::field(Expression::local("user"), "age"),
                Expression::self_field("minimum"),
            ),
        }],
    });
    builder.function(Function {
        name: "can_register".into(),
        parameters: vec![
            Parameter::new("validator", Type::named("Validator")),
            Parameter::new("user", Type::named("User")),
        ],
        return_type: Type::Bool,
        body: Expression::method_call(
            Expression::local("validator"),
            "accepts",
            [Expression::local("user")],
        ),
    });
    builder.function(Function {
        name: "is_adult".into(),
        parameters: vec![Parameter::new("user", Type::named("User"))],
        return_type: Type::Bool,
        body: Expression::greater_than_or_equal(
            Expression::field(Expression::local("user"), "age"),
            Expression::constant("adult_age"),
        ),
    });

    add_test(
        &mut builder,
        "adult is accepted",
        "is_adult",
        vec![user("Alice", 20)],
        true,
    );
    add_test(
        &mut builder,
        "minor is rejected",
        "is_adult",
        vec![user("Bob", 17)],
        false,
    );
    add_test(
        &mut builder,
        "contract accepts adult",
        "can_register",
        vec![validator(18), user("Chloë", 20)],
        true,
    );
    add_test(
        &mut builder,
        "contract rejects minor",
        "can_register",
        vec![validator(18), user("Dora", 17)],
        false,
    );

    builder.finish().unwrap_or_else(|diagnostics| {
        let rendered = diagnostics
            .iter()
            .map(|item| format!("{}: {}", item.code, item.message))
            .collect::<Vec<_>>()
            .join("\n");
        panic!("demonstration module did not check:\n{rendered}")
    })
}

fn add_test(
    builder: &mut ModuleBuilder,
    name: &str,
    function: &str,
    arguments: Vec<Value>,
    expected: bool,
) {
    builder.portable_test(PortableTest {
        name: name.into(),
        function: function.into(),
        arguments,
        expected: Value::Bool(expected),
    });
}

fn user(name: &str, age: i64) -> Value {
    Value::record(
        "User",
        [
            ("name", Value::String(name.into())),
            ("age", Value::I64(age)),
        ],
    )
}

fn validator(minimum: i64) -> Value {
    Value::record("AgeValidator", [("minimum", Value::I64(minimum))])
}
