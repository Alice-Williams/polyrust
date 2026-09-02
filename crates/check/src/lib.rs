#![forbid(unsafe_code)]

//! Resolution and type checking for the prototype portable model.

/// Resolver, type checker, and capability analysis for versioned portable IR.
pub mod v0;

use std::collections::{BTreeMap, BTreeSet};

use portable_ir::{Expression, Function, Interface, MethodSignature, Module, Record, Type, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}

impl Diagnostic {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// The only input accepted by backends. Its field is deliberately private.
#[derive(Clone, Debug)]
pub struct CheckedModule {
    module: Module,
}

impl CheckedModule {
    pub fn module(&self) -> &Module {
        &self.module
    }
}

pub fn check(module: Module) -> Result<CheckedModule, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    check_module_name(&module.name, &mut diagnostics);
    check_top_level_names(&module, &mut diagnostics);
    check_declared_types(&module, &mut diagnostics);
    check_constants(&module, &mut diagnostics);
    check_implementations(&module, &mut diagnostics);
    check_functions(&module, &mut diagnostics);
    check_tests(&module, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(CheckedModule { module })
    } else {
        Err(diagnostics)
    }
}

fn check_module_name(name: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !valid_identifier(name) {
        diagnostics.push(Diagnostic::new(
            "P0001",
            format!("module name `{name}` is not a portable identifier"),
        ));
    }
}

fn check_top_level_names(module: &Module, diagnostics: &mut Vec<Diagnostic>) {
    let mut names = BTreeSet::new();
    for (kind, name) in module
        .constants
        .iter()
        .map(|item| ("constant", item.name.as_str()))
        .chain(
            module
                .records
                .iter()
                .map(|item| ("record", item.name.as_str())),
        )
        .chain(
            module
                .interfaces
                .iter()
                .map(|item| ("interface", item.name.as_str())),
        )
        .chain(
            module
                .functions
                .iter()
                .map(|item| ("function", item.name.as_str())),
        )
    {
        if !valid_identifier(name) {
            diagnostics.push(Diagnostic::new(
                "P0001",
                format!("{kind} name `{name}` is not a portable identifier"),
            ));
        }
        if !names.insert(name) {
            diagnostics.push(Diagnostic::new(
                "P0002",
                format!("duplicate top-level name `{name}`"),
            ));
        }
    }

    let mut implementations = BTreeSet::new();
    for implementation in &module.implementations {
        let key = (&implementation.interface, &implementation.record);
        if !implementations.insert(key) {
            diagnostics.push(Diagnostic::new(
                "P0003",
                format!(
                    "duplicate implementation of `{}` for `{}`",
                    implementation.interface, implementation.record
                ),
            ));
        }
    }
}

fn check_declared_types(module: &Module, diagnostics: &mut Vec<Diagnostic>) {
    for constant in &module.constants {
        check_type(module, &constant.ty, diagnostics);
    }
    for record in &module.records {
        check_fields(module, record, diagnostics);
    }
    for interface in &module.interfaces {
        let mut names = BTreeSet::new();
        for method in &interface.methods {
            if !names.insert(method.name.as_str()) {
                diagnostics.push(Diagnostic::new(
                    "P0004",
                    format!("duplicate method `{}` in `{}`", method.name, interface.name),
                ));
            }
            check_signature(module, method, diagnostics);
        }
    }
    for function in &module.functions {
        check_function_signature(module, function, diagnostics);
    }
}

fn check_fields(module: &Module, record: &Record, diagnostics: &mut Vec<Diagnostic>) {
    let mut names = BTreeSet::new();
    for field in &record.fields {
        if !valid_identifier(&field.name) {
            diagnostics.push(Diagnostic::new(
                "P0001",
                format!("field name `{}` is not a portable identifier", field.name),
            ));
        }
        if !names.insert(field.name.as_str()) {
            diagnostics.push(Diagnostic::new(
                "P0005",
                format!("duplicate field `{}` in `{}`", field.name, record.name),
            ));
        }
        check_type(module, &field.ty, diagnostics);
    }
}

