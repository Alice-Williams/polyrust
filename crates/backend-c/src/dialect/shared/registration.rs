//! Register one canonical symbol inventory from complete checked source files.
use super::{
    CDialect, CSharedTypeKind, CVisibility,
    bindings::{CBindings, CValueBinding},
};
use crate::ast::{
    CDeclarationKey, CDefinitionKind, CFileItem, CGeneratedOrigin, CLinkage, CRegistry,
    CSourceFile, CSynthesisReason, registry::CRegistered,
};
use portable_codegen::{
    GeneratedCallable, GeneratedOrigin, GeneratedType, GeneratedValue, SynthesisReason,
    TargetAstBuilder,
};
use portable_diagnostics::SourceRef;

pub(super) fn register(
    builder: &mut TargetAstBuilder<CDialect>,
    registrations: &CRegistry,
    sources: &[CSourceFile],
) -> Result<CBindings, String> {
    let mut bindings = CBindings::default();
    let visibility: std::collections::BTreeMap<_, _> = sources
        .iter()
        .flat_map(|source| source.items())
        .filter_map(|item| {
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Function {
                    function, linkage, ..
                } = definition.kind()
            {
                Some((
                    function,
                    match linkage {
                        CLinkage::Internal => CVisibility::Private,
                        CLinkage::External => CVisibility::Exported,
                        CLinkage::None => unreachable!("checked linkage profile"),
                    },
                ))
            } else {
                None
            }
        })
        .collect();
    let inventory = registrations.contextual_inventory();
    for entry in &inventory {
        if let CRegistered::Struct(record) = entry {
            let id = builder.generated_type(GeneratedType {
                name: record.key().name.as_str().into(),
                kind: CSharedTypeKind::Struct,
                visibility: CVisibility::Private,
                origin: origin(record.key()),
                source: location(record.key()),
            });
            bindings.types.insert((*record).clone(), id);
            bindings.reverse_types.insert(id, (*record).clone());
        }
    }
    for entry in inventory {
        let value = match entry {
            CRegistered::Struct(_) | CRegistered::Scope(_) => continue,
            CRegistered::Function(function) => {
                let id = builder.callable(GeneratedCallable {
                    name: function.key().name.as_str().into(),
                    signature: bindings.signature(function),
                    visibility: *visibility
                        .get(function)
                        .ok_or("C callable lacks a definition linkage")?,
                    origin: origin(function.key()),
                    source: location(function.key()),
                });
                bindings.functions.insert(function.clone(), id);
                bindings.reverse_functions.insert(id, function.clone());
                continue;
            }
            CRegistered::Member(member) => CValueBinding::Member(member.clone()),
            CRegistered::Parameter(parameter) => CValueBinding::Parameter(parameter.clone()),
            CRegistered::Local(local) => CValueBinding::Local(local.clone()),
            _ => {
                return Err(
                    "registry contains a category outside the first C shared profile".into(),
                );
            }
        };
        let id = builder.value(GeneratedValue {
            name: value.key().name.as_str().into(),
            ty: bindings.ty(value.ty()),
            visibility: CVisibility::Private,
            origin: origin(value.key()),
            source: location(value.key()),
        });
        bindings.values.insert(value.clone(), id);
        bindings.reverse_values.insert(id, value);
    }
    for (function, _) in registrations.imported_functions() {
        bindings.imports.insert(
            function.clone(),
            super::CImportedCallable::from_registry(registrations, function)?,
        );
    }
    Ok(bindings)
}

fn location(key: &CDeclarationKey) -> SourceRef {
    SourceRef::logical(["c", "declaration", key.name.as_str()])
}

fn origin(key: &CDeclarationKey) -> GeneratedOrigin<CDialect> {
    match &key.origin {
        CGeneratedOrigin::RustSource(value) => GeneratedOrigin::RustSource(value.clone()),
        CGeneratedOrigin::CoreDeclaration(value) => GeneratedOrigin::CoreDeclaration(*value),
        CGeneratedOrigin::CoreExpression(value) => GeneratedOrigin::CoreExpression(*value),
        CGeneratedOrigin::Synthesized(reason) => match reason {
            CSynthesisReason::Runtime => GeneratedOrigin::Runtime(*reason),
            CSynthesisReason::OwnershipAdapter => {
                GeneratedOrigin::Synthesized(SynthesisReason::OwnershipAdapter)
            }
            CSynthesisReason::InterfaceAdapter => {
                GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter)
            }
            CSynthesisReason::EvaluationTemporary => {
                GeneratedOrigin::Synthesized(SynthesisReason::EvaluationTemporary)
            }
            CSynthesisReason::TestHarness => {
                GeneratedOrigin::Synthesized(SynthesisReason::TestHarness)
            }
            CSynthesisReason::PlatformAssertion => {
                GeneratedOrigin::Synthesized(SynthesisReason::PlatformAssertion)
            }
        },
    }
}
