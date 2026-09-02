#![forbid(unsafe_code)]

mod v0;

pub use v0::GoV0Backend;

/// Legacy Go source backend for the executable prototype.
use portable_check::CheckedModule;
use portable_codegen::legacy::{Backend, GeneratedFile, GeneratedPackage};
use portable_codegen::{
    Document as CodeDocument, ImportGroup, ImportSet, LanguageRenderer, RawText, RenderOptions,
    render,
};
use portable_ir::{Expression, Function, Module, Type, Value};

pub struct GoBackend;

impl Backend for GoBackend {
    fn target_name(&self) -> &'static str {
        "go"
    }

    fn emit(&self, checked: &CheckedModule) -> GeneratedPackage {
        let module = checked.module();
        GeneratedPackage {
            files: vec![
                GeneratedFile {
                    path: "generated.go".into(),
                    contents: render_source(module),
                },
                GeneratedFile {
                    path: "generated_test.go".into(),
                    contents: render_tests(module),
                },
            ],
        }
    }
}

fn render_source(module: &Module) -> String {
    let package = go_package(&module.name);
    let mut output = format!(
        "// Code generated from the checked portable program. DO NOT EDIT.\npackage {package}\n\n"
    );
    for constant in &module.constants {
        output.push_str(&format!(
            "const {} {} = {}\n\n",
            go_exported(&constant.name),
            go_type(&constant.ty),
            go_value(&constant.value)
        ));
    }
    for record in &module.records {
        output.push_str(&format!("type {} struct {{\n", go_exported(&record.name)));
        for field in &record.fields {
            output.push_str(&format!(
                "\t{} {}\n",
                go_exported(&field.name),
                go_type(&field.ty)
            ));
        }
        output.push_str("}\n\n");
    }
    for contract in &module.contracts {
        output.push_str(&format!(
            "type {} interface {{\n",
            go_exported(&contract.name)
        ));
        for method in &contract.methods {
            output.push_str(&format!(
                "\t{}({}) {}\n",
                go_exported(&method.name),
                method
                    .parameters
                    .iter()
                    .map(go_parameter)
                    .collect::<Vec<_>>()
                    .join(", "),
                go_type(&method.return_type)
            ));
        }
        output.push_str("}\n\n");
    }
    for implementation in &module.implementations {
        output.push_str(&format!(
            "var _ {} = {}{{}}\n\n",
            go_exported(&implementation.contract),
            go_exported(&implementation.record)
        ));
        for method in &implementation.methods {
            output.push_str(&render_go_method(&implementation.record, method));
        }
    }
    for function in &module.functions {
        output.push_str(&format!(
            "func {}({}) {} {{\n\treturn {}\n}}\n\n",
            go_exported(&function.name),
            function
                .parameters
                .iter()
                .map(go_parameter)
                .collect::<Vec<_>>()
                .join(", "),
            go_type(&function.return_type),
            go_expression(&function.body)
        ));
    }
    output
}

fn render_go_method(record: &str, method: &Function) -> String {
    format!(
        "func (self {}) {}({}) {} {{\n\treturn {}\n}}\n\n",
        go_exported(record),
        go_exported(&method.name),
        method
            .parameters
            .iter()
            .map(go_parameter)
            .collect::<Vec<_>>()
            .join(", "),
        go_type(&method.return_type),
        go_expression(&method.body)
    )
}

