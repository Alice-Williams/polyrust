"""Explicit crate identity: pure configuration and separate-process proofs."""

load("@rules_rust//rust:defs.bzl", "rust_clippy_test", "rust_library", "rust_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":package_output.bzl", "rust_public_package")

def crate_identity_targets(name):
    """Declare independently cached identity and compiler integration tests.

    Args:
        name: The separate-process native identity proof target.
    """
    rust_library(
        name = "compiler_configuration",
        srcs = ["src/configuration.rs"] + native.glob(["src/configuration/**/*.rs"]),
        crate_name = "portable_rustc_configuration",
        crate_root = "src/configuration.rs",
        edition = "2024",
    )
    rust_test(
        name = "compiler_configuration_test",
        srcs = ["test/compiler_configuration.rs"],
        crate_root = "test/compiler_configuration.rs",
        edition = "2024",
        deps = [":compiler_configuration"],
    )
    rust_test(
        name = "compiler_configuration_unit_test",
        crate = ":compiler_configuration",
    )
    rust_test(
        name = "crate_graph_test",
        srcs = ["test/crate_graph.rs", "test/crate_graph_arguments.rs"],
        crate_root = "test/crate_graph.rs",
        edition = "2024",
        deps = [":compiler_configuration"],
    )
    rust_library(
        name = "crate_identity_model",
        srcs = ["fixtures/crate_identity.rs"],
        crate_root = "fixtures/crate_identity.rs",
        edition = "2024",
    )
    rust_test(
        name = "resolved_inputs_test",
        srcs = ["test/resolved_inputs.rs"],
        crate_root = "test/resolved_inputs.rs",
        edition = "2024",
        deps = [":compiler_configuration"],
    )
    rust_clippy_test(
        name = "crate_identity_clippy_test",
        targets = [":compiler_configuration", ":compiler_configuration_test", ":compiler_configuration_unit_test", ":crate_graph_test", ":crate_identity_model", ":resolved_inputs_test"],
    )
    for key in ["a", "b"]:
        rust_public_package(
            name = "generate_identity_" + key,
            source = "fixtures/crate_identity.rs",
            crate_name = "identity_fixture",
            crate_key = "polyrust.identity." + key,
        )
    sh_test(
        name = name,
        srcs = ["test/crate_identity_test.sh"],
        args = [
            "$(rootpath :adapter)",
            "$(rootpath fixtures/crate_identity.rs)",
            "$(rootpath :generate_identity_a)",
            "$(rootpath :generate_identity_b)",
            "$(rootpath //tools/c:zig_native_oracle)",
            "$(rootpath test/crate_identity_test.py)",
        ],
        data = [":adapter", "fixtures/crate_identity.rs", ":generate_identity_a", ":generate_identity_b", "//tools/c:zig_native_oracle", "test/crate_identity_test.py"],
    )
