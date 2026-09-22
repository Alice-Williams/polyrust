"""Canonical compiler input and typed addition-value AST observations."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":defs.bzl", "compiler_adapter")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def addition_ast_targets(name, c_sources):
    """Add read-only probes and production-byte comparison.

    Args:
        name: Executable observation test.
        c_sources: Complete C adapter source inventory.
    """
    for language in ["c", "java"]:
        root = "src/main.rs" if language == "c" else "src/java_main.rs"
        sources = c_sources if language == "c" else [root, "src/inputs.rs", "src/source_admission.rs"] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ])
        compiler_adapter(
            name = "addition_" + language + "_ast_probe",
            crate_root = root,
            rustc_cfg = "addition_ast_probe",
            directory_publisher = ":directory_publisher" if language == "c" else None,
            srcs = sources + ["test/addition_" + language + "_ast.rs", "test/addition_input_probe.rs", "test/floating_source_ast.rs", "test/addition_" + language + "_dataflow.rs", "test/addition_source_ast.rs"],
            deps = [
                ":compiler_configuration",
                "//crates/backend-" + language + ":portable_backend_" + language,
                "//crates/binary64:portable_binary64",
                "//crates/codegen:portable_codegen",
                "//crates/diagnostics:portable_diagnostics",
            ] + ([":directory_publication"] if language == "c" else []),
        )
    rust_source_metadata(
        name = "addition_composition_metadata",
        source = "test/addition_ast_fixture.rs",
        crate_name = "addition_composition",
        crate_key = "proof.addition_composition.v1",
    )
    rust_c_bundle(name = "addition_composition_c", crate = ":addition_composition_metadata")
    rust_java_bundle(name = "addition_composition_java", crate = ":addition_composition_metadata")
    inputs = [":addition_c_ast_probe", ":adapter", ":addition_java_ast_probe", ":java_adapter", "test/addition_ast_fixture.rs", "//tools/c:zig_native_oracle", ":addition_composition_c", ":addition_composition_java"]
    sh_test(
        name = name,
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/addition_ast.py)"] + ["$(rootpath " + item + ")" for item in inputs],
        data = ["test/addition_ast.py", "test/addition_composition.py", "test/addition_source_consumers.py", "test/wrapping_add_oracle.py", "test/java_fixture_native.py", "test/constant_export_scratch.py", "@bazel_tools//tools/jdk:current_java_runtime"] + inputs,
    )
