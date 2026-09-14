"""Same real Rust diamond as independently compiled native Rust and C crates."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def crate_native_targets(name):
    """Declare the source diamond and native differential matrix.

    Args:
        name: The separate-crate native proof target.
    """
    libraries = []
    for owner, dependencies in [
        ("leaf", []),
        ("left", ["leaf"]),
        ("right", ["leaf"]),
        ("root", ["left", "right"]),
    ]:
        source = "fixtures/crate_proof_" + owner + ".rs"
        library = "crate_proof_" + owner
        libraries.append(":" + library)
        rust_library(
            name = library,
            srcs = [source],
            crate_name = owner,
            crate_root = source,
            edition = "2024",
            deps = [":crate_proof_" + dependency for dependency in dependencies],
        )
        rust_source_metadata(
            name = library + "_metadata",
            source = source,
            crate_name = owner,
            crate_key = "native.diamond." + owner + ".v1",
            dependencies = {dependency: ":crate_proof_" + dependency + "_metadata" for dependency in dependencies},
        )
    rust_binary(
        name = "rust_crate_proof",
        srcs = ["fixtures/reference_crate_proof.rs"],
        crate_root = "fixtures/reference_crate_proof.rs",
        edition = "2024",
        deps = libraries,
    )
    rust_clippy_test(
        name = "crate_proof_clippy_test",
        targets = libraries + [":rust_crate_proof"],
    )
    rust_c_bundle(
        name = "generated_crate_proof",
        crate = ":crate_proof_root_metadata",
    )
    sh_test(
        name = name,
        size = "medium",
        srcs = ["test/crate_native_test.sh"],
        args = [
            "$(rootpath :generated_crate_proof)",
            "$(rootpath :rust_crate_proof)",
            "$(rootpath fixtures/inputs.txt)",
            "$(rootpath //tools/c:zig_native_oracle)",
            "$(rootpath test/crate_native_consumer.py)",
            "$(rootpath test/crate_native_symbols.py)",
        ],
        data = [":generated_crate_proof", ":rust_crate_proof", "fixtures/inputs.txt", "//tools/c:zig_native_oracle", "test/crate_native_consumer.py", "test/crate_native_symbols.py"],
    )
