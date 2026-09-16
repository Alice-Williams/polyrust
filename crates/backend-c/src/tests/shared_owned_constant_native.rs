//! Independent truth, separate translation units, and native readonly controls.
use super::{
    CDialect, CStructuralRenderer,
    bindings::CValueBinding,
    owned_constant_fixture::{Shape, with_registration},
    owned_constant_tests::linked,
};
use portable_codegen::{OutputContents, certify_resolved_package, render_certified_package};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn options(command: &mut Command, optimization: &str) {
    command.args([
        "-std=c17",
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-Werror",
        "-Wstrict-prototypes",
        "-Wmissing-prototypes",
        "-fno-fast-math",
        "-ffp-contract=off",
        "-fsigned-char",
        "-fno-short-enums",
        optimization,
    ]);
}

pub(super) fn successful(command: &mut Command) {
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{command:?}\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

pub(super) fn compile(
    compiler: &Path,
    optimization: &str,
    input: &Path,
    object: &Path,
    include: &Path,
) {
    let mut command = Command::new(compiler);
    options(&mut command, optimization);
    successful(
        command
            .arg("-I")
            .arg(include)
            .arg("-c")
            .arg(input)
            .arg("-o")
            .arg(object),
    );
}

#[test]
fn owned_constants_have_exact_native_values_and_readonly_storage() {
    let version = Command::new("gcc-14")
        .arg("-dumpfullversion")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(String::from_utf8(version.stdout).unwrap().trim(), "14.2.0");
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    // Deliberately independent of the AST fixture's literal list.
    let truth = [
        "0",
        "1",
        "INT32_MIN",
        "INT32_MAX",
        "INT64_MIN",
        "INT64_MAX",
        "INT64_C(9007199254740993)",
        "INT32_C(62)",
    ];
    for shape in [Shape::ConstantsOnly, Shape::Mixed] {
        let fixture = with_registration(shape, |registry, header, exports| {
            registry.register_source_package(header, exports).unwrap();
        });
        let package = linked(&fixture);
        let header = package
            .files()
            .iter()
            .find(|file| file.module() == fixture.objects[0].file())
            .unwrap();
        let names: Vec<_> = fixture
            .objects
            .iter()
            .map(|object| {
                header.items()[0].spelling.values[&CValueBinding::Global(object.clone())]
                    .as_str()
                    .to_owned()
            })
            .collect();
        let functions: Vec<_> = fixture
            .functions
            .iter()
            .map(|function| {
                header.items()[0].spelling.functions[function]
                    .as_str()
                    .to_owned()
            })
            .collect();
        assert_eq!(names.len(), truth.len());
        let certificate = certify_resolved_package(&CDialect, package).unwrap();
        let rendered = render_certified_package(&CStructuralRenderer, &certificate).unwrap();
        let directory = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
            .join("owned-constants")
            .join(format!("{shape:?}"));
        fs::create_dir_all(&directory).unwrap();
        for output in rendered.files() {
            let OutputContents::Text(text) = output.contents() else {
                panic!("C text")
            };
            fs::write(directory.join(output.path()), text).unwrap();
        }
        let mut consumer = String::from(
            "#include \"polyrust_constants.h\"\n#include \"polyrust_constants.h\"\nint main(void) {\n",
        );
        for (name, expected) in names.iter().zip(truth) {
            consumer.push_str(&format!("if ({name} != {expected}) {{ return 1; }}\n"));
        }
        for (name, expected) in functions.iter().zip(truth) {
            consumer.push_str(&format!("if ({name}() != {expected}) {{ return 2; }}\n"));
        }
        consumer.push_str("return 0;\n}\n");
        let consumer_file = directory.join("consumer.c");
        fs::write(&consumer_file, &consumer).unwrap();
        // Export the actual certified output plus the independent executable consumer.
        if let Some(root) = std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR") {
            let output = PathBuf::from(root)
                .join("owned-constants")
                .join(format!("{shape:?}"));
            fs::create_dir_all(&output).unwrap();
            for file in rendered.files() {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("C text")
                };
                fs::write(output.join(file.path()), text).unwrap();
            }
            fs::write(output.join("consumer.c"), &consumer).unwrap();
        }
        for (label, compiler) in [("gcc", PathBuf::from("gcc-14")), ("zig", zig.clone())] {
            for optimization in ["-O0", "-O2"] {
                let round = directory.join(format!("{label}{optimization}"));
                fs::create_dir_all(&round).unwrap();
                let implementation = round.join("constants.o");
                let consumer_object = round.join("consumer.o");
                compile(
                    &compiler,
                    optimization,
                    &directory.join("constants.c"),
                    &implementation,
                    &directory,
                );
                compile(
                    &compiler,
                    optimization,
                    &consumer_file,
                    &consumer_object,
                    &directory,
                );
                let binary = round.join("consumer");
                let mut link = Command::new(&compiler);
                options(&mut link, optimization);
                successful(
                    link.arg(&implementation)
                        .arg(&consumer_object)
                        .arg("-o")
                        .arg(&binary),
                );
                assert!(super::native_stack::run(&binary).success());
                readonly(&compiler, optimization, &directory, &round, &names);
                // Syntactically valid but semantically wrong output must fail the oracle.
                let original = fs::read_to_string(directory.join("constants.c")).unwrap();
                let correct = format!("{} = ", names[7]);
                let line = original
                    .lines()
                    .find(|line| line.contains(&correct))
                    .unwrap();
                let wrong = format!(
                    "{}INT32_C(17);",
                    line.split_once(&correct).unwrap().0.to_owned() + &correct
                );
                let mutated = original.replace(line, &wrong);
                assert_ne!(original, mutated);
                let mutant_source = round.join("mutant.c");
                let mutant_object = round.join("mutant.o");
                fs::write(&mutant_source, mutated).unwrap();
                compile(
                    &compiler,
                    optimization,
                    &mutant_source,
                    &mutant_object,
                    &directory,
                );
                let mut link = Command::new(&compiler);
                options(&mut link, optimization);
                successful(
                    link.arg(&mutant_object)
                        .arg(&consumer_object)
                        .arg("-o")
                        .arg(&binary),
                );
                assert!(
                    !super::native_stack::run(&binary).success(),
                    "truth oracle missed a changed constant"
                );
            }
        }
    }
}

#[cfg(test)]
fn readonly(compiler: &Path, optimization: &str, include: &Path, round: &Path, names: &[String]) {
    for (index, name) in names.iter().enumerate() {
        let input = round.join(format!("readonly-{index}.c"));
        fs::write(
            &input,
            format!(
                "#include \"polyrust_constants.h\"\nint main(void) {{ {name} = 0; return 0; }}\n"
            ),
        )
        .unwrap();
        let mut command = Command::new(compiler);
        options(&mut command, optimization);
        let result = command
            .arg("-I")
            .arg(include)
            .arg("-c")
            .arg(&input)
            .arg("-o")
            .arg(round.join("must-not-compile.o"))
            .output()
            .unwrap();
        assert!(!result.status.success(), "constant became writable: {name}");
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        assert!(diagnostic.contains(name));
        assert!(
            diagnostic.contains("read-only") || diagnostic.contains("const-qualified"),
            "{diagnostic}"
        );
    }
}
