//! Synthetic symbols resolve through their actual typed declaration owners.

use super::JavaDialect;
use crate::ast::{
    JavaDeclaredPath, JavaFileItem, JavaIdentifier, JavaMember, JavaMethodDeclaration,
    JavaTypeDeclaration,
};
use portable_codegen::{
    AstViolation, GeneratedOrigin, GeneratedSymbolId, SynthesisReason, TargetAstPackage,
};
use portable_diagnostics::DiagnosticCode;

pub(super) fn synthetic_path(
    package: &TargetAstPackage<JavaDialect>,
    symbol: GeneratedSymbolId,
) -> Result<Option<JavaDeclaredPath>, AstViolation> {
    let origin = match symbol {
        GeneratedSymbolId::Type(id) => package.generated_type(id).map(|value| &value.origin),
        GeneratedSymbolId::Callable(id) => package.callable(id).map(|value| &value.origin),
        GeneratedSymbolId::Value(id) => package.value(id).map(|value| &value.origin),
        GeneratedSymbolId::InterfaceMethod(_) => return Ok(None),
    };
    if !matches!(origin, Some(GeneratedOrigin::Synthesized(reason)) if *reason != SynthesisReason::PackageEntryPoint && *reason != SynthesisReason::UninhabitedInterface)
    {
        return Ok(None);
    }
    for file in package.files() {
        for item in file.items() {
            if let JavaFileItem::Type { declaration, .. } = item
                && let Some((owners, member)) = find(declaration, &[], symbol)
            {
                return Ok(Some(JavaDeclaredPath {
                    package: *file.module(),
                    owners,
                    member,
                }));
            }
        }
    }
    Err(AstViolation::new(
        DiagnosticCode::UnresolvedReference,
        "synthetic Java symbol has no authenticated declaration path",
    ))
}

fn find(
    node: &JavaTypeDeclaration,
    parents: &[JavaIdentifier],
    symbol: GeneratedSymbolId,
) -> Option<(Vec<JavaIdentifier>, JavaIdentifier)> {
    if node.declared.map(GeneratedSymbolId::Type) == Some(symbol) {
        return Some((parents.to_vec(), node.name.clone()));
    }
    let mut owners = parents.to_vec();
    owners.push(node.name.clone());
    for member in &node.members {
        let name = match member {
            JavaMember::Field(field)
                if field.declared.map(GeneratedSymbolId::Value) == Some(symbol) =>
            {
                Some(&field.name)
            }
            JavaMember::EnumConstant(value)
                if GeneratedSymbolId::Value(value.declared) == symbol =>
            {
                Some(&value.name)
            }
            JavaMember::Method(method) if matches!(method.declared, JavaMethodDeclaration::Callable(id) if GeneratedSymbolId::Callable(id) == symbol) => {
                Some(&method.name)
            }
            JavaMember::NestedType(child) => {
                if let Some(path) = find(child, &owners, symbol) {
                    return Some(path);
                }
                None
            }
            _ => None,
        };
        if let Some(name) = name {
            return Some((owners, name.clone()));
        }
    }
    None
}
