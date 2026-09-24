use super::{documentation, names::render_java_type, syntax::indent};
use crate::ast::{
    JavaDocumentation, JavaDocumentationOwner, JavaRecordComponent, JavaRecordComponentOrigin,
    JavaResolvedName, JavaTypeDeclaration,
};
use crate::dialect::JavaDialect;
use portable_codegen::{GeneratedTypeId, TargetSymbolRef};
use portable_diagnostics::Diagnostic;
use std::collections::BTreeMap;

pub(super) fn owner(
    owner: Option<GeneratedTypeId>,
    component: &JavaRecordComponent,
) -> Option<JavaDocumentationOwner> {
    match (owner, &component.origin) {
        (Some(owner), JavaRecordComponentOrigin::RustSource(field)) => {
            Some(JavaDocumentationOwner::RecordField {
                owner,
                field: field.origin.declaration,
            })
        }
        (None, JavaRecordComponentOrigin::RustSource(_))
        | (Some(_) | None, JavaRecordComponentOrigin::Core(_))
        | (Some(_) | None, JavaRecordComponentOrigin::Synthesized(_))
        | (Some(_) | None, JavaRecordComponentOrigin::Runtime(_)) => None,
    }
}

pub(super) fn render(
    value: &JavaTypeDeclaration,
    names: &BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    docs: &JavaDocumentation,
    depth: usize,
) -> Result<String, Vec<Diagnostic>> {
    let documented = value.record_components.iter().any(|component| {
        owner(value.declared, component).is_some_and(|owner| docs.get(owner).is_some())
    });
    let mut components = Vec::new();
    for component in &value.record_components {
        let comment = owner(value.declared, component)
            .map_or_else(String::new, |owner| documentation::render(docs, owner));
        components.push(format!(
            "{}{}{} {}",
            comment,
            if documented {
                indent(depth + 1)
            } else {
                String::new()
            },
            render_java_type(&component.ty, names)?,
            component.name.as_str()
        ));
    }
    if documented {
        Ok(format!("\n{}\n{}", components.join(",\n"), indent(depth)))
    } else {
        Ok(components.join(", "))
    }
}
