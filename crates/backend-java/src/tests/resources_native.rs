//! Independent Java 21 format-capacity controls, not production compilation.

use super::totality_oracle::CompiledPackage;
use portable_build::{portable_name, typed_program};

fn parameters(ty: &str, count: usize) -> String {
    (0..count)
        .map(|index| format!("{ty} p{index}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn java21_parameter_name_and_literal_capacity_controls() {
    let program = typed_program(portable_name!("resource_oracle"), |builder| builder);
    let manifest = crate::JavaBackend
        .generate_typed(&program)
        .expect("small package");
    let package = CompiledPackage::new(&manifest, "resource-capacity");
    let positive = format!(
        "package org.polyrust.consumer; public final class Consumer {{
          private Consumer() {{}}
          public static void main(String[] args) {{}}
          public static void staticLimit({}) {{}}
          public void instanceLimit({}) {{}}
          public static void wideLimit({}) {{}}
          public static void boxedLimit({}) {{}}
          public static void {}() {{}}
          public record RecordLimit({}) {{}}
          public static int label(String value) {{ return switch(value) {{ case \"{}\" -> 1; default -> 0; }}; }}
        }}",
        parameters("int", 255), parameters("int", 254), parameters("long", 127),
        parameters("Long", 255), "n".repeat(65_535), parameters("long", 127), "a".repeat(65_534),
    );
    package.consumer(&positive);
    for member in [
        format!(
            "public static void tooMany({}) {{}}",
            parameters("int", 256)
        ),
        format!("public void tooMany({}) {{}}", parameters("int", 255)),
        format!(
            "public static void tooMany({}) {{}}",
            parameters("long", 128)
        ),
        format!("public record TooMany({}) {{}}", parameters("long", 128)),
    ] {
        package.rejects_generated_member(&member, "too many parameters");
    }
    package.rejects_generated_member(
        &format!("public static void {}() {{}}", "n".repeat(65_536)),
        "too long for the constant pool",
    );
    package.rejects_generated_member(&format!("public static int label(String value) {{ return switch(value) {{ case \"{}\" -> 1; default -> 0; }}; }}", "a".repeat(65_535)), "constant string too long");
    package.rejects_generated_member("public static int label(int value) { return switch(value) { case 4294967295 -> 1; default -> 0; }; }", "integer number too large");
}
