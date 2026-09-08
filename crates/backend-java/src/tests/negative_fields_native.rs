//! Assignment conversions must not be mistaken for compiler-negative proof.

#[test]
fn compile_fail_fields_native_controls() {
    use portable_build::{portable_name, typed_program};
    let program = typed_program(portable_name!("negative_fields"), |builder| builder);
    let manifest = crate::JavaBackend
        .generate_typed(&program)
        .expect("small package");
    let package = super::totality_oracle::CompiledPackage::new(&manifest, "negative-fields");
    package.consumer("package org.polyrust.consumer; public final class Consumer { public static void main(String[] args) { long widened = 1; byte narrowed = 1; Object reference = \"s\"; if (widened != 1 || narrowed != 1 || !reference.equals(\"s\")) throw new AssertionError(); } }");
    package.rejects_generated_member("final int invalid = \"missing\";", "incompatible types");
}
