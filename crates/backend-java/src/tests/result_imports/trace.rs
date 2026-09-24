//! Independent values and measured native event traces for typed Result branches.
pub(crate) mod fixture;
mod observer;
use self::fixture::{Mutation, Packages};
use super::fixture::text;
use crate::{
    dialect::JavaDependencyApi,
    tests::{
        budget_oracle,
        source_constants_native::{compile, tool},
    },
};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn success(output: Output) {
    assert!(
        output.status.success() && output.stderr.is_empty(),
        "{output:?}"
    );
}
fn write(root: &Path, path: &str, text: &str) -> PathBuf {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    path
}
fn emit(root: &Path, classes: &Path, api: &JavaDependencyApi) -> PathBuf {
    let package = api.package();
    let file = &package.ast().files()[0];
    let text = text(package);
    assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
    assert!(!text.contains("Runtime."));
    let path = write(root, file.path().as_str(), &text);
    success(compile(&path, classes));
    budget_oracle::verify_directory(
        &classes.join(file.module().name().replace('.', "/")),
        &budget_oracle::collect(package.ast()),
    );
    path
}
fn run(classes: &Path, mutation: Mutation, trace: bool, values: usize, traces: usize) {
    for options in [vec![], vec!["-Xint"]] {
        let output = Command::new(tool("java"))
            .args(&options)
            .arg("-cp")
            .arg(classes)
            .arg("Consumer")
            .arg(trace.to_string())
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            format!("1036 observations; {values} value failures; {traces} trace failures")
        );
        println!(
            "{mutation:?}; JVM {options:?}; instrumented={trace}; {}",
            String::from_utf8_lossy(&output.stdout).trim()
        );
        success(output);
    }
}
fn export(packages: &Packages) {
    let root = PathBuf::from(std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR").unwrap())
        .join("java-result-examples");
    for api in [&packages.family, &packages.producer, &packages.relay] {
        let path = api.package().ast().files()[0].path().as_str();
        write(&root, path, &text(api.package()));
    }
    write(
        &root,
        "README.md",
        "# Generated Java Result branch examples\n\nThese three Generated.java files are the uninstrumented certified family,\nproducer and relay from M35-03A-05A-03C. No runtime or observer is required.\nThe native test compiles each owner separately with pinned Java21 and strict lint.\n",
    );
}

#[test]
fn result_branches_native_measured_effects_detect_compiling_value_preserving_faults() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-result-branches");
    for mutation in [
        Mutation::None,
        Mutation::InactiveArm,
        Mutation::DuplicateScrutinee,
        Mutation::ArmBeforeScrutinee,
        Mutation::TagFlip,
        Mutation::PayloadFlip,
    ] {
        let packages = fixture::packages(mutation);
        let root = root.join(format!("{mutation:?}"));
        let classes = root.join("classes");
        std::fs::create_dir_all(&classes).unwrap();
        emit(&root, &classes, &packages.family);
        let producer_path = emit(&root, &classes, &packages.producer);
        emit(&root, &classes, &packages.relay);
        success(compile(
            &write(
                &root,
                "polyrust/test/TraceObserver.java",
                observer::OBSERVER,
            ),
            &classes,
        ));
        success(compile(
            &write(&root, "Consumer.java", observer::DRIVER),
            &classes,
        ));
        let values = match mutation {
            Mutation::TagFlip => 1036,
            Mutation::PayloadFlip => 518,
            _ => 0,
        };
        // Unmodified controls prove the value oracle independently of instrumentation.
        run(&classes, mutation, false, values, 0);
        if mutation == Mutation::None {
            export(&packages);
        }
        let original = text(packages.producer.package());
        std::fs::write(
            &producer_path,
            observer::instrument(&original, &packages.producer),
        )
        .unwrap();
        success(compile(&producer_path, &classes));
        let traces = match mutation {
            Mutation::None | Mutation::PayloadFlip => 0,
            _ => 1036,
        };
        run(&classes, mutation, true, values, traces);
    }
}
