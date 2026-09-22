"""Original infinity crates, profile-specific Rust references and target bundles."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def infinite_constant_source_targets(name):
    """Declare independent original-source and generated native proofs.

    Args:
        name: Differential native test target.
    """
    owners = [
        ("constants", "fixtures/infinite_constant_data.rs", []),
        ("second", "fixtures/infinite_constant_second.rs", []),
        ("middle", "fixtures/infinite_constant_middle.rs", ["constants", "second"]),
        ("root", "fixtures/infinite_constant_root.rs", ["middle"]),
    ]
    for owner, source, dependencies in owners:
        rust_source_metadata(
            name = "infinite_constant_" + owner + "_metadata",
            source = source,
            crate_name = owner,
            crate_key = "proof.infinite.constant." + owner,
            dependencies = {dep: ":infinite_constant_" + dep + "_metadata" for dep in dependencies},
        )
    libraries = []
    references = []
    for suffix, opt, checks in [("checked", "0", "yes"), ("optimized", "2", "no")]:
        flags = ["-Copt-level=" + opt, "-Coverflow-checks=" + checks]
        for owner, source, dependencies in owners:
            library = "infinite_constant_" + owner + "_" + suffix
            libraries.append(":" + library)
            rust_library(
                name = library,
                crate_name = owner,
                crate_root = source,
                srcs = [source],
                edition = "2024",
                rustc_flags = flags,
                deps = [":infinite_constant_" + dep + "_" + suffix for dep in dependencies],
            )
        reference = "infinite_constant_source_reference_" + suffix
        references.append(":" + reference)
        rust_binary(
            name = reference,
            testonly = True,
            srcs = ["fixtures/reference_infinite_source.rs"],
            edition = "2024",
            rustc_flags = flags,
            deps = [":infinite_constant_root_" + suffix],
        )
    rust_clippy_test(name = "infinite_constant_source_clippy_test", targets = libraries + references)
    rustfmt_test(name = "infinite_constant_source_rustfmt_test", targets = libraries + references)
    for owner in ["middle", "root"]:
        rust_c_bundle(name = "generated_c_infinite_constant_" + owner, crate = ":infinite_constant_" + owner + "_metadata")
        rust_java_bundle(name = "generated_java_infinite_constant_" + owner, crate = ":infinite_constant_" + owner + "_metadata")
    artifacts = [
        ":generated_" + language + "_infinite_constant_" + owner
        for owner in ["middle", "root"]
        for language in ["c", "java"]
    ] + references + ["//tools/c:zig_native_oracle"]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/infinite_constant_source_native.py)"] + ["$(rootpath " + target + ")" for target in artifacts],
        data = artifacts + [
            "test/infinite_constant_source_native.py",
            "test/infinite_constant_source_inventory.py",
            "test/infinite_constant_source_truth.py",
            "test/finite_constant_source_consumers.py",
            "test/constant_export_native.py",
            "test/constant_export_scratch.py",
            ":infinite_constant_oracle_support",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ],
    )