fn check_signature(
    module: &Module,
    signature: &MethodSignature,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_parameters(module, &signature.parameters, diagnostics);
    check_type(module, &signature.return_type, diagnostics);
}

fn check_function_signature(
    module: &Module,
    function: &Function,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_parameters(module, &function.parameters, diagnostics);
    check_type(module, &function.return_type, diagnostics);
}

fn check_parameters(
    module: &Module,
    parameters: &[portable_ir::Parameter],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = BTreeSet::new();
    for parameter in parameters {
        if !valid_identifier(&parameter.name) {
            diagnostics.push(Diagnostic::new(
                "P0001",
                format!(
                    "parameter name `{}` is not a portable identifier",
                    parameter.name
                ),
            ));
        }
        if !names.insert(parameter.name.as_str()) {
            diagnostics.push(Diagnostic::new(
                "P0006",
                format!("duplicate parameter `{}`", parameter.name),
            ));
        }
        check_type(module, &parameter.ty, diagnostics);
    }
}

fn check_type(module: &Module, ty: &Type, diagnostics: &mut Vec<Diagnostic>) {
    if let Type::Named(name) = ty
        && record(module, name).is_none()
        && interface(module, name).is_none()
    {
        diagnostics.push(Diagnostic::new("P0100", format!("unknown type `{name}`")));
    }
}

fn check_constants(module: &Module, diagnostics: &mut Vec<Diagnostic>) {
    for constant in &module.constants {
        let actual = value_type(module, &constant.value, diagnostics);
        if actual.as_ref() != Some(&constant.ty) {
            diagnostics.push(Diagnostic::new(
                "P0200",
                format!("constant `{}` has the wrong value type", constant.name),
            ));
        }
    }
}

fn check_implementations(module: &Module, diagnostics: &mut Vec<Diagnostic>) {
    for implementation in &module.implementations {
        let Some(interface) = interface(module, &implementation.interface) else {
            diagnostics.push(Diagnostic::new(
                "P0300",
                format!("unknown interface `{}`", implementation.interface),
            ));
            continue;
        };
        let Some(receiver) = record(module, &implementation.record) else {
            diagnostics.push(Diagnostic::new(
                "P0301",
                format!("unknown implementation record `{}`", implementation.record),
            ));
            continue;
        };

        for required in &interface.methods {
            let Some(method) = implementation
                .methods
                .iter()
                .find(|method| method.name == required.name)
            else {
                diagnostics.push(Diagnostic::new(
                    "P0302",
                    format!(
                        "implementation `{}` for `{}` is missing method `{}`",
                        implementation.interface, implementation.record, required.name
                    ),
                ));
                continue;
            };
            if method.parameters != required.parameters
                || method.return_type != required.return_type
            {
                diagnostics.push(Diagnostic::new(
                    "P0303",
                    format!("method `{}` does not match its interface", method.name),
                ));
            }
            check_body(module, method, Some(receiver), diagnostics);
        }
        for method in &implementation.methods {
            if !interface
                .methods
                .iter()
                .any(|required| required.name == method.name)
            {
                diagnostics.push(Diagnostic::new(
                    "P0304",
                    format!("extra method `{}` in interface implementation", method.name),
                ));
            }
        }
    }
}

fn check_functions(module: &Module, diagnostics: &mut Vec<Diagnostic>) {
    for function in &module.functions {
        check_body(module, function, None, diagnostics);
    }
}

fn check_body(
    module: &Module,
    function: &Function,
    self_record: Option<&Record>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let locals = function
        .parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
        .collect();
    let actual = expression_type(module, &function.body, &locals, self_record, diagnostics);
    if actual.as_ref() != Some(&function.return_type) {
        diagnostics.push(Diagnostic::new(
            "P0400",
            format!("body of `{}` does not match its return type", function.name),
        ));
    }
}

