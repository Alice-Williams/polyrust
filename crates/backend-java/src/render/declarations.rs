//! Structural rendering: declarations.

use super::expressions::render_expr;
use super::names::{render_java_type, resolved_generated_member_name, resolved_name};
use super::statements::render_block;
use super::syntax::{
    indent, modifiers, render_heritage, render_method_type_parameters, render_type_parameters,
    visibility,
};
use crate::ast::{
    JavaAnnotation, JavaConstructor, JavaDeclarationKind, JavaField, JavaMember, JavaMethod,
    JavaMethodDeclaration, JavaParameter, JavaResolvedName, JavaTypeDeclaration,
};
use crate::dialect::JavaDialect;
use portable_codegen::{GeneratedSymbolId, LinkedFile, TargetSymbolRef};
use portable_diagnostics::Diagnostic;

pub(super) fn render_type(
    value: &JavaTypeDeclaration,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    depth: usize,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    render_type_with_extra(value, names, depth, "", file)
}

pub(super) fn render_type_with_extra(
    value: &JavaTypeDeclaration,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    depth: usize,
    extra_members: &str,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    let indent = indent(depth);
    let visibility = visibility(value.visibility);
    let declaration_modifiers = modifiers(&value.modifiers);
    let type_parameters = render_type_parameters(&value.type_parameters);
    let declaration_name = match value.declared {
        Some(symbol) => resolved_generated_member_name(names, GeneratedSymbolId::Type(symbol))?,
        None => value.name.as_str().to_owned(),
    };
    let heritage = render_heritage(&value.heritage, names)?;
    let mut members = String::new();
    let mut enum_constants = Vec::new();
    for member in &value.members {
        if let JavaMember::EnumConstant(constant) = member {
            enum_constants.push(render_enum_constant(constant, names)?);
        } else {
            members.push_str(&render_member(member, names, depth + 1, file)?);
        }
    }
    members.push_str(extra_members);
    let components = value
        .record_components
        .iter()
        .map(|component| {
            Ok(format!(
                "{} {}",
                render_java_type(&component.ty, names)?,
                component.name.as_str()
            ))
        })
        .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?
        .join(", ");
    let permits = value
        .permits
        .iter()
        .map(|ty| render_java_type(ty, names))
        .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?
        .join(", ");
    match value.kind {
        JavaDeclarationKind::FinalClass => Ok(format!(
            "{indent}{visibility}{declaration_modifiers}final class {declaration_name}{type_parameters}{heritage} {{\n{members}{indent}}}\n"
        )),
        JavaDeclarationKind::Record => Ok(format!(
            "{indent}{visibility}{declaration_modifiers}record {declaration_name}{type_parameters}({components}){heritage} {{\n{members}{indent}}}\n"
        )),
        JavaDeclarationKind::Enum => {
            let constants = enum_constants.join(",\n");
            let constants = constants
                .lines()
                .map(|line| format!("{}{}", "    ".repeat(depth + 1), line))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(format!(
                "{indent}{visibility}{declaration_modifiers}enum {declaration_name} {{\n{constants};\n{members}{indent}}}\n"
            ))
        }
        JavaDeclarationKind::Interface => Ok(format!(
            "{indent}{visibility}{declaration_modifiers}interface {declaration_name}{type_parameters} {{\n{members}{indent}}}\n"
        )),
        JavaDeclarationKind::UninhabitedEnum(_) => Ok(format!(
            "{indent}{visibility}enum {declaration_name}{heritage} {{\n{indent}    ;\n{members}{indent}}}\n"
        )),
        JavaDeclarationKind::SealedInterface => Ok(format!(
            "{indent}{visibility}{declaration_modifiers}sealed interface {declaration_name}{type_parameters} permits {permits} {{\n{members}{indent}}}\n"
        )),
    }
}

