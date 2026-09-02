#![forbid(unsafe_code)]

//! Rust source backend for the executable prototype.

mod v0;

pub use v0::RustBackend;

use portable_check::CheckedModule;
use portable_codegen::legacy::{Backend, GeneratedFile, GeneratedPackage};
use portable_ir::{Expression, Function, Module, Type, Value};

pub struct LegacyRustBackend;

impl Backend for LegacyRustBackend {
    fn target_name(&self) -> &'static str {
        "rust"
    }

    fn emit(&self, checked: &CheckedModule) -> GeneratedPackage {
        let module = checked.module();
        GeneratedPackage {
            files: vec![GeneratedFile {
                path: "src/lib.rs".into(),
                contents: render_module(module),
            }],
        }
    }
}

fn render_module(module: &Module) -> String {
    let mut output = String::from(
        "#![forbid(unsafe_code)]\n\n// Generated from the checked portable program.\n\n",
    );

    for constant in &module.constants {
        output.push_str(&format!(
            "pub const {}: {} = {};\n\n",
            rust_constant(&constant.name),
            rust_owned_type(&constant.ty),
            rust_value(&constant.value)
        ));
    }

    for record in &module.records {
        output.push_str("#[derive(Clone, Debug, PartialEq, Eq)]\n");
        output.push_str(&format!("pub struct {} {{\n", rust_type_name(&record.name)));
        for field in &record.fields {
            output.push_str(&format!(
                "    pub {}: {},\n",
                rust_identifier(&field.name),
                rust_owned_type(&field.ty)
            ));
        }
        output.push_str("}\n\n");
    }

    for interface in &module.interfaces {
        output.push_str(&format!(
            "pub trait {} {{\n",
            rust_type_name(&interface.name)
        ));
        for method in &interface.methods {
            output.push_str(&format!(
                "    fn {}(&self{}{}) -> {};\n",
                rust_identifier(&method.name),
                if method.parameters.is_empty() {
                    ""
                } else {
                    ", "
                },
                method
                    .parameters
                    .iter()
                    .map(|parameter| rust_parameter(module, parameter))
                    .collect::<Vec<_>>()
                    .join(", "),
                rust_owned_type(&method.return_type)
            ));
        }
        output.push_str("}\n\n");
    }

    for implementation in &module.implementations {
        output.push_str(&format!(
            "impl {} for {} {{\n",
            rust_type_name(&implementation.interface),
            rust_type_name(&implementation.record)
        ));
        for method in &implementation.methods {
            output.push_str(&render_rust_method(module, method));
        }
        output.push_str("}\n\n");
    }

    for function in &module.functions {
        output.push_str(&format!(
            "pub fn {}({}) -> {} {{\n    {}\n}}\n\n",
            rust_identifier(&function.name),
            function
                .parameters
                .iter()
                .map(|parameter| rust_parameter(module, parameter))
                .collect::<Vec<_>>()
                .join(", "),
            rust_owned_type(&function.return_type),
            rust_expression(&function.body)
        ));
    }

    for test in &module.tests {
        output.push_str("#[cfg(test)]\n");
        output.push_str("    #[test]\n");
        output.push_str(&format!("    fn {}() {{\n", rust_test_name(&test.name)));
        for (index, value) in test.arguments.iter().enumerate() {
            output.push_str(&format!(
                "        let argument_{index} = {};\n",
                rust_value(value)
            ));
        }
        let function = module
            .functions
            .iter()
            .find(|function| function.name == test.function)
            .expect("checked tests reference a function");
        let arguments = function
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                if matches!(parameter.ty, Type::Named(_)) {
                    format!("&argument_{index}")
                } else {
                    format!("argument_{index}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let call = format!("{}({})", rust_identifier(&test.function), arguments);
        match test.expected {
            Value::Bool(true) => output.push_str(&format!("        assert!({call});\n")),
            Value::Bool(false) => output.push_str(&format!("        assert!(!{call});\n")),
            _ => output.push_str(&format!(
                "        assert_eq!({call}, {});\n",
                rust_value(&test.expected)
            )),
        }
        output.push_str("    }\n\n");
    }
    output
}

fn render_rust_method(module: &Module, method: &Function) -> String {
    format!(
        "    fn {}(&self{}{}) -> {} {{\n        {}\n    }}\n",
        rust_identifier(&method.name),
        if method.parameters.is_empty() {
            ""
        } else {
            ", "
        },
        method
            .parameters
            .iter()
            .map(|parameter| rust_parameter(module, parameter))
            .collect::<Vec<_>>()
            .join(", "),
        rust_owned_type(&method.return_type),
        rust_expression(&method.body)
    )
}

fn rust_parameter(module: &Module, parameter: &portable_ir::Parameter) -> String {
    let ty = match &parameter.ty {
        Type::Named(name) if module.interfaces.iter().any(|item| item.name == *name) => {
            format!("&dyn {}", rust_type_name(name))
        }
        Type::Named(name) => format!("&{}", rust_type_name(name)),
        other => rust_owned_type(other),
    };
    format!("{}: {ty}", rust_identifier(&parameter.name))
}

fn rust_owned_type(ty: &Type) -> String {
    match ty {
        Type::Bool => "bool".into(),
        Type::I64 => "i64".into(),
        Type::String => "String".into(),
        Type::Named(name) => rust_type_name(name),
    }
}

fn rust_expression(expression: &Expression) -> String {
    match expression {
        Expression::Value(value) => rust_value(value),
        Expression::Local(name) => rust_identifier(name),
        Expression::Constant(name) => rust_constant(name),
        Expression::SelfField(name) => format!("self.{}", rust_identifier(name)),
        Expression::Field { base, field } => {
            format!("{}.{}", rust_expression(base), rust_identifier(field))
        }
        Expression::Compare { left, right, .. } => {
            format!("{} >= {}", rust_expression(left), rust_expression(right))
        }
        Expression::MethodCall {
            receiver,
            method,
            arguments,
        } => format!(
            "{}.{}({})",
            rust_expression(receiver),
            rust_identifier(method),
            arguments
                .iter()
                .map(rust_expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn rust_value(value: &Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::I64(value) => format!("{value}_i64"),
        Value::String(value) => format!("String::from({value:?})"),
        Value::Record { name, fields } => format!(
            "{} {{ {} }}",
            rust_type_name(name),
            fields
                .iter()
                .map(|(name, value)| format!("{}: {}", rust_identifier(name), rust_value(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn rust_constant(name: &str) -> String {
    rust_identifier(&name.to_ascii_uppercase())
}

fn rust_type_name(name: &str) -> String {
    rust_identifier(name)
}

fn rust_test_name(name: &str) -> String {
    rust_identifier(&to_snake_case(name))
}

fn rust_identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn",
    ];
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_owned()
    }
}

fn to_snake_case(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            if character.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_owned()
}
