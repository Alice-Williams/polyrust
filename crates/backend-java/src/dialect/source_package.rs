//! Explicit facade provenance is descriptive until package certification.
use super::JavaDialect;
use crate::ast::{
    JavaDeclarationKind, JavaFileItem, JavaFilePlacement, JavaPackage, JavaSourcePackage,
    JavaVisibility,
};
use portable_codegen::{
    AstViolation, CheckedRustDocumentation, GeneratedOrigin, RustSourceOrigin, SourceRole,
    SynthesisReason, TargetAstPackage,
};
use portable_diagnostics::DiagnosticCode;

fn error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::InvalidStructure, message)
}

fn explicit(
    package: &TargetAstPackage<JavaDialect>,
) -> Result<Option<&JavaSourcePackage>, AstViolation> {
    let mut selected = None;
    for file in package.files() {
        for item in file.items() {
            let JavaFileItem::Type {
                source_package: Some(source),
                declaration,
                ..
            } = item
            else {
                continue;
            };
            if selected.replace(source).is_some() {
                return Err(error("Java source package has duplicate provenance owners"));
            }
            let namespace = JavaPackage::RustCrate(source.exports().root.crate_id);
            if package.files().len() != 1
                || file.items().len() != 1
                || *file.module() != namespace
                || file.role() != SourceRole::PublicApi
                || *file.placement() != JavaFilePlacement::Main
                || file.path().as_str()
                    != format!(
                        "{}Generated.java",
                        namespace.source_directory(JavaFilePlacement::Main)
                    )
            {
                return Err(error(
                    "explicit Java source package requires its canonical public Generated.java compilation unit",
                ));
            }
            let registration = declaration
                .declared
                .and_then(|id| package.generated_type(id))
                .ok_or_else(|| error("explicit Java source package has no registered facade"))?;
            if registration.name != "Generated"
                || registration.kind != JavaDeclarationKind::FinalClass
                || registration.visibility != JavaVisibility::Public
                || !matches!(
                    registration.origin,
                    GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint)
                )
                || declaration.name.as_str() != "Generated"
                || declaration.kind != JavaDeclarationKind::FinalClass
                || declaration.visibility != JavaVisibility::Public
            {
                return Err(error(
                    "explicit Java source package requires its registered public final PackageEntryPoint facade",
                ));
            }
        }
    }
    Ok(selected)
}

pub(super) fn metadata<'a>(
    package: &'a TargetAstPackage<JavaDialect>,
    origins: impl IntoIterator<Item = &'a RustSourceOrigin>,
) -> Result<CheckedRustDocumentation<'a>, AstViolation> {
    let checked = match explicit(package)? {
        Some(source) => CheckedRustDocumentation::check_with_exports(source.exports(), origins),
        None => CheckedRustDocumentation::check(origins),
    };
    checked.map_err(|problem| {
        let code = if matches!(problem, portable_codegen::RustDocumentationError::Budget(_)) {
            DiagnosticCode::TargetResourceLimit
        } else {
            DiagnosticCode::InvalidStructure
        };
        AstViolation::new(code, problem.to_string())
    })
}

#[cfg(test)]
#[path = "../tests/source_package.rs"]
mod tests;
