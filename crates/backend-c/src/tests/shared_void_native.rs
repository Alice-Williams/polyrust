//! C17 void ABI, separately compiled owners, both compiler/optimization pairs.
use super::*;
use portable_codegen::*;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn void_owners_compile_and_link_without_unit_storage_or_runtime() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-void-results");
    fs::create_dir_all(&root).unwrap();
    let owners = void_fixture::chain();
    for owner in &owners {
        let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("C source");
            };
            fs::write(root.join(file.path()), text).unwrap();
        }
    }
    fs::write(
        root.join("consumer.c"),
        r#"
#include "polyrust_void_71.h"
#include "polyrust_void_72.h"
#include "polyrust_void_73.h"
int main(void) {
    void (*first)(int32_t) = poly_operation_71;
    void (*second)(int32_t) = poly_operation_72;
    const int32_t values[] = { INT32_MIN, -1, 0, 1, INT32_MAX };
    for (unsigned int index = 0; index < sizeof(values) / sizeof(values[0]); ++index) {
        first(values[index]);
        second(values[index]);
        if (poly_operation_73(values[index]) != values[index]) { return 1; }
    }
    return 0;
}
"#,
    )
    .unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for (compiler_index, compiler) in [PathBuf::from("gcc-14"), zig].iter().enumerate() {
        for optimization in ["-O0", "-O2"] {
            let mut objects = vec![];
            for name in ["void_71", "void_72", "void_73", "consumer"] {
                let object = root.join(format!("{compiler_index}{optimization}_{name}.o"));
                let result = Command::new(compiler)
                    .args([
                        "-std=c17",
                        "-Wall",
                        "-Wextra",
                        "-Wpedantic",
                        "-Werror",
                        "-Wstrict-prototypes",
                        "-Wmissing-prototypes",
                        optimization,
                        "-c",
                    ])
                    .arg(root.join(format!("{name}.c")))
                    .arg("-o")
                    .arg(&object)
                    .output()
                    .unwrap();
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
                objects.push(object);
            }
            for id in [71, 72, 73] {
                let probe = root.join(format!("header_{id}.c"));
                fs::write(&probe, format!("#include \"polyrust_void_{id}.h\"\n")).unwrap();
                let result = Command::new(compiler)
                    .args([
                        "-std=c17",
                        "-Wall",
                        "-Wextra",
                        "-Wpedantic",
                        "-Werror",
                        "-c",
                    ])
                    .arg(probe)
                    .arg("-o")
                    .arg(root.join(format!("header_{id}.o")))
                    .output()
                    .unwrap();
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            let binary = root.join(format!("probe_{compiler_index}{optimization}"));
            let result = Command::new(compiler)
                .args(&objects)
                .arg("-o")
                .arg(&binary)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(Command::new(&binary).status().unwrap().success());
            let invalid = root.join("invalid.c");
            fs::write(&invalid, "#include \"polyrust_void_71.h\"\nint main(void) { int32_t result = poly_operation_71(1); return result; }\n").unwrap();
            let result = Command::new(compiler)
                .args(["-std=c17", "-Werror", "-c"])
                .arg(invalid)
                .arg("-o")
                .arg(root.join("invalid.o"))
                .output()
                .unwrap();
            assert!(!result.status.success());
            assert!(String::from_utf8_lossy(&result.stderr).contains("void"));
        }
    }
}
