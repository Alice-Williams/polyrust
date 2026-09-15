"""Typed lazy Boolean source and real target bundles."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def short_circuit_targets(name):
    """Create independently cached source and target artifacts.

    Args:
        name: Fixture lint target.
    """
    rust_library(
        name = "short_circuit_model",
        srcs = ["fixtures/short_circuit.rs"],
        crate_root = "fixtures/short_circuit.rs",
        edition = "2024",
    )
    rust_binary(
        name = "rust_short_circuit",
        srcs = ["fixtures/reference_short_circuit.rs"],
        crate_root = "fixtures/reference_short_circuit.rs",
        deps = [":short_circuit_model"],
        edition = "2024",
    )
    rust_clippy_test(name = name, targets = [":short_circuit_model", ":rust_short_circuit"])
    rust_source_metadata(
        name = "short_circuit_metadata",
        source = "fixtures/short_circuit.rs",
        crate_name = "short_circuit",
        crate_key = "boolean.short.circuit.v1",
    )
    rust_java_bundle(name = "generated_java_short_circuit", crate = ":short_circuit_metadata")
    rust_c_bundle(name = "generated_c_short_circuit", crate = ":short_circuit_metadata")
    artifacts = [
        ":generated_java_short_circuit",
        ":generated_c_short_circuit",
        ":rust_short_circuit",
        "//tools/c:zig_native_oracle",
    ]
    sh_test(
        name = "short_circuit_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/short_circuit_native.py)"] + ["$(rootpath " + artifact + ")" for artifact in artifacts],
        data = [
            "test/short_circuit_native.py",
            "test/short_circuit_oracle.py",
            "test/short_circuit_mutations.py",
            "test/java_fixture_native.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "short_circuit_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/short_circuit_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/short_circuit_rejections.py", ":adapter", ":java_adapter"],
    )
