//! Documentation is source presentation, not a JVM constant-pool reservation.
use crate::ast::JavaDocumentation;
use crate::dialect::JavaDialect;
use portable_codegen::LinkedTargetPackage;
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

#[cfg(test)]
#[path = "../tests/documentation_resources.rs"]
mod tests;

#[derive(Clone, Copy)]
struct Limits {
    owners: usize,
    bytes: usize,
}
const LIMITS: Limits = Limits {
    owners: 100_000,
    bytes: 64 * 1024 * 1024,
};

pub(super) fn verify(package: &LinkedTargetPackage<JavaDialect>) -> Vec<Diagnostic> {
    match check(
        package
            .files()
            .iter()
            .flat_map(|file| file.items())
            .map(|item| &item.documentation),
        LIMITS,
    ) {
        Ok(()) => vec![],
        Err(message) => vec![Diagnostic::error(
            DiagnosticCode::TargetResourceLimit,
            message,
            SourceRef::logical(["java", "documentation"]),
        )],
    }
}

fn check<'a>(
    docs: impl IntoIterator<Item = &'a JavaDocumentation>,
    limits: Limits,
) -> Result<(), &'static str> {
    let mut owners = 0usize;
    let mut bytes = 0usize;
    for (_, attachment) in docs.into_iter().flat_map(JavaDocumentation::iter) {
        owners = owners.saturating_add(1);
        bytes = bytes.saturating_add(attachment.presentation_len());
        if owners > limits.owners {
            return Err("Java documentation attachment limit exceeded");
        }
        if bytes > limits.bytes {
            return Err("Java documentation presentation byte limit exceeded");
        }
    }
    Ok(())
}