fn render_tests(module: &Module) -> String {
    let package = go_package(&module.name);
    let mut output =
        format!("// Code generated from portable tests. DO NOT EDIT.\npackage {package}\n");
    let mut imports = ImportSet::default();
    if !module.tests.is_empty() {
        imports.require(
            ImportGroup::new(10, "standard").expect("static import group is valid"),
            LegacyGoImport::Testing,
        );
    }
    let import_document = LegacyGoRenderer
        .render_imports(&imports)
        .expect("static legacy imports are renderable");
    let import_text =
        render(&import_document, RenderOptions::default()).expect("dependency document is bounded");
    if !import_text.is_empty() {
        output.push('\n');
        output.push_str(&import_text);
        output.push('\n');
    }
    output.push('\n');
    for test in &module.tests {
        output.push_str(&format!(
            "func Test{}(t *testing.T) {{\n",
            go_exported(&test.name)
        ));
        for (index, value) in test.arguments.iter().enumerate() {
            output.push_str(&format!("\targument{index} := {}\n", go_value(value)));
        }
        let arguments = (0..test.arguments.len())
            .map(|index| format!("argument{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "\tgot := {}({arguments})\n\twant := {}\n\tif got != want {{\n\t\tt.Fatalf(\"{}() = %v, want %v\", got, want)\n\t}}\n}}\n\n",
            go_exported(&test.function),
            go_value(&test.expected),
            go_exported(&test.function)
        ));
    }
    output
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum LegacyGoImport {
    Testing,
}

struct LegacyGoRenderer;

impl LanguageRenderer<LegacyGoImport> for LegacyGoRenderer {
    fn render_imports(&self, imports: &ImportSet<LegacyGoImport>) -> Result<CodeDocument, String> {
        let lines = imports
            .groups()
            .flat_map(|(_, imports)| imports.iter())
            .map(|import| match import {
                LegacyGoImport::Testing => "import \"testing\"",
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(CodeDocument::raw_text(RawText::new(lines)))
    }
}

fn go_parameter(parameter: &portable_ir::Parameter) -> String {
    format!("{} {}", go_local(&parameter.name), go_type(&parameter.ty))
}

fn go_type(ty: &Type) -> String {
    match ty {
        Type::Bool => "bool".into(),
        Type::I64 => "int64".into(),
        Type::String => "string".into(),
        Type::Named(name) => go_exported(name),
    }
}

fn go_expression(expression: &Expression) -> String {
    match expression {
        Expression::Value(value) => go_value(value),
        Expression::Local(name) => go_local(name),
        Expression::Constant(name) => go_exported(name),
        Expression::SelfField(name) => format!("self.{}", go_exported(name)),
        Expression::Field { base, field } => {
            format!("{}.{}", go_expression(base), go_exported(field))
        }
        Expression::Compare { left, right, .. } => {
            format!("{} >= {}", go_expression(left), go_expression(right))
        }
        Expression::MethodCall {
            receiver,
            method,
            arguments,
        } => format!(
            "{}.{}({})",
            go_expression(receiver),
            go_exported(method),
            arguments
                .iter()
                .map(go_expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn go_value(value: &Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::I64(value) => value.to_string(),
        Value::String(value) => go_string(value),
        Value::Record { name, fields } => format!(
            "{}{{{}}}",
            go_exported(name),
            fields
                .iter()
                .map(|(name, value)| format!("{}: {}", go_exported(name), go_value(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn go_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

fn go_package(name: &str) -> String {
    let candidate = go_local(name);
    if candidate.is_empty() {
        "generated".into()
    } else {
        candidate
    }
}

fn go_exported(name: &str) -> String {
    let words = words(name);
    let mut output = String::new();
    for word in words {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            output.push(first.to_ascii_uppercase());
            output.extend(chars.map(|character| character.to_ascii_lowercase()));
        }
    }
    if output.is_empty() {
        "Generated".into()
    } else {
        output
    }
}

fn go_local(name: &str) -> String {
    let exported = go_exported(name);
    let mut chars = exported.chars();
    let Some(first) = chars.next() else {
        return "generated".into();
    };
    let mut output = first.to_ascii_lowercase().to_string();
    output.extend(chars);
    if matches!(
        output.as_str(),
        "break"
            | "default"
            | "func"
            | "interface"
            | "select"
            | "case"
            | "defer"
            | "go"
            | "map"
            | "struct"
            | "chan"
            | "else"
            | "goto"
            | "package"
            | "switch"
            | "const"
            | "fallthrough"
            | "if"
            | "range"
            | "type"
            | "continue"
            | "for"
            | "import"
            | "return"
            | "var"
    ) {
        output.push('_');
    }
    output
}

fn words(name: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    for character in name.chars() {
        if !character.is_ascii_alphanumeric() {
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
        } else if character.is_ascii_uppercase() && !current.is_empty() {
            result.push(std::mem::take(&mut current));
            current.push(character);
        } else {
            current.push(character);
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}
