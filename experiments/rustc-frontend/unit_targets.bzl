"""Typed unit source proof libraries and independently generated crate bundles."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def unit_targets(name):
    """Declare bounded unit-result fixtures.

    Args:
        name: Fixture lint target.
    """
    models = []
    for owner, dependencies in [
        ("unit_leaf", []),
        ("unit_relay", ["unit_leaf"]),
        ("unit_root", ["unit_leaf", "unit_relay"]),
    ]:
        source = "fixtures/" + owner + ".rs"
        model = owner + "_model"
        models.append(":" + model)
        rust_library(
            name = model,
            srcs = [source],
            crate_name = owner,
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
        name = "rust_unit",
        srcs = ["fixtures/reference_unit.rs"],
        crate_root = "fixtures/reference_unit.rs",
        edition = "2024",
        deps = [":unit_root_model"],
    )
    rust_clippy_test(name = name, targets = models + [":rust_unit"])
    rust_c_bundle(name = "generated_c_unit", crate = ":unit_root_metadata")
    rust_java_bundle(name = "generated_java_unit", crate = ":unit_root_metadata")

    artifacts = [":generated_java_unit", ":generated_c_unit", ":rust_unit", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "unit_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/unit_native.py)"] + ["$(rootpath " + artifact + ")" for artifact in artifacts],
        data = [
            "test/unit_native.py",
            "test/unit_inventory.py",
            "test/unit_trace.py",
            "test/unit_consumers.py",
            "test/unit_examples.py",
            "fixtures/unit_leaf.rs",
            "fixtures/unit_relay.rs",
            "fixtures/unit_root.rs",
            "fixtures/reference_unit.rs",
            "test/constant_export_scratch.py",
            "test/java_fixture_native.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )

    sh_test(
        name = "unit_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/unit_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/unit_rejections.py", ":adapter", ":java_adapter"],
    )
