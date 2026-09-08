//! Independent javac rejection of generic creation and misplaced dimensions.

#[test]
fn java21_array_creation_negative_controls() {
    use portable_build::{portable_name, typed_program};
    let program = typed_program(portable_name!("array_creation"), |builder| builder);
    let manifest = crate::JavaBackend
        .generate_typed(&program)
        .expect("small package");
    let package = super::totality_oracle::CompiledPackage::new(&manifest, "array-creation");
    package.rejects_generated_member("public static void invalid() { java.util.List<String>[] values = new java.util.List<String>[1]; }", "generic array creation");
    package.rejects_generated_member(
        "public static <T> void invalid() { T[] values = new T[1]; }",
        "generic array creation",
    );
    package.rejects_generated_member(
        "public static void invalid() { int[][] values = new int[][1]; }",
        "']' expected",
    );
}