pub(super) fn render_member(
    value: &JavaMember,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    depth: usize,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    match value {
        JavaMember::Field(value) => render_field(value, names, depth, file),
        JavaMember::CompileFailField(value) => {
            let field = JavaField {
                declared: None,
                modifiers: value.modifiers.clone(),
                ty: value.expected_type.clone(),
                name: value.name.clone(),
                initializer: Some(value.initializer.clone()),
            };
            render_field(&field, names, depth, file)
        }
        JavaMember::EnumConstant(value) => Ok(format!(
            "{}{}\n",
            indent(depth),
            render_enum_constant(value, names)?
        )),
        JavaMember::Method(value) => render_method(value, names, depth, file),
        JavaMember::Constructor(value) => render_constructor(value, names, depth, file),
        JavaMember::NestedType(value) => render_type(value, names, depth, file),
    }
}

fn render_enum_constant(
    value: &crate::ast::JavaEnumConstant,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
) -> Result<String, Vec<Diagnostic>> {
    resolved_generated_member_name(names, GeneratedSymbolId::Value(value.declared))
}

fn render_field(
    value: &JavaField,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    depth: usize,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    let indent = indent(depth);
    let modifiers = modifiers(&value.modifiers);
    let ty = render_java_type(&value.ty, names)?;
    let initializer = match &value.initializer {
        Some(value) => format!(" = {}", render_expr(value, names, file)?),
        None => String::new(),
    };
    let field_name = match value.declared {
        Some(symbol) => resolved_generated_member_name(names, GeneratedSymbolId::Value(symbol))?,
        None => value.name.as_str().to_owned(),
    };
    Ok(format!(
        "{indent}{modifiers}{ty} {field_name}{initializer};\n"
    ))
}

fn render_method(
    value: &JavaMethod,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    depth: usize,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    let indent = indent(depth);
    let annotations = render_annotations(&value.annotations, depth);
    let modifiers = modifiers(&value.modifiers);
    let type_parameters = render_method_type_parameters(&value.type_parameters);
    let return_type = render_java_type(&value.return_type, names)?;
    let parameters = render_parameters(&value.parameters, names)?;
    let method_name = match value.declared {
        JavaMethodDeclaration::Callable(symbol) => {
            resolved_generated_member_name(names, GeneratedSymbolId::Callable(symbol))?
        }
        JavaMethodDeclaration::Interface(symbol)
        | JavaMethodDeclaration::UninhabitedImplementation(symbol)
        | JavaMethodDeclaration::Implementation {
            interface: symbol, ..
        } => resolved_name(
            names,
            &TargetSymbolRef::Generated(GeneratedSymbolId::InterfaceMethod(symbol)),
        )?,
        JavaMethodDeclaration::Structural => value.name.as_str().to_owned(),
    };
    match &value.body {
        Some(body) => {
            let body = render_block(body, names, depth + 1, file)?;
            Ok(format!(
                "{annotations}{indent}{modifiers}{type_parameters}{return_type} {method_name}({parameters}) {{\n{body}{indent}}}\n"
            ))
        }
        None => Ok(format!(
            "{annotations}{indent}{modifiers}{type_parameters}{return_type} {method_name}({parameters});\n"
        )),
    }
}

fn render_constructor(
    value: &JavaConstructor,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    depth: usize,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    let indent = indent(depth);
    let modifiers = modifiers(&value.modifiers);
    let parameters = render_parameters(&value.parameters, names)?;
    let body = render_block(&value.body, names, depth + 1, file)?;
    Ok(format!(
        "{indent}{modifiers}{}({parameters}) {{\n{body}{indent}}}\n",
        value.name.as_str()
    ))
}

fn render_annotations(values: &[JavaAnnotation], depth: usize) -> String {
    let mut output = String::new();
    let indent = indent(depth);
    for value in values {
        let name = value.simple_name();
        output.push_str(&format!("{indent}@{name}\n"));
    }
    output
}

fn render_parameters(
    values: &[JavaParameter],
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
) -> Result<String, Vec<Diagnostic>> {
    values
        .iter()
        .map(|value| {
            let ty = render_java_type(&value.ty, names)?;
            Ok(format!(
                "{}{ty} {}",
                if value.final_parameter { "final " } else { "" },
                value.name.as_str()
            ))
        })
        .collect::<Result<Vec<_>, Vec<Diagnostic>>>()
        .map(|values| values.join(", "))
}
