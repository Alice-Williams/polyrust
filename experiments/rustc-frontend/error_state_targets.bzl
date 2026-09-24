"""Native safe-public-constructor evidence for original standard error state."""

load("@rules_rust//rust:defs.bzl", "rust_clippy_test", "rust_test", "rustfmt_test")

def error_state_targets(name):
    """Keep native semantic evidence separate from compiler observation.

    Args:
        name: Native oracle test target.
    """
    rust_test(
        name = name,
        srcs = ["test/error_state_native.rs"],
        crate_root = "test/error_state_native.rs",
        edition = "2024",
        deps = ["//crates/codegen:portable_codegen"],
    )
    rust_clippy_test(name = "error_state_native_clippy_test", targets = [":" + name])
    rustfmt_test(name = "error_state_native_format_test", targets = [":" + name])
