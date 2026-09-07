#[test]
fn generated_value_visibility_controls_public_binding_collisions() {
    for visibility in [Visibility::Public, Visibility::Private] {
        let dialect = TestDialect(CatalogueMode::Normal);
        let mut builder = TargetAstBuilder::new(dialect.clone());
        let mut symbols = Vec::new();
        for label in ["first", "second"] {
            symbols.push(builder.value(GeneratedValue {
                name: "value_collision".to_owned(),
                ty: i64_type(),
                visibility: visibility.clone(),
                origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
                source: source(label),
            }));
        }
        let package = builder.build();
        for symbol in &symbols {
            assert_eq!(
                generated_symbol_is_public(&dialect, &package, GeneratedSymbolId::Value(*symbol)),
                visibility == Visibility::Public,
            );
        }
        let mut diagnostics = Vec::new();
        let bindings = allocate_bindings(
            &dialect,
            &package,
            &catalogue(CatalogueMode::Normal),
            &BTreeSet::new(),
            &mut diagnostics,
        );
        if visibility == Visibility::Public {
            assert!(codes(diagnostics).contains(&DiagnosticCode::DuplicateDeclaration));
        } else {
            assert!(diagnostics.is_empty());
            assert_eq!(bindings.len(), 2);
        }
    }
}

#[test]
fn unused_member_catalogue_entries_cannot_disappear_with_the_file_inventory() {
    for omit in [None, Some(0), Some(1)] {
        let dialect = TestDialect(CatalogueMode::Normal);
        let mut builder = TargetAstBuilder::new(dialect.clone());
        let owner = builder.generated_type(GeneratedType {
            name: "Owner".to_owned(),
            kind: DeclarationKind::Record,
            visibility: Visibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source("owner"),
        });
        let method = builder.interface_method(crate::GeneratedInterfaceMethod {
            owner,
            name: "read".to_owned(),
            signature: signature(vec![], i64_type()),
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source("unused-method"),
        });
        let value = builder.value(GeneratedValue {
            name: "VALUE".to_owned(),
            ty: i64_type(),
            visibility: Visibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source("unused-value"),
        });
        let mut items = vec![FileItem::Declaration(GeneratedSymbolId::Type(owner))];
        for (index, symbol) in [
            GeneratedSymbolId::InterfaceMethod(method),
            GeneratedSymbolId::Value(value),
        ]
        .into_iter()
        .enumerate()
        {
            if omit != Some(index) {
                items.push(FileItem::Declaration(symbol));
            }
        }
        builder.file(TargetFile::new(
            RelativeOutputPath::new("src/generated.test").unwrap(),
            SourceRole::Implementation,
            Module::Generated,
            Placement::Implementation,
            items,
            Template::Source,
            source("member-inventory"),
        ));
        let package = builder.build();
        let mut diagnostics = Vec::new();
        derive_and_validate_file_graph(&dialect, &package, &mut [], &mut diagnostics);
        assert_eq!(
            diagnostics.len(),
            usize::from(omit.is_some()),
            "{omit:?}: {diagnostics:?}"
        );
        if omit.is_some() {
            assert!(
                diagnostics[0]
                    .message
                    .contains("not placed in a source file")
            );
        }
    }
}
