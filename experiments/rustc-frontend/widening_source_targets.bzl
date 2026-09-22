"""Original multi-crate Rust and normal runtime-free C/Java packages."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def widening_source_targets(name):
    """Declare source, native-reference and generated-package actions.

    Args:
        name: Clippy gate for original fixtures and native reference.
    """
    owners = [("widening_leaf", []), ("widening_middle", ["widening_leaf"]), ("widening_root", ["widening_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "widening_root" else owner,
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
        name = "rust_widening_source",
        srcs = ["fixtures/reference_widening_source.rs"],
        crate_root = "fixtures/reference_widening_source.rs",
        edition = "2024",
        deps = [":widening_root_model"],
    )
    rust_library(
        name = "widening_composition_model",
        srcs = ["fixtures/widening_composition.rs"],
        crate_root = "fixtures/widening_composition.rs",
        crate_name = "widening_composition",
        edition = "2024",
    )
    rust_binary(
        name = "rust_widening_composition",
        srcs = ["fixtures/reference_widening_composition.rs"],
        crate_root = "fixtures/reference_widening_composition.rs",
        edition = "2024",
        deps = [":widening_composition_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_widening_source", ":widening_composition_model", ":rust_widening_composition"])
    rust_source_metadata(
        name = "widening_composition_metadata",
        source = "fixtures/widening_composition.rs",
        crate_name = "widening_composition",
        crate_key = "proof.widening_composition.v1",
    )
    rust_c_bundle(name = "widening_composition_c", crate = ":widening_composition_metadata")
    rust_java_bundle(name = "widening_composition_java", crate = ":widening_composition_metadata")
    composition = [":widening_composition_java", ":widening_composition_c", ":rust_widening_composition", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "widening_composition_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/widening_composition.py)"] + ["$(rootpath " + item + ")" for item in composition],
        data = ["test/widening_composition.py", "test/widening_source_consumers.py", "test/widening_oracle.py", "test/java_fixture_native.py", "test/constant_export_scratch.py", "@bazel_tools//tools/jdk:current_java_runtime"] + composition,
    )
    native.genrule(
        name = "widening_traced_leaf_source",
        testonly = True,
        srcs = ["fixtures/widening_leaf.rs", "test/widening_rust_trace.py"],
        outs = ["widening_traced_leaf.rs"],
        cmd = "python3 $(location test/widening_rust_trace.py) $(location fixtures/widening_leaf.rs) $@",
    )
    for owner, dependencies in owners:
        source = ":widening_traced_leaf_source" if owner == "widening_leaf" else "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_traced",
            testonly = True,
            srcs = [source],
            crate_name = owner + "_model" if owner == "widening_root" else owner,
            crate_root = source,
            edition = "2024",
            deps = [":" + dependency + "_traced" for dependency in dependencies],
        )
    rust_binary(
        name = "rust_widening_traced",
        testonly = True,
        srcs = ["fixtures/reference_widening_source.rs"],
        crate_root = "fixtures/reference_widening_source.rs",
        edition = "2024",
        deps = [":widening_root_traced"],
    )
    rust_c_bundle(name = "generated_c_widening", crate = ":widening_root_metadata")
    rust_java_bundle(name = "generated_java_widening", crate = ":widening_root_metadata")
    artifacts = [":generated_java_widening", ":generated_c_widening", ":rust_widening_source", ":rust_widening_traced", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "widening_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/widening_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/widening_rejections.py", ":adapter", ":java_adapter"],
    )
    sh_test(
        name = "widening_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/widening_source_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/widening_source_native.py",
            "test/widening_source_inventory.py",
            "test/addition_source_inventory.py",
            "test/widening_source_consumers.py",
            "test/widening_source_mutations.py",
            "test/addition_source_privacy.py",
            "test/widening_oracle.py",
            "test/short_circuit_mutations.py",
            "test/java_fixture_native.py",
            "test/constant_export_scratch.py",
            "fixtures/widening_leaf.rs",
            "fixtures/widening_middle.rs",
            "fixtures/widening_root.rs",
            "fixtures/reference_widening_source.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
