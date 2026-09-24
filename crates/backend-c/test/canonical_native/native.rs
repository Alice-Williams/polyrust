//! Separate objects, mixed compiler ABI, and compiling typed semantic controls.
use super::{driver::driver, package::*};
use portable_backend_c::dialect::*;
use portable_codegen::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn texts(api: &CDependencyApi) -> Vec<(String, String)> {
    render_certified_package(&CStructuralRenderer, api.package())
        .unwrap()
        .files()
        .iter()
        .map(|file| {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            (file.path().to_owned(), text.clone())
        })
        .collect()
}

fn write(root: &Path, api: &CDependencyApi) {
    for (path, text) in texts(api) {
        fs::write(root.join(path), text).unwrap();
    }
}

fn source_file(api: &CDependencyApi) -> String {
    texts(api)
        .into_iter()
        .find(|(path, _)| path.ends_with(".c"))
        .unwrap()
        .0
}

fn compile(
    compiler: &Path,
    root: &Path,
    source: &str,
    object: &str,
    optimization: &str,
    producer: Option<&CDependencyApi>,
) {
    let mut command = Command::new(compiler);
    command.args([
        "-std=c17",
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-Werror",
        "-Wstrict-prototypes",
        "-Wmissing-prototypes",
        "-fsanitize=undefined",
        "-fno-sanitize-recover=all",
        optimization,
    ]);
    if let Some(api) = producer {
        for role in ["construct", "forward"] {
            let function = function(api, role);
            let name = function.symbol().as_str();
            command.arg(format!("-D{name}=actual_{name}"));
        }
    }
    run(command
        .arg("-c")
        .arg(root.join(source))
        .arg("-o")
        .arg(root.join(object)));
}

fn link_and_check(
    compiler: &Path,
    root: &Path,
    a: &str,
    consumer: &str,
    backward: &str,
    expected: i32,
) {
    let executable = root.join("check");
    let mut command = Command::new(compiler);
    command.args(["-fsanitize=undefined", "-fno-sanitize-recover=all"]);
    for object in ["owner.o", a, "b.o", consumer, backward, "driver.o"] {
        command.arg(root.join(object));
    }
    run(command.arg("-o").arg(&executable));
    let result = Command::new(&executable)
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(expected), "{result:?}");
    assert!(
        result.stderr.is_empty(),
        "controls must fail the oracle, not sanitizers: {result:?}"
    );
    if expected == 0 {
        assert_eq!(
            String::from_utf8(result.stdout).unwrap(),
            "8203 canonical states and ordered cross-producer calls\n"
        );
    } else {
        assert!(result.stdout.is_empty());
    }
}

pub fn prove(
    owner: &CDependencyApi,
    a: &CDependencyApi,
    b: &CDependencyApi,
    consumer: &CDependencyApi,
    backward: &CDependencyApi,
) {
    let proof = owner.structs().next().unwrap();
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("canonical-native");
    fs::create_dir_all(&root).unwrap();
    for api in [owner, a, b, consumer, backward] {
        write(&root, api);
    }
    fs::write(
        root.join("driver.c"),
        driver(owner, a, b, consumer, backward),
    )
    .unwrap();
    let faults = [
        ("tag.c", producer(110, proof, Fault::SwapTag)),
        ("payload.c", producer(110, proof, Fault::ZeroPayload)),
        (
            "duplicate.c",
            package(
                112,
                proof,
                &[],
                &[Operation::Compose {
                    inner: function(a, "construct").into(),
                    outer: function(b, "forward").into(),
                    duplicate: true,
                }],
                false,
            ),
        ),
        (
            "reverse.c",
            package(
                112,
                proof,
                &[],
                &[Operation::Compose {
                    inner: function(b, "construct").into(),
                    outer: function(a, "forward").into(),
                    duplicate: false,
                }],
                false,
            ),
        ),
        (
            "back_duplicate.c",
            package(
                113,
                proof,
                &[],
                &[Operation::Compose {
                    inner: function(b, "construct").into(),
                    outer: function(a, "forward").into(),
                    duplicate: true,
                }],
                false,
            ),
        ),
        (
            "back_reverse.c",
            package(
                113,
                proof,
                &[],
                &[Operation::Compose {
                    inner: function(a, "construct").into(),
                    outer: function(b, "forward").into(),
                    duplicate: false,
                }],
                false,
            ),
        ),
    ];
    for (filename, api) in &faults {
        let (_, text) = texts(api)
            .into_iter()
            .find(|(path, _)| path.ends_with(".c"))
            .unwrap();
        fs::write(root.join(filename), text).unwrap();
    }
    let gcc = PathBuf::from("gcc-14");
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    let mut rounds = 0;
    for (left, right) in [(&gcc, &gcc), (&gcc, &zig), (&zig, &gcc), (&zig, &zig)] {
        for optimization in ["-O0", "-O2"] {
            compile(
                left,
                &root,
                &source_file(owner),
                "owner.o",
                optimization,
                None,
            );
            compile(left, &root, &source_file(a), "a.o", optimization, Some(a));
            compile(right, &root, &source_file(b), "b.o", optimization, Some(b));
            compile(
                right,
                &root,
                &source_file(consumer),
                "consumer.o",
                optimization,
                None,
            );
            compile(right, &root, "driver.c", "driver.o", optimization, None);
            compile(
                right,
                &root,
                &source_file(backward),
                "back.o",
                optimization,
                None,
            );
            link_and_check(right, &root, "a.o", "consumer.o", "back.o", 0);
            for (filename, expected) in [("tag.c", 11), ("payload.c", 12)] {
                compile(left, &root, filename, "fault.o", optimization, Some(a));
                link_and_check(right, &root, "fault.o", "consumer.o", "back.o", expected);
            }
            for filename in ["duplicate.c", "reverse.c"] {
                compile(right, &root, filename, "fault.o", optimization, None);
                link_and_check(right, &root, "a.o", "fault.o", "back.o", 13);
            }
            for filename in ["back_duplicate.c", "back_reverse.c"] {
                compile(right, &root, filename, "fault.o", optimization, None);
                link_and_check(right, &root, "a.o", "consumer.o", "fault.o", 13);
            }
            rounds += 1;
        }
    }
    assert_eq!(rounds, 8);
    // Bazel-owned artifacts are exported to the host only after the gate passes.
    let exported = PathBuf::from(std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR").unwrap())
        .join("canonical-example");
    fs::create_dir_all(&exported).unwrap();
    for api in [owner, a, b, consumer, backward] {
        write(&exported, api);
    }
}
