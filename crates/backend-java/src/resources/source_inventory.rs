//! Bound retained registration metadata independently from rendered documents.
use crate::ast::{JavaSourceDeclaration, JavaSourceInventory};
use crate::dialect::JavaDialect;
use portable_codegen::LinkedTargetPackage;
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

const MAX_DECLARATIONS: usize = 100_000;
const MAX_PARAMETERS: usize = 1_000_000;
const MAX_NAME_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn verify(package: &LinkedTargetPackage<JavaDialect>) -> Vec<Diagnostic> {
    match check(
        package
            .files()
            .iter()
            .flat_map(|file| file.items())
            .map(|item| &item.source_inventory),
        MAX_DECLARATIONS,
        MAX_PARAMETERS,
        MAX_NAME_BYTES,
    ) {
        Ok(()) => vec![],
        Err(message) => vec![Diagnostic::error(
            DiagnosticCode::TargetResourceLimit,
            message,
            SourceRef::logical(["java", "source-inventory"]),
        )],
    }
}

fn check<'a>(
    inventories: impl IntoIterator<Item = &'a JavaSourceInventory>,
    max_declarations: usize,
    max_parameters: usize,
    max_name_bytes: usize,
) -> Result<(), &'static str> {
    let (mut declarations, mut parameters, mut bytes) = (0usize, 0usize, 0usize);
    for (_, value) in inventories.into_iter().flat_map(JavaSourceInventory::iter) {
        let (name, arity) = match value {
            JavaSourceDeclaration::Type(value) => (&value.name, 0),
            JavaSourceDeclaration::Callable(value) => {
                (&value.name, value.signature.parameters.len())
            }
            JavaSourceDeclaration::InterfaceMethod(value) => {
                (&value.name, value.signature.parameters.len())
            }
            JavaSourceDeclaration::Value(value) => (&value.name, 0),
        };
        declarations = declarations.saturating_add(1);
        parameters = parameters.saturating_add(arity);
        bytes = bytes.saturating_add(name.len());
        if declarations > max_declarations {
            return Err("Java source inventory declaration limit exceeded");
        }
        if parameters > max_parameters {
            return Err("Java source inventory parameter limit exceeded");
        }
        if bytes > max_name_bytes {
            return Err("Java source inventory name byte limit exceeded");
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/source_inventory_resources.rs"]
mod tests;
