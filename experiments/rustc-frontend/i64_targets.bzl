"""Exact wide scalar values across real Rust crate and target package boundaries."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":defs.bzl", "compiler_adapter_compile_fail_test")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def i64_targets(name, c_sources):
    """Create source fixtures, real bundles and independent boundary proofs.

    Args:
        name: Fixture Clippy target.
        c_sources: Complete C compiler adapter inventory for input privacy checks.
    """
    for owner, dependencies in [("i64_leaf", []), ("i64_values", ["i64_leaf"])]:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner if owner == "i64_leaf" else "i64_values_model",
            crate_root = source,
            edition = "2024",
            deps = [":" + dependency + "_model" for dependency in dependencies],
        )
        rust_source_metadata(
            name = owner + "_metadata",
            source = source,
            crate_name = owner,
            crate_key = "proof." + owner + ".v1",
            dependencies = {dependency: ":" + dependency + "_metadata" for dependency in dependencies},
        )
    rust_binary(
        name = "rust_i64_values",
        srcs = ["fixtures/reference_i64.rs"],
        crate_root = "fixtures/reference_i64.rs",
        edition = "2024",
        deps = [":i64_values_model"],
    )
    rust_clippy_test(name = name, targets = [":i64_leaf_model", ":i64_values_model", ":rust_i64_values"])
    rust_c_bundle(name = "generated_c_i64", crate = ":i64_values_metadata")
    rust_java_bundle(name = "generated_java_i64", crate = ":i64_values_metadata")
    artifacts = [":generated_java_i64", ":generated_c_i64", ":rust_i64_values", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "i64_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/i64_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/i64_native.py",
            "test/i64_inventory.py",
            "test/i64_oracle.py",
            "test/i64_order_mutations.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "i64_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/i64_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/i64_rejections.py", ":adapter", ":java_adapter", "@bazel_tools//tools/jdk:current_java_runtime"],
    )
    for language in ["c", "java"]:
        root = "src/main.rs" if language == "c" else "src/java_main.rs"
        sources = c_sources if language == "c" else [root, "src/inputs.rs", "src/source_admission.rs"] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ])
        compiler_adapter_compile_fail_test(
            name = "literal_" + language + "_private_input_test",
            crate_root = root,
            srcs = sources + ["test/literal_private_input.rs"],
            rustc_cfg = "literal_private_input",
            expected_error = "error[E0451]",
            deps = [
                ":compiler_configuration",
                "//crates/backend-" + language + ":portable_backend_" + language,
                "//crates/binary64:portable_binary64",
                "//crates/codegen:portable_codegen",
                "//crates/diagnostics:portable_diagnostics",
            ] + ([":directory_publication"] if language == "c" else []),
        )
