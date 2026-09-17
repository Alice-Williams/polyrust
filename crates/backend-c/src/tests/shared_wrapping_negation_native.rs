//! Independently compiled C owners, modular truth and sanitizer evidence.
use super::*;
use crate::dialect::CStructuralRenderer;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn cases() -> String {
    let mut values = vec![
        i64::MIN,
        i64::MIN + 1,
        i64::MAX,
        -1,
        0,
        1,
        i64::from(i32::MIN),
        i64::from(i32::MAX),
    ];
    for bit in 0..63 {
        let n = 1_i64 << bit;
        values.extend([n - 1, n, n + 1, -n - 1, -n, -n + 1]);
    }
    let mut seed = 0x83e51a93f6294db7_u64;
    for _ in 0..4096 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        values.push(seed as i64);
    }
    values
        .into_iter()
        .map(|wide| {
            let small = wide as i32;
            let a = if small == i32::MIN {
                i64::from(small)
            } else {
                -i64::from(small)
            };
            let b = if wide == i64::MIN {
                i128::from(wide)
            } else {
                -i128::from(wide)
            };
            format!("{small} {a} {wide} {b}\n")
        })
        .collect()
}
fn run(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[cfg(test)]
fn prepare(root: &Path, mutant: bool) {
    let fixture = fixture(
        81,
        if mutant {
            Guard::WrongOperation
        } else {
            Guard::Valid
        },
    );
    let first = CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, f::linked(&fixture)).unwrap(),
    )
    .unwrap();
    let imports: Vec<_> = first.functions().cloned().collect();
    let fixture = f::fixture(
        82,
        &[CScalarType::I32, CScalarType::I64],
        &imports,
        &[Some(0), Some(1)],
    );
    let second = CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, f::linked(&fixture)).unwrap(),
    )
    .unwrap();
    for owner in [&first, &second] {
        let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        assert_eq!(output.files().len(), 2);
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("C text")
            };
            assert!(!text.contains("runtime.") && !text.contains("goto "));
            fs::write(root.join(file.path()), text).unwrap();
        }
    }
    let mut source = r#"#include "polyrust_dep_81.h"
#include "polyrust_dep_82.h"
#include <inttypes.h>
#include <stdio.h>
int main(void) {
    int32_t small, expected_small;
    int64_t wide, expected_wide;
    unsigned int rows = 0;
    while (scanf("%" SCNd32 " %" SCNd32 " %" SCNd64 " %" SCNd64, &small, &expected_small, &wide, &expected_wide) == 4) {
        if (PRODUCER_32(small) != expected_small || CONSUMER_32(small) != expected_small ||
            PRODUCER_64(wide) != expected_wide || CONSUMER_64(wide) != expected_wide) return 1;
        ++rows;
    }
    return rows == 4482 ? 0 : 2;
}
"#.to_owned();
    for (prefix, owner) in [("PRODUCER", &first), ("CONSUMER", &second)] {
        for (width, function) in [32, 64].into_iter().zip(owner.functions()) {
            source = source.replace(&format!("{prefix}_{width}"), function.symbol().as_str());
        }
    }
    fs::write(root.join("consumer.c"), source).unwrap();
    fs::write(root.join("cases.txt"), cases()).unwrap();
}
#[test]
fn wrapping_negation_native_boundaries_and_mutation_have_no_c_signed_overflow() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-wrapping-negation");
    fs::create_dir_all(&root).unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for mutant in [false, true] {
        let directory = root.join(if mutant { "wrong-operation" } else { "valid" });
        fs::create_dir(&directory).unwrap();
        prepare(&directory, mutant);
        for (index, compiler) in [PathBuf::from("gcc-14"), zig.clone()].iter().enumerate() {
            for optimization in ["-O0", "-O2"] {
                for sanitized in [false, true] {
                    if sanitized && (index != 0 || optimization != "-O2") {
                        continue;
                    }
                    let extra = if sanitized {
                        vec!["-fsanitize=undefined", "-fno-sanitize-recover=all"]
                    } else {
                        vec![]
                    };
                    let mut objects = vec![];
                    for name in ["polyrust_dep_81", "polyrust_dep_82", "consumer"] {
                        let object =
                            directory.join(format!("{name}-{index}-{optimization}-{sanitized}.o"));
                        run(Command::new(compiler)
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
                            .args(&extra)
                            .arg(directory.join(format!("{name}.c")))
                            .arg("-o")
                            .arg(&object));
                        objects.push(object);
                    }
                    for id in [81, 82] {
                        let header = directory.join(format!("header_{id}.c"));
                        fs::write(&header, format!("#include \"polyrust_dep_{id}.h\"\n")).unwrap();
                        run(Command::new(compiler)
                            .args(["-std=c17", "-Wall", "-Wextra", "-Werror", "-c"])
                            .arg(header)
                            .arg("-o")
                            .arg(directory.join(format!("header_{id}.o"))));
                    }
                    let binary =
                        directory.join(format!("consumer-{index}-{optimization}-{sanitized}"));
                    run(Command::new(compiler)
                        .args(&extra)
                        .args(objects)
                        .arg("-o")
                        .arg(&binary));
                    let output = Command::new(binary)
                        .stdin(fs::File::open(directory.join("cases.txt")).unwrap())
                        .output()
                        .unwrap();
                    assert_eq!(
                        output.status.code(),
                        Some(if mutant { 1 } else { 0 }),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    assert!(
                        output.stderr.is_empty(),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
        }
    }
}
