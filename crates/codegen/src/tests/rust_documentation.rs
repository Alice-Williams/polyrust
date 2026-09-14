use super::*;

#[path = "rust_documentation/budgets.rs"]
mod budgets;

#[path = "rust_documentation/export_kinds.rs"]
mod export_kinds;

fn id(hash: u64) -> RustDeclarationId {
    RustDeclarationId {
        crate_id: 7,
        definition_path_hash: hash,
    }
}

fn fixture() -> RustSourceOrigin {
    let root = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: RustSourceLocation {
            file: "lib.rs".into(),
            line: 1,
            column: 0,
        },
        documentation: vec!["crate".into()],
    });
    let ancestry: RustModuleAncestry = vec![root.clone()].into();
    RustSourceOrigin {
        declaration: id(2),
        node: RustSourceNode::Declaration,
        module: id(1),
        location: root.location.clone(),
        visibility: RustVisibility::RestrictedTo(id(1)),
        externally_reachable: false,
        documentation: vec!["first".into(), "second".into()],
        module_ancestors: ancestry.clone(),
        crate_exports: Arc::new(RustCrateExports {
            root: id(1),
            modules: BTreeMap::from([(id(1), BTreeMap::new())]),
            module_ancestries: BTreeMap::from([(id(1), ancestry)]),
        }),
    }
}

fn name(text: &str) -> RustExportName {
    RustExportName {
        namespace: RustExportNamespace::Type,
        name: text.into(),
    }
}

fn reject(origin: &RustSourceOrigin, reason: &str) {
    let error = CheckedRustDocumentation::check([origin]).unwrap_err();
    assert!(
        error.to_string().contains(reason),
        "{error:?}, expected {reason}"
    );
}

#[test]
fn shared_and_independent_equal_metadata_have_canonical_ordered_owners() {
    let first = fixture();
    let mut shared = first.clone();
    shared.declaration = id(3);
    let mut independent = fixture();
    independent.declaration = id(4);
    let checked = CheckedRustDocumentation::check([&first, &shared, &independent]).unwrap();
    assert_eq!(checked.modules().count(), 1);
    assert_eq!(
        checked
            .declarations()
            .map(|origin| origin.declaration)
            .collect::<Vec<_>>(),
        vec![id(2), id(3), id(4)]
    );
    assert_eq!(
        checked.declarations().next().unwrap().documentation,
        ["first", "second"]
    );
    assert_eq!(checked.crate_exports().unwrap().root, id(1));
    let empty = CheckedRustDocumentation::check([]).unwrap();
    assert_eq!(empty.modules().count(), 0);
    assert_eq!(empty.declarations().count(), 0);
    assert!(empty.crate_exports().is_none());
}

#[test]
fn private_ancestry_public_alias_cycles_and_foreign_edges_are_finite() {
    let mut origin = fixture();
    let root = origin.module_ancestors[0].clone();
    let child = Arc::new(RustModuleDocumentation {
        declaration: id(3),
        parent: Some(id(1)),
        location: root.location.clone(),
        documentation: vec!["private logical child".into()],
    });
    origin.module = id(3);
    origin.module_ancestors = vec![root, child].into();
    // A private module need not be reachable in the export graph.
    CheckedRustDocumentation::check([&origin]).unwrap();
    let exports = Arc::make_mut(&mut origin.crate_exports);
    exports
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(name("alias"), RustExportTarget::Module(id(3)));
    exports.modules.insert(
        id(3),
        BTreeMap::from([
            (name("back"), RustExportTarget::Module(id(1))),
            (
                name("foreign"),
                RustExportTarget::Module(RustDeclarationId {
                    crate_id: 8,
                    ..id(1)
                }),
            ),
        ]),
    );
    exports
        .module_ancestries
        .insert(id(3), origin.module_ancestors.clone());
    let checked = CheckedRustDocumentation::check([&origin]).unwrap();
    assert_eq!(checked.modules().count(), 2);
    assert_eq!(checked.crate_exports().unwrap().modules.len(), 2);
}

