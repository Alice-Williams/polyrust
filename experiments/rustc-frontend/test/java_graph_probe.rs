//! Test-only observations of complete checked graphs; not a bundle publisher.
use super::{CheckedGraph, JavaDependencyApi};
use portable_backend_java::{
    ast::{JavaFileItem, JavaMember, JavaModifier, JavaPrimitive, JavaType},
    dialect::JavaStructuralRenderer,
};
use portable_codegen::{
    OutputContents, RustExportTarget, TargetSymbolRef, render_certified_package,
};
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use std::{collections::BTreeSet, fmt::Write, path::PathBuf};
#[path = "java_graph_manifest_probe.rs"]
mod manifest_probe;
#[path = "java_graph_source_probe.rs"]
mod source_probe;

pub(super) fn owner(tcx: TyCtxt<'_>, api: &JavaDependencyApi) {
    source_probe::check(tcx, api);
    let mut expected = BTreeSet::new();
    let mut origins = crate::source_origin::Cache::default();
    for id in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let identity = crate::source_origin::identity(tcx, id.to_def_id());
        if tcx.effective_visibilities(()).is_exported(id) {
            expected.insert(identity);
            let function = api
                .function(identity)
                .expect("compiler public function mapped");
            let source = crate::source_origin::read(
                tcx,
                &mut origins,
                id.to_def_id(),
                portable_codegen::RustSourceNode::Declaration,
                tcx.def_span(id),
            )
            .unwrap();
            assert_eq!(
                function.source(),
                source.as_ref(),
                "exact compiler docs/exports/origin"
            );
        } else {
            assert!(
                api.function(identity).is_none(),
                "private compiler function exported"
            );
        }
    }
    assert_eq!(
        expected,
        api.functions()
            .map(|function| function.declaration())
            .collect()
    );
}

fn hex(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        write!(encoded, "{byte:02x}").unwrap();
    }
    encoded
}

