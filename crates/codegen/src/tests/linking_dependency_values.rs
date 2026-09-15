// Context-free dependency values link independently of callable identity.
use super::*;
mod fixture {
    include!("linking_dependency_value_fixture.rs");
}
pub(super) use fixture::Value;
use fixture::{Kind, value};

pub(super) fn spec(value: &Value) -> DependencyValueSpec<TestDialect> {
    fixture::spec(value)
}
pub(super) fn catalogue(mode: CatalogueMode) -> Vec<DependencyValueSpec<TestDialect>> {
    fixture::catalogue(mode)
}

pub(super) fn scalar_type() -> KnownTypeSpec<TestDialect> {
    KnownTypeSpec {
        symbol: KnownType::ScalarAlias,
        name: id("ScalarAlias"),
        alias_stem: "ScalarAlias".into(),
        qualified_name: None,
        origin: SymbolOrigin::LanguagePrelude(Prelude::Integer),
        arity: 0,
        policy: DependencyPolicy::Implicit,
        dependency: None,
        source: source("known-scalar"),
    }
}

pub(super) fn verify_type(ty: &TargetTypeRef<TestDialect>) -> Result<(), AstViolation> {
    match ty {
        TargetTypeRef::Primitive(Primitive::Bool | Primitive::I64)
        | TargetTypeRef::Known(KnownType::ScalarAlias) => Ok(()),
        _ => Err(AstViolation::new(
            DiagnosticCode::TypeMismatch,
            "dependency value type is not a dialect scalar",
        )),
    }
}

#[test]
fn known_value_types_require_context_free_catalogue_and_dialect_authority() {
    #[derive(Clone, Copy, Debug)]
    enum InvalidKnown {
        Generic,
        ExternalOrigin,
        PackageRequirement,
        Uncatalogued,
    }
    let dialect = TestDialect(CatalogueMode::Normal);
    let original = dialect.symbol_catalogue();
    assert!(original.verify(&dialect).is_ok());
    for change in [
        InvalidKnown::Generic,
        InvalidKnown::ExternalOrigin,
        InvalidKnown::PackageRequirement,
        InvalidKnown::Uncatalogued,
    ] {
        let mut catalogue = original.clone();
        let entry = catalogue
            .types
            .iter_mut()
            .find(|entry| entry.symbol == KnownType::ScalarAlias)
            .unwrap();
        match change {
            InvalidKnown::Generic => entry.arity = 1,
            InvalidKnown::ExternalOrigin => {
                entry.origin = SymbolOrigin::ExternalPackage(ExternalPackage::Math)
            }
            InvalidKnown::PackageRequirement => {
                entry.dependency = Some(requirement("1", [PackageFeature::Fast]))
            }
            InvalidKnown::Uncatalogued => {
                catalogue
                    .types
                    .retain(|entry| entry.symbol != KnownType::ScalarAlias);
            }
        }
        let mut errors = vec![];
        dependency_values::verify(&catalogue, &dialect, &mut errors);
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("require context-free target types")),
            "{change:?}"
        );
    }
    // Clock is catalogued, zero-arity and standard-library-owned, but is not scalar.
    // Test the language hook independently of witness metadata reconstruction.
    assert!(verify_type(&TargetTypeRef::Known(KnownType::Clock)).is_err());
    let mut catalogue = original;
    catalogue.dependency_values[0].ty = TargetTypeRef::Known(KnownType::Clock);
    let mut errors = vec![];
    dependency_values::verify(&catalogue, &dialect, &mut errors);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("not a dialect scalar"))
    );
    assert!(
        !errors
            .iter()
            .any(|error| error.message.contains("context-free"))
    );
}

fn symbol(kind: Kind) -> TargetSymbolRef<TestDialect> {
    TargetSymbolRef::DependencyValue(value(kind))
}
fn link(
    mode: CatalogueMode,
    symbols: Vec<TargetSymbolRef<TestDialect>>,
) -> Result<LinkedTargetPackage<TestDialect>, Vec<Diagnostic>> {
    TargetLinker::new(TestDialect(mode)).link_ast(&verified(package(mode, symbols)))
}