fn expression_type(
    module: &Module,
    expression: &Expression,
    locals: &BTreeMap<String, Type>,
    self_record: Option<&Record>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Type> {
    match expression {
        Expression::Value(value) => value_type(module, value, diagnostics),
        Expression::Local(name) => locals.get(name).cloned().or_else(|| {
            diagnostics.push(Diagnostic::new("P0401", format!("unknown local `{name}`")));
            None
        }),
        Expression::Constant(name) => module
            .constants
            .iter()
            .find(|constant| constant.name == *name)
            .map(|constant| constant.ty.clone())
            .or_else(|| {
                diagnostics.push(Diagnostic::new(
                    "P0402",
                    format!("unknown constant `{name}`"),
                ));
                None
            }),
        Expression::SelfField(name) => self_record
            .and_then(|record| record.fields.iter().find(|field| field.name == *name))
            .map(|field| field.ty.clone())
            .or_else(|| {
                diagnostics.push(Diagnostic::new(
                    "P0403",
                    format!("unknown self field `{name}`"),
                ));
                None
            }),
        Expression::Field { base, field } => {
            let base_type = expression_type(module, base, locals, self_record, diagnostics)?;
            let Type::Named(record_name) = base_type else {
                diagnostics.push(Diagnostic::new("P0404", "field access requires a record"));
                return None;
            };
            record(module, &record_name)
                .and_then(|record| record.fields.iter().find(|item| item.name == *field))
                .map(|field| field.ty.clone())
                .or_else(|| {
                    diagnostics.push(Diagnostic::new(
                        "P0405",
                        format!("unknown field `{field}` on `{record_name}`"),
                    ));
                    None
                })
        }
        Expression::Compare { left, right, .. } => {
            let left = expression_type(module, left, locals, self_record, diagnostics);
            let right = expression_type(module, right, locals, self_record, diagnostics);
            if left == Some(Type::I64) && right == Some(Type::I64) {
                Some(Type::Bool)
            } else {
                diagnostics.push(Diagnostic::new(
                    "P0406",
                    "ordered comparison currently requires two I64 operands",
                ));
                None
            }
        }
        Expression::MethodCall {
            receiver,
            method,
            arguments,
        } => {
            let receiver_type =
                expression_type(module, receiver, locals, self_record, diagnostics)?;
            let Type::Named(interface_name) = receiver_type else {
                diagnostics.push(Diagnostic::new(
                    "P0407",
                    "method receiver must have a named interface type",
                ));
                return None;
            };
            let Some(interface) = interface(module, &interface_name) else {
                diagnostics.push(Diagnostic::new(
                    "P0407",
                    format!("`{interface_name}` is not a interface receiver"),
                ));
                return None;
            };
            let Some(signature) = interface.methods.iter().find(|item| item.name == *method) else {
                diagnostics.push(Diagnostic::new(
                    "P0408",
                    format!("interface `{interface_name}` has no method `{method}`"),
                ));
                return None;
            };
            check_expression_arguments(
                module,
                arguments,
                &signature.parameters,
                locals,
                self_record,
                diagnostics,
            );
            Some(signature.return_type.clone())
        }
    }
}

fn check_expression_arguments(
    module: &Module,
    arguments: &[Expression],
    parameters: &[portable_ir::Parameter],
    locals: &BTreeMap<String, Type>,
    self_record: Option<&Record>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if arguments.len() != parameters.len() {
        diagnostics.push(Diagnostic::new("P0410", "wrong method argument count"));
        return;
    }
    for (argument, parameter) in arguments.iter().zip(parameters) {
        let actual = expression_type(module, argument, locals, self_record, diagnostics);
        if actual.as_ref() != Some(&parameter.ty) {
            diagnostics.push(Diagnostic::new(
                "P0411",
                format!("argument for `{}` has the wrong type", parameter.name),
            ));
        }
    }
}

fn check_tests(module: &Module, diagnostics: &mut Vec<Diagnostic>) {
    let mut names = BTreeSet::new();
    for test in &module.tests {
        if !names.insert(test.name.as_str()) {
            diagnostics.push(Diagnostic::new(
                "P0500",
                format!("duplicate portable test name `{}`", test.name),
            ));
        }
        let Some(function) = module
            .functions
            .iter()
            .find(|function| function.name == test.function)
        else {
            diagnostics.push(Diagnostic::new(
                "P0501",
                format!("test calls unknown function `{}`", test.function),
            ));
            continue;
        };
        if test.arguments.len() != function.parameters.len() {
            diagnostics.push(Diagnostic::new("P0502", "wrong test argument count"));
            continue;
        }
        for (argument, parameter) in test.arguments.iter().zip(&function.parameters) {
            let actual = value_type(module, argument, diagnostics);
            if !is_assignable(module, actual.as_ref(), &parameter.ty) {
                diagnostics.push(Diagnostic::new(
                    "P0503",
                    format!("test argument for `{}` has the wrong type", parameter.name),
                ));
            }
        }
        let expected = value_type(module, &test.expected, diagnostics);
        if expected.as_ref() != Some(&function.return_type) {
            diagnostics.push(Diagnostic::new(
                "P0504",
                "test expectation has the wrong type",
            ));
        }
    }
}

fn is_assignable(module: &Module, actual: Option<&Type>, expected: &Type) -> bool {
    if actual == Some(expected) {
        return true;
    }
    let (Some(Type::Named(record)), Type::Named(interface)) = (actual, expected) else {
        return false;
    };
    module
        .implementations
        .iter()
        .any(|item| item.record == *record && item.interface == *interface)
}

fn value_type(module: &Module, value: &Value, diagnostics: &mut Vec<Diagnostic>) -> Option<Type> {
    match value {
        Value::Bool(_) => Some(Type::Bool),
        Value::I64(_) => Some(Type::I64),
        Value::String(_) => Some(Type::String),
        Value::Record { name, fields } => {
            let Some(record) = record(module, name) else {
                diagnostics.push(Diagnostic::new(
                    "P0201",
                    format!("value constructs unknown record `{name}`"),
                ));
                return None;
            };
            let supplied: BTreeMap<_, _> =
                fields.iter().map(|(name, value)| (name, value)).collect();
            if supplied.len() != fields.len() || supplied.len() != record.fields.len() {
                diagnostics.push(Diagnostic::new(
                    "P0202",
                    format!("record value `{name}` has missing or duplicate fields"),
                ));
            }
            for field in &record.fields {
                let Some(value) = supplied.get(&field.name) else {
                    diagnostics.push(Diagnostic::new(
                        "P0202",
                        format!("record value `{name}` is missing field `{}`", field.name),
                    ));
                    continue;
                };
                let actual = value_type(module, value, diagnostics);
                if actual.as_ref() != Some(&field.ty) {
                    diagnostics.push(Diagnostic::new(
                        "P0203",
                        format!("field `{}.{}` has the wrong type", name, field.name),
                    ));
                }
            }
            for field in supplied.keys() {
                if !record.fields.iter().any(|item| item.name == **field) {
                    diagnostics.push(Diagnostic::new(
                        "P0204",
                        format!("record value `{name}` has unknown field `{field}`"),
                    ));
                }
            }
            Some(Type::Named(name.clone()))
        }
    }
}

fn record<'a>(module: &'a Module, name: &str) -> Option<&'a Record> {
    module.records.iter().find(|record| record.name == name)
}

fn interface<'a>(module: &'a Module, name: &str) -> Option<&'a Interface> {
    module
        .interfaces
        .iter()
        .find(|interface| interface.name == name)
}

fn valid_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some('_' | 'a'..='z' | 'A'..='Z'))
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use portable_ir::{Expression, Function, Module, Type};

    use super::check;

    fn empty_module() -> Module {
        Module {
            name: "demo".into(),
            constants: vec![],
            records: vec![],
            interfaces: vec![],
            implementations: vec![],
            functions: vec![],
            tests: vec![],
        }
    }

    #[test]
    fn rejects_an_unresolved_local() {
        let mut module = empty_module();
        module.functions.push(Function {
            name: "broken".into(),
            parameters: vec![],
            return_type: Type::Bool,
            body: Expression::local("missing"),
        });
        let diagnostics = check(module).unwrap_err();
        assert!(diagnostics.iter().any(|item| item.code == "P0401"));
    }
}
