"""Independent executable registration and native Boolean-negation proofs."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":defs.bzl", "compiler_adapter_compile_fail_test")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def boolean_negation_native_targets(name):
    """Compile real source and production bundles, not hand-built AST fixtures.

    Args:
        name: Aggregate native, rejection and fixture-lint suite.
    """
    sh_test(
        name = "boolean_negation_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/boolean_negation_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/boolean_negation_rejections.py", ":adapter", ":java_adapter"],
    )
    rust_library(
        name = "boolean_negation_model",
        srcs = ["fixtures/boolean_negation.rs"],
        crate_root = "fixtures/boolean_negation.rs",
        edition = "2024",
    )
    rust_binary(
        name = "rust_boolean_negation",
        srcs = ["fixtures/reference_boolean_negation.rs"],
        crate_root = "fixtures/reference_boolean_negation.rs",
        deps = [":boolean_negation_model"],
        edition = "2024",
    )
    rust_clippy_test(
        name = "boolean_negation_clippy_test",
        targets = [":boolean_negation_model", ":rust_boolean_negation"],
    )
    rust_source_metadata(
        name = "boolean_negation_metadata",
        source = "fixtures/boolean_negation.rs",
        crate_name = "boolean_negation",
        crate_key = "boolean.negation.v1",
    )
    rust_java_bundle(name = "generated_java_boolean_negation", crate = ":boolean_negation_metadata")
    rust_c_bundle(name = "generated_c_boolean_negation", crate = ":boolean_negation_metadata")
    artifacts = [
        ":generated_java_boolean_negation",
        ":generated_c_boolean_negation",
        ":rust_boolean_negation",
        "fixtures/inputs.txt",
        "//tools/c:zig_native_oracle",
    ]
    sh_test(
        name = "boolean_negation_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/java_fixture_native.py)", "boolean_negation"] + ["$(rootpath " + artifact + ")" for artifact in artifacts],
        data = ["test/java_fixture_native.py", "@bazel_tools//tools/jdk:current_java_runtime"] + artifacts,
    )
    native.test_suite(name = name, tests = [
        ":boolean_negation_native_test",
        ":boolean_negation_rejection_test",
        ":boolean_negation_clippy_test",
    ])

def boolean_negation_contract_targets(name, c_sources):
    """Check the new slot without weakening the existing ten-slot controls.

    Args:
        name: Aggregate compile-contract suite.
        c_sources: Complete source inventory for the C compiler adapter.
    """
    tests = []
    for language in ["c", "java"]:
        root = "src/main.rs" if language == "c" else "src/java_main.rs"
        sources = c_sources if language == "c" else [
            root,
            "src/inputs.rs",
            "src/source_admission.rs",
        ] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ])
        for case, error in [
            ("missing", "error[E0599]"),
            ("duplicate", "error[E0599]"),
            ("wrong_capability", "error[E0271]"),
            ("wrong_context", "error[E0271]"),
            ("wrong_output", "error[E0271]"),
            ("wrong_input", "error[E0308]"),
            ("private_input", "error[E0451]"),
        ]:
            target = "boolean_" + language + "_" + case + "_test"
            tests.append(":" + target)
            compiler_adapter_compile_fail_test(
                name = target,
                crate_root = root,
                srcs = sources + ["test/boolean_negation_contract.rs"],
                rustc_cfgs = ["boolean_contract", "boolean_" + language, "boolean_" + case],
                expected_error = error,
                deps = [
                    ":compiler_configuration",
                    "//crates/backend-" + language + ":portable_backend_" + language,
                    "//crates/binary64:portable_binary64",
                    "//crates/codegen:portable_codegen",
                    "//crates/diagnostics:portable_diagnostics",
                ] + ([":directory_publication"] if language == "c" else []),
            )
    native.test_suite(name = name, tests = tests)
