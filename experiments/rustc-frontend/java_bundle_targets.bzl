"""Independent typed Java bundle projection, reservation and JSON actions."""

load("@rules_rust//rust:defs.bzl", "rust_clippy_test", "rust_library", "rust_test")

def java_bundle_targets(name):
    """Declare the pure bundle library and its unit/lint gates."""
    rust_library(
        name = name,
        srcs = native.glob(["java_bundle/*.rs"]),
        crate_root = "java_bundle/lib.rs",
        crate_name = "portable_java_bundle",
        edition = "2024",
        deps = ["//crates/backend-java:portable_backend_java", "//crates/codegen:portable_codegen", "//crates/diagnostics:portable_diagnostics"],
    )
    rust_test(name = name + "_test", crate = ":" + name)
    rust_clippy_test(name = name + "_clippy_test", targets = [":" + name, ":" + name + "_test"])
