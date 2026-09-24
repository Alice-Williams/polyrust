//! Compile only the actual owner output, with no source/runtime dependencies.
use super::{fixture::Owner, model::certify};
use portable_backend_java::dialect::{JavaDialect, JavaStructuralRenderer};
use portable_codegen::{LinkedTargetPackage, OutputContents, render_certified_package};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn javac() -> PathBuf {
    let root = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .unwrap();
    fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .find_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("remotejdk21")
                .then(|| entry.path().join("bin/javac"))
                .filter(|path| path.is_file())
        })
        .expect("Bazel pinned Java21")
}

pub fn prove(verify_metrics: impl Fn(&LinkedTargetPackage<JavaDialect>, &Path)) {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("canonical-owner");
    for maximum in [false, true] {
        let owner = Owner::new(maximum);
        let namespace = owner.namespace.name();
        let certificate = certify(owner.finish()).unwrap();
        let manifest = render_certified_package(&JavaStructuralRenderer, &certificate).unwrap();
        assert_eq!(manifest.files().len(), 1);
        let directory = root.join(if maximum { "maximum" } else { "normal" });
        let classes = directory.join("classes");
        fs::create_dir_all(&classes).unwrap();
        let file = &manifest.files()[0];
        let OutputContents::Text(text) = file.contents() else {
            panic!("source text")
        };
        assert!(
            !text.contains("import "),
            "closed owner has no foreign references"
        );
        let path = directory.join(file.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        let output = Command::new(javac())
            .args([
                "--release",
                "21",
                "-encoding",
                "UTF-8",
                "-Xlint:all",
                "-Werror",
                "-implicit:none",
                "-sourcepath",
                "",
                "-cp",
            ])
            .arg(&classes)
            .arg("-d")
            .arg(&classes)
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            output.status.success() && output.stderr.is_empty(),
            "{output:?}"
        );
        let class_directory = classes.join(namespace.replace('.', "/"));
        let mut names: Vec<_> = fs::read_dir(&class_directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(
            names,
            [
                "Generated$Error.class",
                "Generated$Outcome.class",
                "Generated$Success.class",
                "Generated.class"
            ]
        );
        verify_metrics(certificate.ast(), &class_directory);
        let export = PathBuf::from(std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR").unwrap())
            .join("canonical-owner")
            .join(file.path());
        fs::create_dir_all(export.parent().unwrap()).unwrap();
        fs::write(export, text).unwrap();
    }
}
