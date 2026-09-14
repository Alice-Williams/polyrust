//! Whole-graph filesystem budgets, independent of compiler-internal actions.
#[path = "../src/source_check/inventory.rs"]
mod inventory;

use portable_rustc_configuration::graph::{CrateDescription, CrateGraph, InputMapping};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn directory() -> PathBuf {
    let path = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join(format!(
        "source-inventory-{}",
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("source.rs"), b"pub fn identity() {}").unwrap();
    path
}

fn description(work: &Path, name: &str) -> CrateDescription {
    CrateDescription::new(
        name,
        name,
        InputMapping::new(work.join("source.rs").to_str().unwrap(), "source.rs").unwrap(),
        work.join(format!("lib{name}.rmeta")).to_str().unwrap(),
    )
    .unwrap()
}

fn sparse(path: &Path, size: u64) {
    fs::File::create(path).unwrap().set_len(size).unwrap();
}

#[test]
fn aggregate_metadata_limit_accepts_boundary_and_rejects_one_more_byte() {
    let work = directory();
    let mut root = description(&work, "root");
    let mut crates = Vec::new();
    for index in 0..4 {
        let name = format!("leaf{index}");
        let leaf = description(&work, &name);
        sparse(Path::new(leaf.metadata()), 64 * 1024 * 1024);
        root = root.with_dependency(&name, &name).unwrap();
        crates.push(leaf);
    }
    crates.push(root.clone());
    assert!(inventory::resolve(&CrateGraph::new("root", crates.clone()).unwrap()).is_ok());
    crates.pop();
    let extra = description(&work, "extra");
    sparse(Path::new(extra.metadata()), 1);
    crates.push(extra);
    crates.push(root.with_dependency("extra", "extra").unwrap());
    let result = inventory::resolve(&CrateGraph::new("root", crates).unwrap());
    assert!(result.unwrap_err().contains("exceeds 256 MiB"));
}

#[test]
fn metadata_file_limit_and_nonregular_paths_reject_before_compilation() {
    let work = directory();
    let leaf = description(&work, "leaf");
    let root = description(&work, "root")
        .with_dependency("dep", "leaf")
        .unwrap();
    let graph = CrateGraph::new("root", vec![leaf.clone(), root]).unwrap();
    for size in [0, 64 * 1024 * 1024 + 1] {
        sparse(Path::new(leaf.metadata()), size);
        assert!(
            inventory::resolve(&graph)
                .unwrap_err()
                .contains("within 64 MiB")
        );
    }
    fs::remove_file(leaf.metadata()).unwrap();
    fs::create_dir(leaf.metadata()).unwrap();
    assert!(
        inventory::resolve(&graph)
            .unwrap_err()
            .contains("regular nonempty")
    );
}

#[test]
fn source_sharing_is_allowed_but_metadata_cannot_alias_any_graph_source() {
    let work = directory();
    let left = description(&work, "left");
    let right = description(&work, "right");
    let root = description(&work, "root")
        .with_dependency("left", "left")
        .unwrap()
        .with_dependency("right", "right")
        .unwrap();
    sparse(Path::new(left.metadata()), 1);
    sparse(Path::new(right.metadata()), 1);
    let graph = CrateGraph::new("root", vec![left.clone(), right.clone(), root]).unwrap();
    assert!(inventory::resolve(&graph).is_ok());
    fs::remove_file(left.metadata()).unwrap();
    fs::hard_link(work.join("source.rs"), left.metadata()).unwrap();
    assert!(
        inventory::resolve(&graph)
            .unwrap_err()
            .contains("aliases a source")
    );
    fs::remove_file(left.metadata()).unwrap();
    fs::hard_link(right.metadata(), left.metadata()).unwrap();
    assert!(
        inventory::resolve(&graph)
            .unwrap_err()
            .contains("another artifact")
    );
}

#[test]
fn root_metadata_is_not_a_checking_input_or_output() {
    let work = directory();
    let root = description(&work, "root");
    let graph = CrateGraph::new("root", vec![root.clone()]).unwrap();
    assert!(inventory::resolve(&graph).is_ok());
    assert!(!Path::new(root.metadata()).exists());
    fs::write(root.metadata(), b"untouched root metadata").unwrap();
    assert!(inventory::resolve(&graph).is_ok());
    assert_eq!(
        fs::read(root.metadata()).unwrap(),
        b"untouched root metadata"
    );
}

#[test]
fn toolchain_allowlist_contains_exact_resolved_artifacts_not_directory_prefixes() {
    let work = directory();
    let sysroot = work.join("sysroot");
    let libraries = sysroot.join("lib/rustlib/x86_64-unknown-linux-gnu/lib");
    fs::create_dir_all(&libraries).unwrap();
    let external = work.join("libactual.rmeta");
    fs::write(&external, b"pinned metadata fixture").unwrap();
    std::os::unix::fs::symlink(&external, libraries.join("libpinned.rmeta")).unwrap();
    fs::write(libraries.join("ignored.txt"), b"not metadata").unwrap();
    fs::write(work.join("libambient.rmeta"), b"not in sysroot").unwrap();
    let artifacts = inventory::toolchain_artifacts(&sysroot).unwrap();
    assert_eq!(artifacts.into_iter().collect::<Vec<_>>(), [external]);
    fs::create_dir(libraries.join("directory.rmeta")).unwrap();
    assert!(
        inventory::toolchain_artifacts(&sysroot)
            .unwrap_err()
            .contains("not regular")
    );
}
