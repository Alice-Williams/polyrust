//! Expected bindings are independent of generated C symbol spellings.
use portable_backend_c::ast::*;
use portable_codegen::{
    RustCrateExports, RustDeclarationId, RustExportName, RustExportNamespace as Ns,
    RustExportTarget as Target, RustSourceOrigin, RustVisibility,
};
use std::{collections::BTreeSet, sync::Arc};

fn origin(key: &CDeclarationKey) -> &RustSourceOrigin {
    let CGeneratedOrigin::RustSource(value) = &key.origin else {
        panic!("compiler origin")
    };
    value
}

fn binding(graph: &RustCrateExports, module: RustDeclarationId, ns: Ns, name: &str) -> Target {
    graph.modules[&module][&RustExportName {
        namespace: ns,
        name: name.into(),
    }]
}

fn module(graph: &RustCrateExports, owner: RustDeclarationId, name: &str) -> RustDeclarationId {
    let Target::Module(id) = binding(graph, owner, Ns::Type, name) else {
        panic!("module binding")
    };
    id
}

pub(super) fn check(source: &CSourceFile) -> Arc<RustCrateExports> {
    let mut records = Vec::new();
    let mut function = None;
    for item in source.items() {
        if let CFileItem::Declaration(declaration) = item {
            match declaration.kind() {
                CDeclarationKind::Aggregate {
                    owner: CAggregateRef::Struct(record),
                    members,
                } => {
                    records.push((origin(record.key()), members));
                }
                CDeclarationKind::FunctionPrototype {
                    function: value, ..
                } => {
                    function = Some(origin(value.key()));
                }
                _ => panic!("unexpected declaration"),
            }
        }
    }
    assert_eq!(records.len(), 6);
    let function = function.unwrap();
    let graph = &function.crate_exports;
    let root = graph.root;
    assert_eq!(root, function.module_ancestors[0].declaration);
    assert_eq!(root.crate_id, function.declaration.crate_id);
    assert!(!graph.modules.contains_key(&function.module));
    assert!(function.externally_reachable);
    assert_eq!(graph.modules.len(), 6);
    let alias_a = module(graph, root, "alias_only");
    let alias_b = module(graph, alias_a, "next");
    assert_ne!(alias_a, alias_b);
    assert_eq!(module(graph, alias_b, "next"), alias_a);
    assert_eq!(graph.modules[&alias_a].len(), 2);
    assert_eq!(graph.modules[&alias_b].len(), 1);
    assert_eq!(
        binding(graph, alias_a, Ns::Value, "invoke"),
        Target::Declaration(function.declaration)
    );
    let api = module(graph, root, "api");
    assert_eq!(module(graph, root, "mirror"), api);
    assert_eq!(module(graph, api, "again"), api);
    assert_eq!(module(graph, api, "type"), api);
    assert_eq!(graph.modules[&root].len(), 11);
    assert_eq!(graph.modules[&api].len(), 5);
    for (owner, name) in [(root, "score"), (root, "entry"), (api, "eval")] {
        assert_eq!(
            binding(graph, owner, Ns::Value, name),
            Target::Declaration(function.declaration)
        );
    }
    let visible = records[0].0;
    for (owner, name) in [(root, "Exported"), (api, "Renamed")] {
        assert_eq!(
            binding(graph, owner, Ns::Type, name),
            Target::Declaration(visible.declaration)
        );
    }
    assert!(visible.externally_reachable);
    assert_eq!(visible.visibility, RustVisibility::Public);
    assert_eq!(
        origin(records[0].1[0].key()).visibility,
        RustVisibility::Public
    );
    assert_eq!(
        origin(records[0].1[1].key()).visibility,
        RustVisibility::RestrictedTo(function.module)
    );
    assert_eq!(records[1].0.visibility, RustVisibility::Public);
    assert!(!records[1].0.externally_reachable);
    for index in [2, 3] {
        assert_eq!(
            records[index].0.visibility,
            RustVisibility::RestrictedTo(root)
        );
    }
    for index in [4, 5] {
        assert_eq!(
            records[index].0.visibility,
            RustVisibility::RestrictedTo(function.module)
        );
    }
    let exported: BTreeSet<_> = graph
        .modules
        .values()
        .flat_map(|bindings| bindings.values())
        .copied()
        .collect();
    for (index, (metadata, members)) in records.iter().enumerate() {
        assert!(Arc::ptr_eq(&metadata.crate_exports, graph));
        assert_eq!(metadata.module, function.module);
        if index != 0 {
            assert!(!metadata.externally_reachable);
            assert!(!exported.contains(&Target::Declaration(metadata.declaration)));
        }
        for field in *members {
            assert!(Arc::ptr_eq(&origin(field.key()).crate_exports, graph));
            assert!(!exported.contains(&Target::Declaration(origin(field.key()).declaration)));
        }
    }
    let same = module(graph, root, "same");
    let record = binding(graph, same, Ns::Type, "Name");
    let callable = binding(graph, same, Ns::Value, "name");
    assert_ne!(record, callable);
    assert_eq!(binding(graph, same, Ns::Type, "dual"), record);
    assert_eq!(binding(graph, same, Ns::Value, "dual"), callable);
    assert_eq!(graph.modules[&same].len(), 6);
    assert_eq!(module(graph, api, "peer"), same);
    assert_eq!(module(graph, same, "peer"), api);
    assert_eq!(
        binding(graph, same, Ns::Macro, "dual"),
        binding(graph, root, Ns::Macro, "public_marker")
    );
    let dependency = module(graph, root, "dependency");
    assert_ne!(dependency.crate_id, root.crate_id);
    assert!(!graph.modules.contains_key(&dependency));
    let Target::Declaration(foreign) = binding(graph, root, Ns::Value, "foreign_function") else {
        panic!("dependency function binding");
    };
    assert_eq!(foreign.crate_id, dependency.crate_id);
    let external = module(graph, root, "external");
    assert_eq!(graph.modules[&external].len(), 2);
    assert_eq!(
        binding(graph, external, Ns::Value, "alias"),
        Target::Declaration(function.declaration)
    );
    assert_ne!(binding(graph, external, Ns::Type, "Ticket"), record);
    graph.clone()
}
