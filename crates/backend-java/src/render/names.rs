//! Structural rendering: names.

use super::*;

pub(super) fn render_java_type(
    value: &JavaType,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
) -> Result<String, Vec<Diagnostic>> {
    match value {
        JavaType::Primitive(value) => Ok(value.keyword().to_owned()),
        JavaType::Boxed(value) => Ok(match value {
            JavaPrimitive::Boolean => "Boolean",
            JavaPrimitive::Byte => "Byte",
            JavaPrimitive::Char => "Character",
            JavaPrimitive::Int => "Integer",
            JavaPrimitive::Long => "Long",
            JavaPrimitive::Double => "Double",
            JavaPrimitive::Void => "Void",
        }
        .to_owned()),
        JavaType::Reference(JavaTypeName::Known(value)) => resolved_type_name(names, *value),
        JavaType::Reference(JavaTypeName::Generated(value)) => resolved_name(
            names,
            &TargetSymbolRef::Generated(GeneratedSymbolId::Type(*value)),
        ),
        JavaType::Array { component, .. } => {
            Ok(format!("{}[]", render_java_type(component, names)?))
        }
        JavaType::Generic { raw, arguments } => {
            let raw = match raw {
                JavaTypeName::Known(value) => resolved_type_name(names, *value)?,
                JavaTypeName::Generated(value) => resolved_name(
                    names,
                    &TargetSymbolRef::Generated(GeneratedSymbolId::Type(*value)),
                )?,
            };
            let arguments = arguments
                .iter()
                .map(|value| render_java_type(value, names))
                .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?
                .join(", ");
            Ok(format!("{raw}<{arguments}>"))
        }
        JavaType::Wildcard { bound: None } => Ok("?".to_owned()),
        JavaType::Wildcard {
            bound: Some((kind, ty)),
        } => Ok(format!(
            "? {} {}",
            match kind {
                JavaWildcardBound::Extends => "extends",
                JavaWildcardBound::Super => "super",
            },
            render_java_type(ty, names)?
        )),
        JavaType::TypeVariable(value) => Ok(value.as_str().to_owned()),
    }
}

pub(super) fn resolved_type_name(
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    value: JavaKnownType,
) -> Result<String, Vec<Diagnostic>> {
    resolved_name(names, &TargetSymbolRef::KnownType(value))
}

pub(super) fn resolved_name(
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    symbol: &TargetSymbolRef<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    names
        .get(symbol)
        .map(|value| match value {
            JavaResolvedName::Local(value) => value.as_str().to_owned(),
            JavaResolvedName::Qualified(value) => value.text().to_owned(),
            JavaResolvedName::GeneratedMember { owner, member } => {
                format!("{}.{}", owner.text(), member.as_str())
            }
            JavaResolvedName::Member { owner, member } => {
                format!("{}.{}", owner.text(), member.text())
            }
        })
        .ok_or_else(|| {
            vec![Diagnostic::error(
                DiagnosticCode::UnresolvedReference,
                "renderer received a symbol without a linker-owned spelling",
                portable_diagnostics::SourceRef::logical(["java-renderer", "symbol"]),
            )]
        })
}

pub(super) fn resolved_generated_member_name(
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    symbol: GeneratedSymbolId,
) -> Result<String, Vec<Diagnostic>> {
    names
        .get(&TargetSymbolRef::Generated(symbol))
        .and_then(|value| match value {
            JavaResolvedName::Local(value)
            | JavaResolvedName::GeneratedMember { member: value, .. } => {
                Some(value.as_str().to_owned())
            }
            JavaResolvedName::Qualified(_) | JavaResolvedName::Member { .. } => None,
        })
        .ok_or_else(|| {
            vec![Diagnostic::error(
                DiagnosticCode::UnresolvedReference,
                "renderer received a generated member without a member spelling",
                portable_diagnostics::SourceRef::logical(["java-renderer", "generated-member"]),
            )]
        })
}
