//! Explicit graph checks cannot depend on a fabricated declaration.
use super::*;

#[test]
fn package_without_declarations_retains_checked_module_metadata() {
    let origin = fixture();
    let checked = CheckedRustDocumentation::check_with_exports(&origin.crate_exports, []).unwrap();
    assert_eq!(checked.declarations().count(), 0);
    assert_eq!(checked.modules().count(), 1);
    assert_eq!(checked.crate_exports(), Some(origin.crate_exports.as_ref()));
    assert_eq!(checked.modules().next().unwrap().documentation, ["crate"]);
}

#[test]
fn explicit_and_inferred_graphs_must_agree_including_documentation() {
    let first = fixture();
    for independent in [false, true] {
        let origin = if independent {
            fixture()
        } else {
            first.clone()
        };
        CheckedRustDocumentation::check_with_exports(&first.crate_exports, [&origin]).unwrap();
    }
    let mut conflicting = fixture();
    Arc::make_mut(&mut conflicting.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(name("new_binding"), RustExportTarget::Declaration(id(9)));
    assert!(
        CheckedRustDocumentation::check_with_exports(&first.crate_exports, [&conflicting])
            .unwrap_err()
            .to_string()
            .contains("conflicting export inventories")
    );
    let mut conflicting = fixture();
    let graph = Arc::make_mut(&mut conflicting.crate_exports);
    let path = graph.module_ancestries.get_mut(&id(1)).unwrap();
    let mut root = (**path.first().unwrap()).clone();
    root.documentation = vec!["changed".into()];
    *path = vec![Arc::new(root)].into();
    assert!(
        CheckedRustDocumentation::check_with_exports(&first.crate_exports, [&conflicting])
            .unwrap_err()
            .to_string()
            .contains("conflicting module documentation")
    );
    let mut wrong_root = (*first.crate_exports).clone();
    wrong_root.root = id(99);
    assert!(CheckedRustDocumentation::check_with_exports(&Arc::new(wrong_root), []).is_err());
}

#[test]
fn declaration_free_graph_validates_cycles_ancestry_and_identity_roles() {
    let origin = fixture();
    let mut graph = (*origin.crate_exports).clone();
    graph
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(name("self_alias"), RustExportTarget::Module(id(1)));
    CheckedRustDocumentation::check_with_exports(&Arc::new(graph.clone()), []).unwrap();
    graph.modules.get_mut(&id(1)).unwrap().insert(
        RustExportName {
            namespace: RustExportNamespace::Value,
            name: "bad".into(),
        },
        RustExportTarget::Declaration(id(1)),
    );
    assert!(
        CheckedRustDocumentation::check_with_exports(&Arc::new(graph), [])
            .unwrap_err()
            .to_string()
            .contains("both module and declaration")
    );
    let mut graph = (*origin.crate_exports).clone();
    graph
        .module_ancestries
        .get_mut(&id(1))
        .unwrap()
        .clone_from(&Vec::new().into());
    assert!(CheckedRustDocumentation::check_with_exports(&Arc::new(graph), []).is_err());
}

#[test]
fn explicit_empty_package_charges_every_graph_and_document_budget() {
    let origin = fixture();
    let mut graph = (*origin.crate_exports).clone();
    graph
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(name("cycle"), RustExportTarget::Module(id(1)));
    let graph = Arc::new(graph);
    let exact = Limits {
        declarations: 0,
        ancestry_nodes: 1,
        ancestry_depth: 1,
        export_modules: 1,
        export_bindings: 1,
        attributes: 1,
        text_bytes: 6 + 5 + 5,
    };
    check_input([], Some(&graph), exact).unwrap();
    let mut cases = vec![];
    for index in 0..6 {
        let mut limit = exact;
        match index {
            0 => limit.ancestry_nodes = 0,
            1 => limit.ancestry_depth = 0,
            2 => limit.export_modules = 0,
            3 => limit.export_bindings = 0,
            4 => limit.attributes = 0,
            5 => limit.text_bytes -= 1,
            _ => unreachable!(),
        }
        cases.push(limit);
    }
    for limit in cases {
        assert!(matches!(
            check_input([], Some(&graph), limit),
            Err(RustDocumentationError::Budget(_))
        ));
    }
}

#[test]
fn explicit_graph_and_matching_origin_share_allocation_budget() {
    let first = fixture();
    let mut limits = Limits::PRODUCTION;
    limits.export_modules = 1;
    check_input([&first], Some(&first.crate_exports), limits).unwrap();
    let independent = fixture();
    assert_eq!(
        check_input([&independent], Some(&first.crate_exports), limits).unwrap_err(),
        RustDocumentationError::Budget("export modules")
    );
}