#[test]
fn primitive_values_preserve_typed_owner_names_and_deduplicate_references() {
    let targets = vec![
        symbol(Kind::First),
        symbol(Kind::Second),
        symbol(Kind::Known),
        symbol(Kind::First),
        TargetSymbolRef::DependencyCallable(dependency_tests::Callable::First),
    ];
    let linked = link(CatalogueMode::Normal, targets.clone()).unwrap();
    assert_eq!(linked, link(CatalogueMode::Normal, targets).unwrap());
    assert!(verify_linked_package(&linked).is_ok());
    for target in [
        symbol(Kind::First),
        symbol(Kind::Second),
        symbol(Kind::Known),
        TargetSymbolRef::DependencyCallable(dependency_tests::Callable::First),
    ] {
        let mut found = false;
        for file in &linked.files {
            let imports = file
                .imports
                .iter()
                .filter(|item| item.symbols.contains(&target))
                .collect::<Vec<_>>();
            assert!(imports.len() <= 1, "duplicate binding for one file/value");
            for import in imports {
                found = true;
                assert_eq!(
                    import.origin,
                    SymbolOrigin::CertifiedDependency(dependency_tests::Owner::First)
                );
                assert_eq!(import.binding, import.original_binding);
                if let TargetSymbolRef::DependencyValue(ref value) = target {
                    assert_eq!(import.binding, spec(value).name);
                }
            }
        }
        assert!(found);
    }
    assert_eq!(
        linked.dependencies,
        link(CatalogueMode::Normal, vec![]).unwrap().dependencies
    );
}

#[test]
fn qualified_values_keep_owners_without_local_bindings_or_imports() {
    let baseline = link(CatalogueMode::Normal, vec![]).unwrap();
    let linked = link(
        CatalogueMode::Normal,
        vec![symbol(Kind::QualifiedFirst), symbol(Kind::QualifiedOther)],
    )
    .unwrap();
    assert_eq!(linked.bindings, baseline.bindings);
    for (file, original) in linked.files.iter().zip(&baseline.files) {
        assert_eq!(file.imports, original.imports);
    }
    for kind in [Kind::QualifiedFirst, Kind::QualifiedOther] {
        let DependencySpelling::Qualified(path) = spec(&value(kind)).spelling else {
            panic!("qualified fixture")
        };
        let found = linked
            .files
            .iter()
            .flat_map(|file| &file.references)
            .filter(|reference| reference.symbol == symbol(kind))
            .collect::<Vec<_>>();
        assert!(!found.is_empty());
        assert!(
            found
                .iter()
                .all(|reference| reference.resolved == ResolvedReference::Qualified(path.clone()))
        );
    }
    assert!(verify_linked_package(&linked).is_ok());
}

#[test]
fn fixed_and_qualified_collisions_include_callables_and_unused_values() {
    for mode in [
        CatalogueMode::DependencyValueOwnedCollision,
        CatalogueMode::DependencyValueCallableCollision,
        CatalogueMode::DependencyValueQualifiedCollision,
        CatalogueMode::DependencyValueInvalidType,
    ] {
        assert!(link(mode, vec![]).is_err(), "{mode:?}");
    }
    assert!(link(CatalogueMode::Normal, vec![symbol(Kind::Missing)]).is_err());
    let separate = link(
        CatalogueMode::DependencyValueSeparateNamespace,
        vec![
            symbol(Kind::CallableCollision),
            TargetSymbolRef::DependencyCallable(dependency_tests::Callable::First),
        ],
    )
    .unwrap();
    assert!(verify_linked_package(&separate).is_ok());
}

mod import_tests {
    include!("linking_dependency_imports.rs");
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    Type,
    Owner,
    Name,
    Spelling,
    Source,
    RemoveUnused,
    Duplicate,
}

