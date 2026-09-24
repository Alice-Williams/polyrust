//! Standalone generated header/source smoke proof on both pinned C compilers.
use super::fixture::*;
use portable_backend_c::dialect::CStructuralRenderer;
use portable_codegen::{OutputContents, render_certified_package};
use std::{fs, path::PathBuf, process::Command};

fn run(command: &mut Command) {
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn standalone_owner_compiles_without_runtime_or_source_crate() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("canonical-owner");
    fs::create_dir_all(&root).unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for maximum in [false, true] {
        let mut value = fixture(maximum, Fault::None);
        register(&mut value, Fault::None).unwrap();
        let base = PROFILE.basename(value.facts.instance().key());
        let output =
            render_certified_package(&CStructuralRenderer, &certify(value).unwrap()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text");
            };
            fs::write(root.join(file.path()), text).unwrap();
        }
        // Test-only foreign consumer. Generated implementation is just its typed include.
        let driver = root.join("driver.c");
        fs::write(
            &driver,
            format!(
                "#include \"{base}.h\"\n#include \"{base}.h\"\nint main(void) {{ return 0; }}\n"
            ),
        )
        .unwrap();
        for (index, compiler) in [PathBuf::from("gcc-14"), zig.clone()].iter().enumerate() {
            for optimization in ["-O0", "-O2"] {
                let object = root.join(format!("{maximum}-{index}{optimization}.o"));
                let flags = [
                    "-std=c17",
                    "-Wall",
                    "-Wextra",
                    "-Wpedantic",
                    "-Werror",
                    "-Wstrict-prototypes",
                    "-Wmissing-prototypes",
                    optimization,
                ];
                run(Command::new(compiler)
                    .args(flags)
                    .arg("-c")
                    .arg(root.join(format!("{base}.c")))
                    .arg("-o")
                    .arg(&object));
                let binary = root.join(format!("{maximum}-{index}{optimization}"));
                run(Command::new(compiler)
                    .args(flags)
                    .arg(&driver)
                    .arg(&object)
                    .arg("-o")
                    .arg(&binary));
                run(&mut Command::new(&binary));
                let symbols = Command::new("nm")
                    .args(["--defined-only", "--extern-only"])
                    .arg(&object)
                    .output()
                    .unwrap();
                assert!(symbols.status.success());
                assert!(
                    symbols.stdout.is_empty(),
                    "type-only owner exported an executable symbol"
                );
            }
        }
    }
}
