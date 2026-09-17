//! Exact finite bits, separate translation units and compiling corruption controls.
use super::*;
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
#[cfg(test)]
fn prepare(root: &Path, expected: &[u64], actual: &[u64]) {
    let first = api(&fixture(actual));
    let imports: Vec<_> = first.functions().cloned().collect();
    let second = api(&f::fixture(
        92,
        &vec![CScalarType::F64; imports.len()],
        &imports,
        &(0..imports.len()).map(Some).collect::<Vec<_>>(),
    ));
    let record = records::record();
    let record_function = crate::dialect::c_defined_functions(&record)
        .find(|function| function.linkage() == CLinkage::External)
        .unwrap();
    let record_name = record_function.name().as_str();
    let output = render_certified_package(&CStructuralRenderer, &record).unwrap();
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        fs::write(root.join(file.path()), text).unwrap();
    }
    let comparison = comparisons::comparisons();
    for owner in [&first, &second, &comparison] {
        let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            fs::write(root.join(file.path()), text).unwrap();
        }
    }
    let mut source = String::from(
        r#"#include "polyrust_dep_91.h"
#include "polyrust_dep_92.h"
#include "polyrust_dep_93.h"
#include "polyrust_crate_api.h"
#include <stdint.h>
#include <string.h>
#include <math.h>
#include <fenv.h>
static uint64_t bits(double value) { uint64_t result; memcpy(&result, &value, sizeof result); return result; }
int main(void) {
    if (fegetround() != FE_TONEAREST) return 2;
"#,
    );
    for owner in [&first, &second] {
        for (value, function) in expected.iter().zip(owner.functions()) {
            source.push_str(&format!(
                "if (bits({}(0.0)) != UINT64_C({value})) return 1;\n",
                function.symbol().as_str()
            ));
        }
        let identity = owner.functions().last().unwrap().symbol().as_str();
        for value in expected {
            source.push_str(&format!("{{ uint64_t raw = UINT64_C({value}); double value; memcpy(&value, &raw, sizeof value); if (bits({identity}(value)) != raw) return 1; }}\n"));
        }
        source.push_str(&format!("if (!isnan({identity}(NAN)) || {identity}(INFINITY) != INFINITY || {identity}(-INFINITY) != -INFINITY) return 1;\n"));
    }
    for (raw, expectations) in comparisons::cases() {
        source.push_str(&format!(
            "{{ uint64_t raw = UINT64_C({raw}); double value; memcpy(&value, &raw, sizeof value);\n"
        ));
        for (expected, function) in expectations.into_iter().zip(comparison.functions()) {
            source.push_str(&format!(
                "if ({}(value) != {}) return 3;\n",
                function.symbol().as_str(),
                u8::from(expected)
            ));
        }
        source.push_str("}\n");
    }
    for value in expected {
        source.push_str(&format!("{{ uint64_t raw = UINT64_C({value}); double value; memcpy(&value, &raw, sizeof value); if (bits({record_name}(value)) != raw) return 4; }}\n"));
    }
    // Volatile input makes a flush-to-zero execution mode observable.
    source.push_str("volatile double tiny = 0x1p-1074; if (!(tiny > 0.0) || bits(tiny) != 1) return 2; return 0; }\n");
    fs::write(root.join("consumer.c"), source).unwrap();
}
#[test]
fn finite_bits_native_and_sign_exponent_subnormal_mutants() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-binary64");
    fs::create_dir_all(&root).unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for mutation in 0..4 {
        let expected = if mutation == 0 {
            values()
        } else {
            values()[..8].to_vec()
        };
        let mut actual = expected.clone();
        match mutation {
            1 => actual[0] ^= 1 << 63,
            2 => actual[3] ^= 1 << 52,
            3 => actual[1] = 2,
            _ => {}
        }
        let directory = root.join(format!("case-{mutation}"));
        fs::create_dir_all(&directory).unwrap();
        prepare(&directory, &expected, &actual);
        for compiler in [PathBuf::from("gcc-14"), zig.clone()] {
            for optimization in ["-O0", "-O2"] {
                let mut objects = vec![];
                for name in [
                    "polyrust_dep_91",
                    "polyrust_dep_92",
                    "polyrust_dep_93",
                    "crate_api",
                    "consumer",
                ] {
                    let object = directory.join(format!("{name}.o"));
                    run(Command::new(&compiler)
                        .args([
                            "-std=c17",
                            "-Wall",
                            "-Wextra",
                            "-Wpedantic",
                            "-Werror",
                            "-Wstrict-prototypes",
                            "-Wmissing-prototypes",
                            "-fno-fast-math",
                            "-ffp-contract=off",
                            optimization,
                            "-c",
                        ])
                        .arg(directory.join(format!("{name}.c")))
                        .arg("-o")
                        .arg(&object));
                    objects.push(object);
                }
                let executable = directory.join("consumer");
                run(Command::new(&compiler)
                    .args(objects)
                    .arg("-lm")
                    .arg("-o")
                    .arg(&executable));
                let output = Command::new(executable).output().unwrap();
                assert_eq!(
                    output.status.code(),
                    Some(i32::from(mutation != 0)),
                    "mutation {mutation}, {compiler:?}, {optimization}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(output.stderr.is_empty());
            }
        }
    }
}
