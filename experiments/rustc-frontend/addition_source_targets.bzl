"""Original multi-crate Rust and normal runtime-free C/Java packages."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def addition_source_targets(name):
    """Declare source, native-reference and generated-package actions.

    Args:
        name: Clippy gate for original fixtures and native reference.
    """
    owners = [("addition_leaf", []), ("addition_middle", ["addition_leaf"]), ("addition_root", ["addition_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "addition_root" else owner,
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
        name = "rust_addition_source",
        srcs = ["fixtures/reference_addition_source.rs"],
        crate_root = "fixtures/reference_addition_source.rs",
        edition = "2024",
        deps = [":addition_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_addition_source"])
    native.genrule(
        name = "addition_traced_leaf_source",
        testonly = True,
        srcs = ["fixtures/addition_leaf.rs", "test/addition_rust_trace.py"],
        outs = ["addition_traced_leaf.rs"],
        cmd = "python3 $(location test/addition_rust_trace.py) $(location fixtures/addition_leaf.rs) $@",
    )
    for owner, dependencies in owners:
        source = ":addition_traced_leaf_source" if owner == "addition_leaf" else "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_traced",
            testonly = True,
            srcs = [source],
            crate_name = owner + "_model" if owner == "addition_root" else owner,
            crate_root = source,
            edition = "2024",
            deps = [":" + dependency + "_traced" for dependency in dependencies],
        )
    rust_binary(
        name = "rust_addition_traced",
        testonly = True,
        srcs = ["fixtures/reference_addition_source.rs"],
        crate_root = "fixtures/reference_addition_source.rs",
        edition = "2024",
        deps = [":addition_root_traced"],
    )
    rust_c_bundle(name = "generated_c_addition", crate = ":addition_root_metadata")
    rust_java_bundle(name = "generated_java_addition", crate = ":addition_root_metadata")
    artifacts = [":generated_java_addition", ":generated_c_addition", ":rust_addition_source", ":rust_addition_traced", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "addition_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/addition_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/addition_rejections.py", ":adapter", ":java_adapter"],
    )
    sh_test(
        name = "addition_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/addition_source_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/addition_source_native.py",
            "test/addition_source_inventory.py",
            "test/addition_source_consumers.py",
            "test/addition_source_mutations.py",
            "test/addition_source_privacy.py",
            "test/addition_source_examples.py",
            "test/wrapping_add_oracle.py",
            "test/short_circuit_mutations.py",
            "test/java_fixture_native.py",
            "test/constant_export_scratch.py",
            "fixtures/addition_leaf.rs",
            "fixtures/addition_middle.rs",
            "fixtures/addition_root.rs",
            "fixtures/reference_addition_source.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
