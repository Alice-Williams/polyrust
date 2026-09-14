use super::*;

#[test]
fn export_declarations_cannot_impersonate_root_private_or_foreign_modules() {
    let mut origin = fixture();
    Arc::make_mut(&mut origin.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(name("root_as_type"), RustExportTarget::Declaration(id(1)));
    reject(&origin, "both module and declaration");

    let mut origin = fixture();
    let root = origin.module_ancestors[0].clone();
    let private = Arc::new(RustModuleDocumentation {
        declaration: id(3),
        parent: Some(id(1)),
        location: root.location.clone(),
        documentation: vec![],
    });
    origin.module = id(3);
    origin.module_ancestors = vec![root, private].into();
    Arc::make_mut(&mut origin.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(
            name("private_as_type"),
            RustExportTarget::Declaration(id(3)),
        );
    reject(&origin, "both a module and a declaration");

    let foreign = RustDeclarationId {
        crate_id: 8,
        ..id(3)
    };
    let mut origin = fixture();
    let entries = Arc::make_mut(&mut origin.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap();
    entries.insert(name("foreign_module"), RustExportTarget::Module(foreign));
    entries.insert(name("foreign_type"), RustExportTarget::Declaration(foreign));
    reject(&origin, "both module and declaration");
}

#[test]
fn ordinary_declaration_aliases_can_share_an_identity() {
    let mut origin = fixture();
    let entries = Arc::make_mut(&mut origin.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap();
    entries.insert(name("first"), RustExportTarget::Declaration(id(2)));
    entries.insert(name("second"), RustExportTarget::Declaration(id(2)));
    let checked = CheckedRustDocumentation::check([&origin]).unwrap();
    assert_eq!(checked.declarations().count(), 1);
    assert_eq!(checked.crate_exports().unwrap().modules[&id(1)].len(), 2);
}