#[test]
fn every_reconstructed_metadata_field_is_authoritative() {
    let original = link(CatalogueMode::Normal, vec![symbol(Kind::First)]).unwrap();
    for change in [
        Mutation::Type,
        Mutation::Owner,
        Mutation::Name,
        Mutation::Spelling,
        Mutation::Source,
        Mutation::RemoveUnused,
        Mutation::Duplicate,
    ] {
        let mut linked = original.clone();
        let first = &mut linked.catalogue.dependency_values[0];
        match change {
            Mutation::Type => first.ty = TargetTypeRef::Primitive(Primitive::Bool),
            Mutation::Owner => first.owner = dependency_tests::Owner::Other,
            Mutation::Name => first.name = id("forged_constant"),
            Mutation::Spelling => {
                first.spelling = DependencySpelling::Qualified(QualifiedName::DependencyValueOther)
            }
            Mutation::Source => first.source = source("forged-source"),
            Mutation::RemoveUnused => {
                linked.catalogue.dependency_values.pop();
            }
            Mutation::Duplicate => linked
                .catalogue
                .dependency_values
                .push(spec(&value(Kind::First))),
        }
        if !matches!(change, Mutation::RemoveUnused) {
            assert!(
                linked.catalogue.verify(&linked.dialect).is_err(),
                "{change:?}"
            );
        }
        assert!(verify_linked_package(&linked).is_err(), "{change:?}");
    }
}

#[test]
fn unsupported_values_do_not_smuggle_consumer_local_type_authority() {
    let mut builder = TargetAstBuilder::new(TestDialect(CatalogueMode::Normal));
    let parameter = builder.type_parameter(source("parameter"));
    let generated = builder.generated_type(GeneratedType {
        name: "Foreign".into(),
        kind: DeclarationKind::Record,
        visibility: Visibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
        source: source("generated"),
    });
    for ty in [
        TargetTypeRef::Known(KnownType::Uncatalogued),
        TargetTypeRef::Runtime(RuntimeType::Error),
        TargetTypeRef::Constructed(ConstructedType::List),
        TargetTypeRef::Generated(generated),
        TargetTypeRef::TypeParameter(parameter),
    ] {
        let mut catalogue = TestDialect(CatalogueMode::Normal).symbol_catalogue();
        catalogue.dependency_values[0].ty = ty;
        let errors = catalogue
            .verify(&TestDialect(CatalogueMode::Normal))
            .unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("require context-free target types"))
        );
    }
}

#[test]
fn coordinated_fixed_import_rename_is_not_a_new_dependency_authority() {
    let target = symbol(Kind::First);
    let mut linked = link(CatalogueMode::Normal, vec![target.clone()]).unwrap();
    for file in &mut linked.files {
        for import in &mut file.imports {
            if import.symbols.contains(&target) {
                import.binding = id("alias");
                import.original_binding = id("alias");
            }
        }
        for reference in &mut file.references {
            if reference.symbol == target
                && let ResolvedReference::Imported { binding, .. } = &mut reference.resolved
            {
                *binding = id("alias");
            }
        }
    }
    linked.catalogue.dependency_values[0].name = id("alias");
    assert!(verify_linked_package(&linked).is_err());
}

#[test]
fn valid_catalogue_replacement_and_retargeted_references_cannot_hide_missing_original_uses() {
    let mut linked = link(CatalogueMode::Normal, vec![symbol(Kind::QualifiedFirst)]).unwrap();
    for file in &mut linked.files {
        for reference in &mut file.references {
            if reference.symbol == symbol(Kind::QualifiedFirst) {
                reference.symbol = symbol(Kind::QualifiedOther);
                reference.resolved =
                    ResolvedReference::Qualified(QualifiedName::DependencyValueOther);
            }
        }
        for item in &mut file.items {
            for name in &mut item.references {
                if *name == ResolvedReference::Qualified(QualifiedName::DependencyValueFirst) {
                    *name = ResolvedReference::Qualified(QualifiedName::DependencyValueOther);
                }
            }
        }
    }
    assert!(linked.catalogue.verify(&linked.dialect).is_ok());
    assert!(verify_linked_package(&linked).is_err());
}
