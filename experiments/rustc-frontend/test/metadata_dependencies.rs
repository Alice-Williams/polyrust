//! Exact staged closure, direct-only externs and bounded filesystem contracts.
#[path = "../src/metadata_dependencies.rs"]
mod metadata_dependencies;
use metadata_dependencies::MetadataDependencies;
use portable_rustc_configuration::graph::{
    CrateDescription, CrateGraph, InputMapping, ResolvedInputs,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn work() -> PathBuf {
    let path = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join(format!(
        "metadata-dependencies-{}",
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("root.rs"), "pub fn identity(v: i32) -> i32 { v }").unwrap();
    fs::create_dir(path.join("stage")).unwrap();
    path
}

fn node(work: &Path, key: &str, path: &Path) -> CrateDescription {
    CrateDescription::new(
        "fixture",
        key,
        InputMapping::new(work.join("root.rs").to_str().unwrap(), "root.rs").unwrap(),
        path.to_str().unwrap(),
    )
    .unwrap()
}

fn graph(work: &Path, imported: &Path) -> (CrateGraph, ResolvedInputs) {
    let root = node(work, "root", &work.join("liboutput.rmeta"))
        .with_dependency("renamed", "leaf")
        .unwrap();
    let source = ResolvedInputs::load(&root).unwrap();
    (
        CrateGraph::new("root", vec![root, node(work, "leaf", imported)]).unwrap(),
        source,
    )
}

#[test]
fn snapshots_exact_closure_and_exposes_only_direct_aliases() {
    let work = work();
    let leaf = work.join("libleaf.rmeta");
    let middle = work.join("libmiddle.rmeta");
    fs::write(&leaf, b"leaf").unwrap();
    fs::write(&middle, b"middle").unwrap();
    let root = node(&work, "root", &work.join("liboutput.rmeta"))
        .with_dependency("renamed", "middle")
        .unwrap()
        .with_dependency("other_alias", "middle")
        .unwrap();
    let source = ResolvedInputs::load(&root).unwrap();
    let graph = CrateGraph::new(
        "root",
        vec![
            root,
            node(&work, "leaf", &leaf),
            node(&work, "middle", &middle)
                .with_dependency("leaf", "leaf")
                .unwrap(),
        ],
    )
    .unwrap();
    let stage = MetadataDependencies::prepare(&graph, &source, &work.join("stage")).unwrap();
    let arguments = stage.compiler_arguments(&graph).unwrap();
    assert_eq!(arguments.iter().filter(|arg| *arg == "--extern").count(), 2);
    assert!(!arguments.iter().any(|arg| arg.starts_with("leaf=")));
    assert!(arguments.iter().any(|arg| arg.starts_with("renamed=")));
    let directory = work.join("stage/dependencies");
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 2);
    let snapshot = directory.join("libfixture-0.rmeta");
    assert_eq!(fs::read(&snapshot).unwrap(), b"leaf");
    fs::write(&leaf, b"changed original").unwrap();
    assert_eq!(fs::read(snapshot).unwrap(), b"leaf");
    drop(stage);
    assert_eq!(fs::read_dir(work.join("stage")).unwrap().count(), 0);
    assert_eq!(fs::read(leaf).unwrap(), b"changed original");
}

#[test]
fn missing_empty_directory_and_oversized_metadata_reject_without_leaks() {
    let work = work();
    let path = work.join("libinput.rmeta");
    let (graph, source) = graph(&work, &path);
    for kind in 0..4 {
        match kind {
            0 => (),
            1 => fs::write(&path, b"").unwrap(),
            2 => {
                fs::remove_file(&path).unwrap();
                fs::create_dir(&path).unwrap();
            }
            _ => {
                fs::remove_dir(&path).unwrap();
                fs::File::create(&path)
                    .unwrap()
                    .set_len(64 * 1024 * 1024 + 1)
                    .unwrap();
            }
        }
        assert!(MetadataDependencies::prepare(&graph, &source, &work.join("stage")).is_err());
        assert_eq!(fs::read_dir(work.join("stage")).unwrap().count(), 0);
    }
}

#[test]
fn symlink_and_hardlink_source_aliases_reject() {
    let work = work();
    for hardlink in [false, true] {
        let path = work.join(format!("libalias-{hardlink}.rmeta"));
        if hardlink {
            fs::hard_link(work.join("root.rs"), &path).unwrap();
        } else {
            std::os::unix::fs::symlink(work.join("root.rs"), &path).unwrap();
        }
        let (graph, source) = graph(&work, &path);
        assert!(
            MetadataDependencies::prepare(&graph, &source, &work.join("stage"))
                .err()
                .unwrap()
                .contains("aliases")
        );
        assert_eq!(fs::read_dir(work.join("stage")).unwrap().count(), 0);
    }
}

#[test]
fn artifact_aliases_under_distinct_keys_reject_and_clean_prior_snapshots() {
    let work = work();
    let original = work.join("liboriginal.rmeta");
    let linked = work.join("liblinked.rmeta");
    fs::write(&original, b"metadata").unwrap();
    fs::hard_link(&original, &linked).unwrap();
    let root = node(&work, "root", &work.join("liboutput.rmeta"))
        .with_dependency("first", "a")
        .unwrap()
        .with_dependency("second", "b")
        .unwrap();
    let source = ResolvedInputs::load(&root).unwrap();
    let graph = CrateGraph::new(
        "root",
        vec![root, node(&work, "a", &original), node(&work, "b", &linked)],
    )
    .unwrap();
    assert!(
        MetadataDependencies::prepare(&graph, &source, &work.join("stage"))
            .err()
            .unwrap()
            .contains("aliases")
    );
    assert_eq!(fs::read_dir(work.join("stage")).unwrap().count(), 0);
}

#[test]
fn preexisting_staging_directory_is_not_adopted_or_removed() {
    let work = work();
    let path = work.join("libinput.rmeta");
    fs::write(&path, b"metadata").unwrap();
    let (graph, source) = graph(&work, &path);
    let directory = work.join("stage/dependencies");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("sentinel"), b"keep").unwrap();
    assert!(MetadataDependencies::prepare(&graph, &source, &work.join("stage")).is_err());
    assert_eq!(fs::read(directory.join("sentinel")).unwrap(), b"keep");
}