pub(super) fn write(graph: &CheckedGraph) -> Result<(), String> {
    let destination = PathBuf::from(
        std::env::var_os("POLYRUST_JAVA_GRAPH_PROBE").ok_or("test probe destination missing")?,
    );
    assert!(
        !destination.exists(),
        "test owns a fresh observation directory"
    );
    let owners: Vec<_> = graph
        .crates
        .values()
        .map(|member| portable_java_bundle::Owner {
            key: &member.key,
            api: &member.api,
        })
        .collect();
    let prepared = portable_java_bundle::PreparedBundle::new(graph.root, &owners)?;
    let bundle = prepared.render()?;
    assert_eq!(bundle.files(), prepared.render()?.files());
    assert_eq!(bundle.owner_count(), graph.crates.len());
    assert!(
        bundle
            .files()
            .iter()
            .map(|(_, value)| value.len() as u64)
            .sum::<u64>()
            <= prepared.reserved_bytes()
    );
    assert!(portable_java_bundle::PreparedBundle::new(graph.root, &[]).is_err());
    let mut duplicated = owners.clone();
    duplicated.push(owners[0]);
    assert!(portable_java_bundle::PreparedBundle::new(graph.root, &duplicated).is_err());
    let used = owners
        .iter()
        .find(|o| {
            owners
                .iter()
                .any(|c| c.api.dependencies().any(|d| d.root() == o.api.root()))
        })
        .unwrap();
    let replacement = JavaDependencyApi::from_certificate(used.api.package().clone())?;
    assert_eq!(replacement.root(), used.api.root());
    assert_ne!(replacement.package_identity(), used.api.package_identity());
    let substituted: Vec<_> = owners
        .iter()
        .map(|o| {
            if o.api.root() == replacement.root() {
                portable_java_bundle::Owner {
                    key: o.key,
                    api: &replacement,
                }
            } else {
                *o
            }
        })
        .collect();
    assert!(portable_java_bundle::PreparedBundle::new(graph.root, &substituted).is_err());
    for omitted in 0..owners.len() {
        let incomplete: Vec<_> = owners
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != omitted)
            .map(|(_, owner)| *owner)
            .collect();
        assert!(portable_java_bundle::PreparedBundle::new(graph.root, &incomplete).is_err());
    }
    std::fs::create_dir(&destination).map_err(|error| error.to_string())?;
    let mut inventory = format!("ROOT\t{:016x}\n", graph.root.crate_id);
    for (id, checked) in &graph.crates {
        let api = &checked.api;
        manifest_probe::write(api, &mut inventory);
        let owner = id.crate_id;
        let reserved = api.source_byte_bound()?;
        let rendered = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        for _ in 0..2 {
            assert_eq!(
                rendered,
                render_certified_package(&JavaStructuralRenderer, api.package()).unwrap()
            );
        }
        let [file] = rendered.files() else {
            panic!("one certified owner file")
        };
        let OutputContents::Text(text) = file.contents() else {
            panic!("Java text")
        };
        assert!(
            text.len() as u64 <= reserved,
            "compiler owner source exceeds reservation"
        );
        let path = destination.join(file.path());
        assert!(!path.exists(), "one implementation per owner");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
        writeln!(
            inventory,
            "OWNER\t{owner:016x}\t{:016x}\t{}\t{}",
            id.definition_path_hash,
            hex(&checked.key),
            file.path()
        )
        .unwrap();
        for dependency in api.dependencies() {
            assert_eq!(
                dependency,
                graph.crates[&dependency.root()].api.package_identity()
            );
            writeln!(
                inventory,
                "DEP\t{owner:016x}\t{:016x}",
                dependency.root().crate_id
            )
            .unwrap();
        }
        let exports = &api.functions().next().unwrap().source().crate_exports;
        for (module, bindings) in &exports.modules {
            for (name, target) in bindings {
                if let RustExportTarget::Declaration(target) = target {
                    writeln!(
                        inventory,
                        "EXPORT\t{owner:016x}\t{:016x}\t{}\t{:016x}",
                        module.definition_path_hash,
                        hex(&name.name),
                        target.definition_path_hash
                    )
                    .unwrap();
                }
            }
        }
        for function in api.functions() {
            writeln!(
                inventory,
                "FUNCTION\t{owner:016x}\t{:016x}\t{}",
                function.declaration().definition_path_hash,
                function.path().member().as_str()
            )
            .unwrap();
            for doc in &function.source().documentation {
                writeln!(
                    inventory,
                    "DOC\t{owner:016x}\t{:016x}\t{}",
                    function.declaration().definition_path_hash,
                    hex(doc)
                )
                .unwrap();
            }
        }
        for file in api.package().ast().files() {
            for item in file.items() {
                for symbol in item.names.keys() {
                    if let TargetSymbolRef::DependencyCallable(callable) = symbol {
                        let function = callable.function();
                        let owning = &graph.crates[&function.package_identity().root()].api;
                        assert_eq!(owning.function(function.declaration()), Some(function));
                    }
                }
                if let JavaFileItem::Type { declaration, .. } = &item.item {
                    for member in &declaration.members {
                        if let JavaMember::Method(method) = member
                            && method.modifiers.contains(&JavaModifier::Private)
                            && method.parameters.len() == 1
                            && method.parameters[0].ty == JavaType::primitive(JavaPrimitive::Int)
                        {
                            writeln!(inventory, "PRIVATE\t{owner:016x}\t{}", method.name.as_str())
                                .unwrap();
                        }
                    }
                }
            }
        }
    }
    for (name, contents) in bundle.files() {
        let path = destination.join(name);
        if name.ends_with(".java") {
            assert_eq!(std::fs::read_to_string(path).unwrap(), *contents);
        } else {
            std::fs::write(path, contents).map_err(|error| error.to_string())?;
        }
    }
    std::fs::write(destination.join("graph.probe"), inventory).map_err(|error| error.to_string())
}
