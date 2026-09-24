//! Strict type-only publication. No source exports, helpers or foreign inventory.
use crate::ast::{
    CAggregateRef, CCanonicalTypeRole as Role, CDeclarationKind, CFileItem, CObjectType, CRegistry,
    CScalarType, CSourceFile, registry::CRegistered,
};

pub(super) fn check(registry: &CRegistry, sources: &[CSourceFile]) -> Result<bool, String> {
    let Some(package) = registry.canonical_type_package() else {
        return Ok(false);
    };
    let base = package.profile().basename(package.facts().instance().key());
    if registry.registered_files().count() != 2
        || sources.len() != 2
        || package.header().key().path.as_str() != format!("{base}.h")
        || package.implementation().key().path.as_str() != format!("{base}.c")
        || registry.imported_functions().next().is_some()
        || registry.imported_constants().next().is_some()
        || registry.imported_structs().next().is_some()
    {
        return Err("C canonical type owner requires exact files and no foreign inventory".into());
    }
    let header = sources
        .iter()
        .find(|source| source.identity() == package.header())
        .ok_or("C canonical type owner missing its original header")?;
    let implementation = sources
        .iter()
        .find(|source| source.identity() == package.implementation())
        .ok_or("C canonical type owner missing its original implementation")?;
    if !implementation.items().is_empty() {
        return Err(
            "C canonical type implementation cannot contain declarations or helpers".into(),
        );
    }
    for (role, key) in [
        (Role::Result, package.record().key()),
        (Role::SuccessTag, package.tag().key()),
        (Role::Value, package.value().key()),
    ] {
        let expected = package
            .profile()
            .declaration_key(package.facts().instance().key(), role)
            .map_err(|error| error.to_string())?;
        if *key != expected {
            return Err("C canonical type role spelling or origin differs".into());
        }
    }
    let owner = CAggregateRef::Struct(package.record().clone());
    if package.tag().ty() != &CObjectType::scalar(CScalarType::Bool)
        || package.value().ty() != &CObjectType::scalar(CScalarType::I32)
        || registry
            .members(&owner)
            .map_err(|error| error.to_string())?
            != Some([package.tag().clone(), package.value().clone()].as_slice())
    {
        return Err("C canonical type owner requires its exact Bool/I32 role inventory".into());
    }
    let inventory = registry.contextual_inventory();
    if inventory.len() != 3
        || inventory.iter().any(|item| match item {
            CRegistered::Struct(record) => *record != package.record(),
            CRegistered::Member(member) => *member != package.tag() && *member != package.value(),
            _ => true,
        })
    {
        return Err("C canonical type owner cannot register unrelated declarations".into());
    }
    let mut declared = false;
    let mut previous_assertion = None;
    for item in header.items() {
        match item {
            CFileItem::Declaration(declaration) => {
                if declared
                    || !matches!(declaration.kind(), CDeclarationKind::Aggregate {
                    owner: actual, members } if *actual == owner
                    && members.as_slice() == [package.tag().clone(), package.value().clone()])
                {
                    return Err(
                        "C canonical type header requires its single original declaration".into(),
                    );
                }
                declared = true;
            }
            CFileItem::StaticAssert(assertion) if !declared => {
                let check = super::platform::classify(assertion)?;
                let expected = format!("C profile {:?} {:?}", check.object, check.query);
                if assertion.diagnostic().bytes() != expected.as_bytes()
                    || previous_assertion.is_some_and(|previous| previous >= check)
                {
                    return Err("C canonical type platform assertions must be canonical".into());
                }
                previous_assertion = Some(check);
            }
            _ => return Err("C canonical type header cannot contain unrelated content".into()),
        }
    }
    if !declared {
        return Err("C canonical type declaration missing".into());
    }
    Ok(true)
}

/// Read descriptive placement from an already checked package, not rustc proof.
pub fn c_canonical_type_package(
    package: &portable_codegen::RenderReadyPackage<super::CDialect>,
) -> Option<&crate::ast::CCanonicalTypePackage> {
    package
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .next()
        .and_then(|unit| {
            unit.unit
                .projection
                .registry
                .registrations()
                .canonical_type_package()
        })
}
