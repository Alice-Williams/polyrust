//! Alias-only facade input: source identity without an invented owned definition.
use super::{CDependencyConstant, owned_constant_fixture::Fixture};
use crate::ast::*;
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn facade(crate_id: u64, values: &[CDependencyConstant]) -> Fixture {
    configured(crate_id, values, |_, _| {})
}

/// Configuration mutates unresolved source metadata/import registration only.
/// Projection and certification must still establish every relevant invariant.
pub(super) fn configured(
    crate_id: u64,
    values: &[CDependencyConstant],
    configure: impl FnOnce(&mut CRegistry, &mut RustCrateExports),
) -> Fixture {
    build(crate_id, values, None, configure)
}

pub(super) fn mixed(crate_id: u64, values: &[CDependencyConstant], name: &str) -> Fixture {
    build(crate_id, values, Some(name), |_, _| {})
}

pub(super) fn configured_mixed(
    crate_id: u64,
    values: &[CDependencyConstant],
    name: &str,
    configure: impl FnOnce(&mut CRegistry, &mut RustCrateExports),
) -> Fixture {
    build(crate_id, values, Some(name), configure)
}

fn build(
    crate_id: u64,
    values: &[CDependencyConstant],
    owned_name: Option<&str>,
    configure: impl FnOnce(&mut CRegistry, &mut RustCrateExports),
) -> Fixture {
    let root = RustDeclarationId {
        crate_id,
        definition_path_hash: 1,
    };
    let documentation = Arc::new(RustModuleDocumentation {
        declaration: root,
        parent: None,
        location: RustSourceLocation {
            file: format!("facade_{crate_id}/src/lib.rs"),
            line: 1,
            column: 0,
        },
        documentation: vec![format!("Public constant facade {crate_id}.")],
    });
    let mut exports = RustCrateExports {
        root,
        modules: BTreeMap::from([(
            root,
            values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    (
                        RustExportName {
                            namespace: RustExportNamespace::Value,
                            name: format!("alias_{index}"),
                        },
                        RustExportTarget::Declaration(value.declaration()),
                    )
                })
                .collect(),
        )]),
        module_ancestries: BTreeMap::from([(root, vec![documentation].into())]),
    };
    if let Some(name) = owned_name {
        exports.modules.get_mut(&root).unwrap().insert(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: name.into(),
            },
            RustExportTarget::Declaration(RustDeclarationId {
                definition_path_hash: 1000,
                ..root
            }),
        );
    }
    let mut registry = CRegistry::new();
    // Multiple source aliases do not imply duplicate imported registrations.
    let mut imported = std::collections::BTreeSet::new();
    for value in values {
        if imported.insert(value.clone()) {
            registry.import_constant(value.clone()).unwrap();
        }
    }
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_constant_facade_{crate_id}.h"))
                .unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let source = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_constant_facade_{crate_id}.c"))
                .unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    configure(&mut registry, &mut exports);
    let exports = Arc::new(exports);
    registry
        .register_source_package(&header, exports.clone())
        .unwrap();
    let mut objects = vec![];
    if let Some(name) = owned_name {
        let module = exports.module_ancestries[&root][0].clone();
        objects.push(
            registry
                .register_object(
                    &header,
                    CDeclarationKey {
                        name: CIdentifier::new(name).unwrap(),
                        origin: CGeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                            declaration: RustDeclarationId {
                                definition_path_hash: 1000,
                                ..root
                            },
                            node: RustSourceNode::Declaration,
                            module: root,
                            location: module.location.clone(),
                            visibility: RustVisibility::Public,
                            externally_reachable: true,
                            documentation: vec!["Owned facade value.".into()],
                            module_ancestors: vec![module].into(),
                            crate_exports: exports,
                        })),
                    },
                    CObjectType::scalar(CScalarType::I32)
                        .with_constness(CConstness::Const)
                        .unwrap(),
                )
                .unwrap(),
        );
    }
    let declarations = CDeclarations::new(&registry, header).unwrap();
    let definitions = CDeclarations::new(&registry, source).unwrap();
    let expressions = CExpressions::new(&registry);
    let header_items = objects
        .iter()
        .map(|object| {
            CFileItem::Declaration(declarations.object_declaration(object.clone()).unwrap())
        })
        .collect();
    let source_items = objects
        .iter()
        .map(|object| {
            CFileItem::Definition(
                definitions
                    .object_definition(
                        object.clone(),
                        CLinkage::External,
                        expressions
                            .expression_initializer(
                                expressions
                                    .literal(CLiteral::Signed(CSignedLiteral::I32(42)))
                                    .unwrap(),
                            )
                            .unwrap(),
                    )
                    .unwrap(),
            )
        })
        .collect();
    let files = vec![
        declarations.source_file(header_items).unwrap(),
        definitions.source_file(source_items).unwrap(),
    ];
    Fixture {
        registry: registry.freeze(),
        files,
        objects,
        functions: vec![],
    }
}

pub(super) fn add_module_cycle(exports: &mut RustCrateExports) {
    let root = exports.root;
    let nested = RustDeclarationId {
        definition_path_hash: 2,
        ..root
    };
    let root_doc = exports.module_ancestries[&root][0].clone();
    let nested_doc = Arc::new(RustModuleDocumentation {
        declaration: nested,
        parent: Some(root),
        location: root_doc.location.clone(),
        documentation: vec!["Nested alias namespace.".into()],
    });
    let mut nested_bindings = exports.modules[&root].clone();
    nested_bindings.insert(
        RustExportName {
            namespace: RustExportNamespace::Type,
            name: "parent".into(),
        },
        RustExportTarget::Module(root),
    );
    exports.modules.get_mut(&root).unwrap().insert(
        RustExportName {
            namespace: RustExportNamespace::Type,
            name: "nested".into(),
        },
        RustExportTarget::Module(nested),
    );
    exports.modules.insert(nested, nested_bindings);
    exports
        .module_ancestries
        .insert(nested, vec![root_doc, nested_doc].into());
}
