"""Compiler-owned public constants: typed probes and independent native consumers."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter", "compiler_adapter_compile_fail_test")

def public_constant_targets(name, c_sources):
    """Declare source mappings, native proof, and negative builder contracts.

    Args:
        name: Native equivalence target.
        c_sources: Complete C adapter source inventory.
    """
    for language in ["c", "java"]:
        root = "src/main.rs" if language == "c" else "src/java_main.rs"
        sources = c_sources if language == "c" else [root, "src/inputs.rs", "src/source_admission.rs"] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ])
        deps = [
            ":compiler_configuration",
            "//crates/backend-" + language + ":portable_backend_" + language,
            "//crates/codegen:portable_codegen",
            "//crates/diagnostics:portable_diagnostics",
        ] + ([":directory_publication"] if language == "c" else [])
        compiler_adapter(
            name = "public_constant_" + language + "_probe",
            crate_root = root,
            rustc_cfg = "public_constant_ast_probe",
            srcs = sources + ["test/public_constant_" + language + "_ast.rs"] + (["test/public_constant_java_probe.rs"] if language == "java" else ["test/public_constant_manifest.rs"]),
            directory_publisher = ":directory_publisher" if language == "c" else None,
            deps = deps,
        )
        for slot in ["declaration", "read"]:
            for case, error in [
                ("missing", "error[E0599]"),
                ("duplicate", "error[E0599]"),
                ("wrong_capability", "error[E0271]"),
                ("wrong_context", "error[E0271]"),
                ("wrong_output", "error[E0271]"),
                ("wrong_input", "error[E0308]"),
                ("private_input", "error[E0451]"),
            ]:
                compiler_adapter_compile_fail_test(
                    name = "public_constant_" + language + "_" + slot + "_" + case + "_test",
                    crate_root = root,
                    srcs = sources + ["test/public_constant_contract.rs"],
                    rustc_cfgs = ["public_constant_contract", "public_constant_" + language, "public_constant_" + slot, "public_constant_" + case],
                    expected_error = error,
                    deps = deps,
                )
    rust_library(
        name = "public_constant_values_model",
        crate_name = "public_constant_values_model",
        srcs = ["fixtures/public_constant_values.rs", "fixtures/public_constant_data.rs"],
        crate_root = "fixtures/public_constant_values.rs",
        edition = "2024",
    )
    rust_binary(
        name = "rust_public_constant_values",
        srcs = ["fixtures/reference_public_constant.rs"],
        edition = "2024",
        deps = [":public_constant_values_model"],
    )
    rust_library(
        name = "public_constant_entry_model",
        crate_name = "public_constant_entry_model",
        srcs = ["fixtures/public_constant_entry.rs"],
        edition = "2024",
    )
    rust_binary(
        name = "rust_public_constant_entry",
        srcs = ["fixtures/reference_public_constant_entry.rs"],
        edition = "2024",
        deps = [":public_constant_entry_model"],
    )
    rust_clippy_test(
        name = "public_constant_fixture_clippy_test",
        targets = [":public_constant_values_model", ":rust_public_constant_values", ":public_constant_entry_model", ":rust_public_constant_entry"],
    )
    artifacts = [
        ":adapter",
        ":public_constant_c_probe",
        ":java_adapter",
        ":public_constant_java_probe",
        "fixtures/public_constant_data.rs",
        "fixtures/public_constant_values.rs",
        ":rust_public_constant_values",
        "//tools/c:zig_native_oracle",
        "fixtures/public_constant_entry.rs",
        ":rust_public_constant_entry",
    ]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/public_constant_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = ["test/public_constant_native.py", "test/public_constant_entry.py", "@bazel_tools//tools/jdk:current_java_runtime"] + artifacts,
    )
