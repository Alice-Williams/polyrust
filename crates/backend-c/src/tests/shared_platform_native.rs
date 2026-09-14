//! Paired native controls for platform rejection and diagnostic token safety.
use crate::ast::*;
use std::{fs, path::PathBuf, process::Command};

pub(super) fn diagnostic_fixture(bytes: Vec<u8>) -> (CFrozenRegistry, CSourceFile) {
    let (registry, source) = super::platform_tests::installed();
    let declarations =
        CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
    let mut items = source.items().to_vec();
    let CFileItem::StaticAssert(assertion) = &items[0] else {
        panic!("required assertion");
    };
    items[0] = CFileItem::StaticAssert(
        declarations
            .static_assert(assertion.condition().clone(), CAssertDiagnostic::new(bytes))
            .unwrap(),
    );
    let source = declarations.source_file(items).unwrap();
    (registry, source)
}

#[test]
fn certified_assertions_compile_hostile_bytes_and_reject_wrong_layout() {
    let directory =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-platform-assertions");
    fs::create_dir_all(&directory).unwrap();
    let ordinary = super::spelling_tests::linked_source(super::platform_tests::installed());
    // Every byte, plus preprocessing hazards and escape-following digits.
    let mut bytes: Vec<_> = (0..=255).collect();
    bytes.extend_from_slice(b"\"; bad_token; /* */ ??/\n\\\n0123456789");
    let hostile = super::spelling_tests::linked_source(diagnostic_fixture(bytes));
    assert!(hostile.contains("[0x00][0x01]"));
    assert!(hostile.contains("[0xFF]"));
    assert!(!hostile.contains("??/"));
    let boundary = super::spelling_tests::linked_source(diagnostic_fixture(vec![b'a'; 4095]));
    let expanded = super::spelling_tests::linked_source(diagnostic_fixture(vec![0; 682]));
    let correct = "sizeof(_Bool) == 1UL";
    assert_eq!(ordinary.matches(correct).count(), 1);
    // Deliberate post-generation mutation, never admitted as certified AST.
    let wrong = ordinary.replacen(correct, "sizeof(_Bool) == 2UL", 1);
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for (compiler_id, compiler) in [PathBuf::from("gcc-14"), zig].iter().enumerate() {
        for optimization in ["-O0", "-O2"] {
            for (case, source, accepted) in [
                ("ordinary", &ordinary, true),
                ("hostile", &hostile, true),
                ("boundary", &boundary, true),
                ("expanded-boundary", &expanded, true),
                ("wrong-layout", &wrong, false),
            ] {
                let input = directory.join(format!("{compiler_id}{optimization}-{case}.c"));
                fs::write(&input, source).unwrap();
                let output = Command::new(compiler)
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
                    .arg(&input)
                    .arg("-o")
                    .arg(input.with_extension("o"))
                    .output()
                    .unwrap();
                let diagnostic = String::from_utf8_lossy(&output.stderr);
                assert_eq!(output.status.success(), accepted, "{case}: {diagnostic}");
                if !accepted {
                    assert!(diagnostic.contains("static assertion"), "{diagnostic}");
                    assert!(diagnostic.contains("C profile Bool Size"), "{diagnostic}");
                    assert!(!input.with_extension("o").exists());
                }
            }
        }
    }
}
