//! Typed origin-to-document projection; rendering never receives raw attributes.
use super::JavaDialect;
use crate::ast::{
    JavaDocumentation, JavaDocumentationOwner, JavaDocumentationStyle, JavaFileItem,
    JavaTypeDeclaration,
};
use portable_codegen::{
    AstViolation, CheckedRustDocumentation, GeneratedOrigin, GeneratedSymbolId, SynthesisReason,
    TargetAstPackage,
};
use portable_diagnostics::DiagnosticCode;

mod declarations;
mod normalization;
use normalization::Projection;

#[cfg(test)]
#[path = "../tests/source_documentation.rs"]
mod tests;

pub(super) fn verify_presentation_owner(
    package: &TargetAstPackage<JavaDialect>,
    metadata: &CheckedRustDocumentation<'_>,
) -> Vec<AstViolation> {
    if !metadata
        .modules()
        .any(|module| !module.documentation.is_empty())
    {
        return vec![];
    }
    let owners = package.files().flat_map(|file| file.items()).filter(|item| {
        matches!(item, JavaFileItem::Type { declaration, .. } if is_facade(package, declaration))
    }).count();
    if owners == 1 {
        vec![]
    } else {
        vec![error(
            "nonempty source module documentation requires exactly one emitted PackageEntryPoint",
        )]
    }
}

pub(super) fn lower(
    package: &TargetAstPackage<JavaDialect>,
    item: &JavaFileItem,
) -> Result<JavaDocumentation, AstViolation> {
    let mut projection = Projection::default();
    match item {
        JavaFileItem::Type { declaration, .. } => {
            if is_facade(package, declaration) {
                modules(package, &mut projection)?;
            }
            declarations::ty(package, declaration, 0, &mut projection)?;
        }
        JavaFileItem::RuntimeMembers { .. } => {}
    }
    Ok(projection.documentation)
}

fn is_facade(package: &TargetAstPackage<JavaDialect>, declaration: &JavaTypeDeclaration) -> bool {
    declaration
        .declared
        .and_then(|id| package.generated_type(id))
        .is_some_and(|ty| {
            matches!(
                ty.origin,
                GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint)
            )
        })
}

fn modules(
    package: &TargetAstPackage<JavaDialect>,
    projection: &mut Projection,
) -> Result<(), AstViolation> {
    let metadata = CheckedRustDocumentation::check(super::source_registration::origins(package))
        .map_err(|problem| error(&problem.to_string()))?;
    let Some(exports) = metadata.crate_exports() else {
        return Ok(());
    };
    projection.documentation.root = Some(exports.root);
    let modules: std::collections::BTreeMap<_, _> = metadata
        .modules()
        .map(|module| (module.declaration, module))
        .collect();
    let mut order = Vec::new();
    for module in modules.values() {
        let owner = JavaDocumentationOwner::Module(module.declaration);
        let style = if module.declaration == exports.root {
            JavaDocumentationStyle::Declaration
        } else {
            JavaDocumentationStyle::ExplanatoryModule
        };
        projection.attach(owner, 0, style, &module.documentation)?;
        if module.declaration != exports.root && !module.documentation.is_empty() {
            let mut depth = 0;
            let mut parent = module.parent;
            while let Some(id) = parent {
                depth += 1;
                parent = modules
                    .get(&id)
                    .ok_or_else(|| error("module document has no canonical parent"))?
                    .parent;
            }
            order.push((depth, module.declaration));
        }
    }
    order.sort();
    projection.documentation.module_order = order.into_iter().map(|(_, id)| id).collect();
    Ok(())
}

fn symbol_origin(
    package: &TargetAstPackage<JavaDialect>,
    symbol: GeneratedSymbolId,
) -> Option<&GeneratedOrigin<JavaDialect>> {
    match symbol {
        GeneratedSymbolId::Type(id) => package.generated_type(id).map(|value| &value.origin),
        GeneratedSymbolId::Callable(id) => package.callable(id).map(|value| &value.origin),
        GeneratedSymbolId::InterfaceMethod(id) => {
            package.interface_method(id).map(|value| &value.origin)
        }
        GeneratedSymbolId::Value(id) => package.value(id).map(|value| &value.origin),
    }
}

fn error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::InvalidStructure, message)
}
