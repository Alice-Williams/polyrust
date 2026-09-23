"""Original character crates, profile-specific Rust references and target bundles."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def character_constant_source_targets(name):
    """Declare independent original-source and generated native proofs.

    Args:
        name: Differential native test target.
    """
    owners = [
        ("constants", "fixtures/character_constant_data.rs", []),
        ("second", "fixtures/character_constant_second.rs", []),
        ("middle", "fixtures/character_constant_middle.rs", ["constants", "second"]),
        ("root", "fixtures/character_constant_root.rs", ["middle"]),
    ]
    for owner, source, dependencies in owners:
        rust_source_metadata(
            name = "character_constant_" + owner + "_metadata",
            source = source,
            crate_name = owner,
            crate_key = "proof.character.constant." + owner,
            dependencies = {dep: ":character_constant_" + dep + "_metadata" for dep in dependencies},
        )
    libraries = []
    references = []
    for suffix, opt, checks in [("checked", "0", "yes"), ("optimized", "2", "no")]:
        flags = ["-Copt-level=" + opt, "-Coverflow-checks=" + checks]
        for owner, source, dependencies in owners:
            library = "character_constant_" + owner + "_" + suffix
            libraries.append(":" + library)
            rust_library(
                name = library,
                crate_name = owner,
                crate_root = source,
                srcs = [source],
                edition = "2024",
                rustc_flags = flags,
                deps = [":character_constant_" + dep + "_" + suffix for dep in dependencies],
            )
        reference = "character_constant_source_reference_" + suffix
        references.append(":" + reference)
        rust_binary(
            name = reference,
            testonly = True,
            srcs = ["fixtures/reference_character_constant_source.rs"],
            edition = "2024",
            rustc_flags = flags,
            deps = [":character_constant_root_" + suffix],
        )
    rust_clippy_test(name = "character_constant_source_clippy_test", targets = libraries + references)
    rustfmt_test(name = "character_constant_source_rustfmt_test", targets = libraries + references)
    probes = [
        ":character_c_probe",
        ":character_java_probe",
        ":generated_c_character_constant_root",
        ":generated_java_character_constant_root",
    ] + [source for _, source, _ in owners] + [
        ":character_constant_" + owner + "_metadata"
        for owner, _, _ in owners
    ] + [
        ":constant_import_" + language + "_" + fault
        for language in ["c", "java"]
        for fault in ["wrong_declaration", "wrong_type", "wrong_value", "replaced_owner", "wrong_owner"]
    ]
    sh_test(
        name = "character_constant_source_ast_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/character_constant_source_ast.py)"] + ["$(rootpath " + target + ")" for target in probes],
        data = probes + ["test/character_constant_source_ast.py"],
    )
    for owner in ["middle", "root"]:
        rust_c_bundle(name = "generated_c_character_constant_" + owner, crate = ":character_constant_" + owner + "_metadata")
        rust_java_bundle(name = "generated_java_character_constant_" + owner, crate = ":character_constant_" + owner + "_metadata")
    artifacts = [
        ":generated_" + language + "_character_constant_" + owner
        for owner in ["middle", "root"]
        for language in ["c", "java"]
    ] + references + ["//tools/c:zig_native_oracle"]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/character_constant_source_native.py)"] + ["$(rootpath " + target + ")" for target in artifacts],
        data = artifacts + [
            "test/character_constant_source_native.py",
            "test/character_constant_source_inventory.py",
            "test/character_constant_source_truth.py",
            "test/constant_export_native.py",
            "test/constant_export_scratch.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ],
    )
