"""Original character crates and native Rust truth, before target publication."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def character_source_targets(name):
    """Declare original Rust profiles and source-owned target artifacts.

    Args:
        name: Original-source differential proof against independent scalar truth.
    """
    owners = [("character_leaf", []), ("character_middle", ["character_leaf"]), ("character_root", ["character_middle"])]
    for owner, dependencies in owners:
        rust_source_metadata(
            name = owner + "_metadata",
            source = "fixtures/" + owner + ".rs",
            crate_name = owner,
            crate_key = "proof." + owner + ".v1",
            dependencies = {dependency: ":" + dependency + "_metadata" for dependency in dependencies},
        )
    lint = []
    references = []
    for suffix, opt, checks in [("checked", "0", "yes"), ("optimized", "2", "no")]:
        flags = ["-Copt-level=" + opt, "-Coverflow-checks=" + checks]
        for owner, dependencies in owners:
            source = "fixtures/" + owner + ".rs"
            target = owner + "_" + suffix
            lint.append(":" + target)
            rust_library(
                name = target,
                crate_name = owner,
                crate_root = source,
                srcs = [source],
                edition = "2024",
                rustc_flags = flags,
                deps = [":" + dependency + "_" + suffix for dependency in dependencies],
            )
        reference = "character_source_reference_" + suffix
        references.append(":" + reference)
        lint.append(":" + reference)
        rust_binary(
            name = reference,
            testonly = True,
            srcs = ["fixtures/reference_character_source.rs"],
            edition = "2024",
            rustc_flags = flags,
            deps = [":character_root_" + suffix, ":character_leaf_" + suffix],
        )
    rust_clippy_test(name = "character_source_clippy_test", targets = lint)
    rustfmt_test(name = "character_source_rustfmt_test", targets = lint)
    native.genrule(
        name = "character_traced_leaf_source",
        testonly = True,
        srcs = ["fixtures/character_leaf.rs", "test/character_source_trace.py"],
        outs = ["character_traced_leaf.rs"],
        cmd = "python3 $(location test/character_source_trace.py) $(location fixtures/character_leaf.rs) $@",
    )
    for owner, dependencies in owners:
        source = ":character_traced_leaf_source" if owner == "character_leaf" else "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_traced",
            testonly = True,
            crate_name = owner,
            crate_root = source,
            srcs = [source],
            edition = "2024",
            deps = [":" + dependency + "_traced" for dependency in dependencies],
        )
    rust_binary(
        name = "character_source_reference_traced",
        testonly = True,
        srcs = ["fixtures/reference_character_source.rs"],
        edition = "2024",
        deps = [":character_root_traced", ":character_leaf_traced"],
    )
    references.append(":character_source_reference_traced")
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/character_source_rust.py)"] + ["$(rootpath " + target + ")" for target in references],
        data = references + [
            "test/character_source_rust.py",
            "test/character_source_truth.py",
            ":character_oracle_support",
        ],
    )
    for owner in ["middle", "root"]:
        rust_c_bundle(name = "generated_c_character_" + owner, crate = ":character_" + owner + "_metadata")
        rust_java_bundle(name = "generated_java_character_" + owner, crate = ":character_" + owner + "_metadata")
    artifacts = [":generated_c_character_root", ":generated_java_character_root"] + references[:2] + ["//tools/c:zig_native_oracle"]
    sh_test(
        name = "character_source_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/character_source_native.py)"] + ["$(rootpath " + target + ")" for target in artifacts],
        data = artifacts + [
            "test/character_source_native.py",
            "test/character_source_privacy.py",
            "test/character_source_examples.py",
            "fixtures/character_leaf.rs",
            "fixtures/character_middle.rs",
            "fixtures/character_root.rs",
            "fixtures/reference_character_source.rs",
            "test/character_source_clients.py",
            "test/character_source_inventory.py",
            "test/character_source_truth.py",
            "test/constant_export_native.py",
            "test/constant_export_scratch.py",
            "test/finite_constant_source_consumers.py",
            ":character_oracle_support",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ],
    )
    observations = [":generated_c_character_root", ":generated_java_character_root", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "character_source_observations_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/character_source_observations.py)"] + ["$(rootpath " + target + ")" for target in observations],
        data = observations + [
            "test/character_source_observations.py",
            "test/character_source_faults.py",
            "test/character_source_native.py",
            "test/character_source_privacy.py",
            "test/character_source_examples.py",
            "test/character_source_clients.py",
            "test/character_source_inventory.py",
            "test/character_source_truth.py",
            "test/constant_export_native.py",
            "test/constant_export_scratch.py",
            "test/finite_constant_source_consumers.py",
            ":character_oracle_support",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ],
    )