#[test]
fn malformed_owner_and_ancestry_variants_reject() {
    let mut origin = fixture();
    origin.node = RustSourceNode::Binding(0);
    reject(&origin, "body-local");
    let mut origin = fixture();
    origin.declaration.crate_id = 8;
    reject(&origin, "crate roots");
    let mut origin = fixture();
    origin.declaration = id(1);
    reject(&origin, "both a module");
    let mut origin = fixture();
    origin.visibility = RustVisibility::RestrictedTo(id(99));
    reject(&origin, "not an ancestor");
    let mut origin = fixture();
    origin.module = id(99);
    reject(&origin, "owning module");
    let mut origin = fixture();
    origin.module_ancestors = vec![].into();
    reject(&origin, "crate root");
    for (parent, declaration) in [
        (Some(id(9)), id(1)),
        (None, id(9)),
        (
            None,
            RustDeclarationId {
                crate_id: 8,
                ..id(1)
            },
        ),
    ] {
        let mut origin = fixture();
        let mut root = (*origin.module_ancestors[0]).clone();
        root.parent = parent;
        root.declaration = declaration;
        origin.module_ancestors = vec![Arc::new(root)].into();
        assert!(CheckedRustDocumentation::check([&origin]).is_err());
    }
    let mut origin = fixture();
    origin.module_ancestors = vec![origin.module_ancestors[0].clone(); 2].into();
    reject(&origin, "invalid module ancestry");
}

#[test]
fn inconsistent_shared_payloads_and_duplicate_registrations_reject() {
    let first = fixture();
    assert_eq!(
        CheckedRustDocumentation::check([&first, &first]).unwrap_err(),
        RustDocumentationError::DuplicateDeclaration
    );
    let mut second = fixture();
    second.declaration = id(3);
    Arc::make_mut(&mut second.module_ancestors)[0] = Arc::new(RustModuleDocumentation {
        documentation: vec!["different root docs".into()],
        ..(*second.module_ancestors[0]).clone()
    });
    assert!(
        CheckedRustDocumentation::check([&first, &second])
            .unwrap_err()
            .to_string()
            .contains("conflicting module documentation")
    );
    let mut second = fixture();
    second.declaration = id(3);
    Arc::make_mut(&mut second.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(name("different"), RustExportTarget::Declaration(id(2)));
    assert!(
        CheckedRustDocumentation::check([&first, &second])
            .unwrap_err()
            .to_string()
            .contains("conflicting export inventories")
    );
    let mut second = fixture();
    second.declaration = id(3);
    Arc::make_mut(&mut second.crate_exports).root = id(99);
    assert!(
        CheckedRustDocumentation::check([&first, &second])
            .unwrap_err()
            .to_string()
            .contains("conflicting crate roots")
    );
    // Insertion order cannot hide a module/declaration collision.
    let mut second = fixture();
    second.declaration = id(1);
    for origins in [[&first, &second], [&second, &first]] {
        assert!(
            CheckedRustDocumentation::check(origins)
                .unwrap_err()
                .to_string()
                .contains("both a module")
        );
    }
}

#[test]
fn malformed_export_inventory_variants_reject() {
    let mut origin = fixture();
    Arc::make_mut(&mut origin.crate_exports).modules.clear();
    reject(&origin, "ancestry keys");
    let mut origin = fixture();
    Arc::make_mut(&mut origin.crate_exports)
        .module_ancestries
        .clear();
    reject(&origin, "ancestry keys");
    for key in [
        id(99),
        RustDeclarationId {
            crate_id: 8,
            ..id(99)
        },
    ] {
        let mut origin = fixture();
        let exports = Arc::make_mut(&mut origin.crate_exports);
        exports.modules.insert(key, BTreeMap::new());
        exports.module_ancestries.insert(key, vec![].into());
        reject(&origin, "unreachable or foreign");
    }
    let mut origin = fixture();
    Arc::make_mut(&mut origin.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(name("missing"), RustExportTarget::Module(id(99)));
    reject(&origin, "omits a reachable");
    for namespace in [RustExportNamespace::Value, RustExportNamespace::Macro] {
        let mut origin = fixture();
        Arc::make_mut(&mut origin.crate_exports)
            .modules
            .get_mut(&id(1))
            .unwrap()
            .insert(
                RustExportName {
                    namespace,
                    name: "bad".into(),
                },
                RustExportTarget::Module(id(1)),
            );
        reject(&origin, "wrong namespace");
    }
}
