//! Exact canonical type-only publication; source crate rules remain separate.
use super::{JavaDialect, scalar_result};
use crate::ast::{
    JavaCanonicalTypePackage, JavaDeclarationKind, JavaFileItem, JavaFilePlacement, JavaHeritage,
    JavaMember, JavaModifier, JavaPackage, JavaPackageMetadata, JavaSourceInventory, JavaType,
    JavaTypeName, JavaVisibility,
};
use portable_codegen::{
    AstViolation, FileGroupRole, GeneratedOrigin, GeneratedSymbolId, RenderReadyPackage,
    RustIntegerErrorKind, SourceRole, SynthesisReason, TargetAstPackage, TargetFileMember,
};
use portable_diagnostics::DiagnosticCode;

/// The descriptor is retained by an immutable certificate, not inferred from a
/// namespace or fabricated source root.
pub fn java_canonical_type_package(
    package: &RenderReadyPackage<JavaDialect>,
) -> Option<&JavaCanonicalTypePackage> {
    package
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .find_map(|item| match &item.item {
            JavaFileItem::Type {
                package_metadata: Some(JavaPackageMetadata::Canonical(owner)),
                ..
            } => Some(owner.as_ref()),
            _ => None,
        })
}

pub(super) fn verify(package: &TargetAstPackage<JavaDialect>) -> Vec<AstViolation> {
    check(package)
        .err()
        .map(|message| AstViolation::new(DiagnosticCode::InvalidStructure, message))
        .into_iter()
        .collect()
}

fn check(package: &TargetAstPackage<JavaDialect>) -> Result<(), String> {
    let canonical = package.files().any(|file| {
        matches!(file.module(), JavaPackage::CanonicalInstance { .. })
            || file.items().iter().any(|item| {
                matches!(
                    item,
                    JavaFileItem::Type {
                        package_metadata: Some(JavaPackageMetadata::Canonical(_)),
                        ..
                    }
                )
            })
    });
    if !canonical {
        return Ok(());
    }
    if package.files().len() != 1 || package.groups().len() != 1 || package.artifacts().len() != 0 {
        return Err(
            "Java canonical owner requires exactly one source file and public group".into(),
        );
    }
    let file = package.files().next().unwrap();
    let group = package.groups().next().unwrap();
    if group.role() != FileGroupRole::PublicApi
        || group.members().len() != 1
        || !matches!(group.members()[0], TargetFileMember::Source(id) if package.file(id) == Some(file))
    {
        return Err("Java canonical owner group must contain its one original source".into());
    }
    let [
        JavaFileItem::Type {
            declaration,
            package_metadata: Some(JavaPackageMetadata::Canonical(owner)),
            dependencies,
            ..
        },
    ] = file.items()
    else {
        return Err("Java canonical namespace requires one explicit canonical owner item".into());
    };
    let namespace = JavaPackage::CanonicalInstance {
        instance: owner.facts().instance().key(),
        profile: owner.profile(),
    };
    if *file.module() != namespace
        || file.role() != SourceRole::PublicApi
        || *file.placement() != JavaFilePlacement::Main
        || file.path().as_str()
            != format!(
                "{}Generated.java",
                namespace.source_directory(JavaFilePlacement::Main)
            )
    {
        return Err("Java canonical owner namespace, source path or role disagrees".into());
    }
    if dependencies.owners().next().is_some() || dependencies != &Default::default() {
        return Err("Java canonical owner forbids foreign dependency inventory".into());
    }
    if package.generated_types().len() != 4
        || package.values().len() != 6
        || package.callables().len() != 0
        || package.interface_methods().len() != 0
    {
        return Err("Java canonical owner registration inventory must contain only its facade, family and six constants".into());
    }
    let facade = declaration
        .declared
        .and_then(|id| package.generated_type(id))
        .ok_or("Java canonical owner lacks its registered facade")?;
    if facade.name != "Generated"
        || facade.kind != JavaDeclarationKind::FinalClass
        || facade.visibility != JavaVisibility::Public
        || facade.origin != GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint)
        || declaration.name.as_str() != "Generated"
        || declaration.kind != JavaDeclarationKind::FinalClass
        || declaration.visibility != JavaVisibility::Public
        || !declaration.modifiers.is_empty()
        || !declaration.type_parameters.is_empty()
        || !declaration.record_components.is_empty()
        || !declaration.permits.is_empty()
        || declaration.heritage != JavaHeritage::None
    {
        return Err("Java canonical owner requires its exact public Generated facade".into());
    }
    let [
        JavaMember::Constructor(constructor),
        JavaMember::NestedType(interface),
        JavaMember::NestedType(success),
        JavaMember::NestedType(error),
    ] = declaration.members.as_slice()
    else {
        return Err(
            "Java canonical facade permits only its private empty constructor and ordered family"
                .into(),
        );
    };
    if constructor.name != declaration.name
        || constructor.modifiers != [JavaModifier::Private]
        || !constructor.parameters.is_empty()
        || !constructor.body.statements.is_empty()
    {
        return Err(
            "Java canonical facade constructor must be private, empty and parameterless".into(),
        );
    }
    let types = owner.types();
    for (node, id, spelling) in [
        (interface, types.interface, "Outcome"),
        (success, types.success, "Success"),
        (error, types.error, "Error"),
    ] {
        if node.declared != Some(id)
            || node.name.as_str() != spelling
            || package
                .generated_type(id)
                .is_none_or(|value| value.name != spelling)
        {
            return Err(
                "Java canonical family roles must retain exact original registered names".into(),
            );
        }
    }
    let inventory =
        JavaSourceInventory::derive(package, &file.items()[0]).map_err(|e| e.message)?;
    let payload =
        scalar_result::check::check_item(declaration, &inventory, types, Some(owner.kinds()))?
            .ok_or("Java canonical owner lacks its selected local family")?;
    if payload.as_str() != "value"
        || interface.permits
            != [
                JavaType::Reference(JavaTypeName::Generated(types.success)),
                JavaType::Reference(JavaTypeName::Generated(types.error)),
            ]
    {
        return Err("Java canonical payload and permits order must match the fixed profile".into());
    }
    for (member, kind) in error.members.iter().zip(RustIntegerErrorKind::ALL) {
        if !matches!(member, JavaMember::EnumConstant(value) if value.declared == owner.kinds().value(kind))
        {
            return Err("Java canonical error constants must retain semantic-role order".into());
        }
    }
    // All retained metadata must be the exact selected facade/family/value set.
    let expected: std::collections::BTreeSet<_> = [
        GeneratedSymbolId::Type(declaration.declared.unwrap()),
        GeneratedSymbolId::Type(types.interface),
        GeneratedSymbolId::Type(types.success),
        GeneratedSymbolId::Type(types.error),
    ]
    .into_iter()
    .chain(
        RustIntegerErrorKind::ALL.map(|kind| GeneratedSymbolId::Value(owner.kinds().value(kind))),
    )
    .collect();
    if inventory
        .iter()
        .map(|(id, _)| *id)
        .collect::<std::collections::BTreeSet<_>>()
        != expected
    {
        return Err("Java canonical registration inventory differs from selected roles".into());
    }
    Ok(())
}
