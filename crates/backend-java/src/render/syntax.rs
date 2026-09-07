//! Structural rendering: syntax.

use super::names::render_java_type;
use crate::ast::{
    JavaBinaryOperator, JavaHeritage, JavaIdentifier, JavaLiteral, JavaModifier, JavaResolvedName,
    JavaUnaryOperator, JavaVisibility,
};
use crate::dialect::JavaDialect;
use portable_codegen::TargetSymbolRef;
use portable_diagnostics::Diagnostic;

pub(super) fn render_literal(value: &JavaLiteral) -> String {
    match value {
        JavaLiteral::Boolean(value) => value.to_string(),
        JavaLiteral::I32(value) => value.to_string(),
        JavaLiteral::I64(value) => format!("{value}L"),
        JavaLiteral::CharScalar(value) => value.to_string(),
        JavaLiteral::String(value) => java_string(value),
        JavaLiteral::Utf16Units(values) => {
            let mut result = String::from("\"");
            for value in values {
                result.push_str(&format!("\\u{value:04x}"));
            }
            result.push('"');
            result
        }
        JavaLiteral::InternalNull(_) => "null".to_owned(),
    }
}

fn java_string(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            ch if ch.is_control() => result.push_str(&format!("\\u{:04x}", u32::from(ch))),
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}

pub(super) fn render_heritage(
    value: &JavaHeritage,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
) -> Result<String, Vec<Diagnostic>> {
    match value {
        JavaHeritage::None => Ok(String::new()),
        JavaHeritage::Interfaces(values) => values
            .iter()
            .map(|value| render_java_type(value, names))
            .collect::<Result<Vec<_>, Vec<Diagnostic>>>()
            .map(|values| format!(" implements {}", values.join(", "))),
    }
}

pub(super) fn visibility(value: JavaVisibility) -> &'static str {
    match value {
        JavaVisibility::Public => "public ",
        JavaVisibility::Private => "private ",
        JavaVisibility::Package => "",
    }
}

pub(super) fn modifiers(values: &[JavaModifier]) -> String {
    values
        .iter()
        .map(|value| match value {
            JavaModifier::Public => "public ",
            JavaModifier::Private => "private ",
            JavaModifier::Static => "static ",
            JavaModifier::Final => "final ",
            JavaModifier::Transient => "transient ",
            JavaModifier::Sealed => "sealed ",
            JavaModifier::NonSealed => "non-sealed ",
            JavaModifier::Abstract => "abstract ",
        })
        .collect()
}

pub(super) fn render_type_parameters(values: &[JavaIdentifier]) -> String {
    if values.is_empty() {
        String::new()
    } else {
        format!(
            "<{}>",
            values
                .iter()
                .map(JavaIdentifier::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub(super) fn render_method_type_parameters(values: &[JavaIdentifier]) -> String {
    let value = render_type_parameters(values);
    if value.is_empty() {
        value
    } else {
        format!("{value} ")
    }
}

pub(super) fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

pub(super) fn unary_operator(value: JavaUnaryOperator) -> &'static str {
    match value {
        JavaUnaryOperator::Not => "!",
        JavaUnaryOperator::Negate => "-",
        JavaUnaryOperator::BitNot => "~",
    }
}

pub(super) fn binary_operator(value: JavaBinaryOperator) -> &'static str {
    match value {
        JavaBinaryOperator::LogicalAnd => "&&",
        JavaBinaryOperator::LogicalOr => "||",
        JavaBinaryOperator::Equal => "==",
        JavaBinaryOperator::NotEqual => "!=",
        JavaBinaryOperator::Less => "<",
        JavaBinaryOperator::LessEqual => "<=",
        JavaBinaryOperator::Greater => ">",
        JavaBinaryOperator::GreaterEqual => ">=",
        JavaBinaryOperator::Add => "+",
        JavaBinaryOperator::Subtract => "-",
        JavaBinaryOperator::Multiply => "*",
        JavaBinaryOperator::Divide => "/",
        JavaBinaryOperator::Remainder => "%",
        JavaBinaryOperator::BitAnd => "&",
        JavaBinaryOperator::BitOr => "|",
        JavaBinaryOperator::BitXor => "^",
        JavaBinaryOperator::ShiftLeft => "<<",
        JavaBinaryOperator::ShiftRight => ">>",
    }
}
