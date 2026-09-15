"""Compiler public-package fixtures and independent native consumers."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")
load(":package_output.bzl", "rust_public_package")

def public_package_targets(name):
    """Declare the real-Rust package smoke and typed metadata contracts.

    Args:
        name: The separately compiled native public-package proof target.
    """
    rust_public_package(
        name = "generate_public_package",
        source = "fixtures/public_package.rs",
    )
    rust_library(
        name = "public_package_model",
        srcs = ["fixtures/public_package.rs"],
        crate_root = "fixtures/public_package.rs",
        edition = "2024",
    )
    rust_binary(
        name = "rust_public_package",
        srcs = ["fixtures/reference_public_package.rs"],
        crate_root = "fixtures/reference_public_package.rs",
        edition = "2024",
        deps = [":public_package_model"],
    )
    rust_clippy_test(
        name = "public_package_clippy_test",
        targets = [":public_package_model", ":rust_public_package"],
    )
    compiler_adapter(
        name = "public_package_adapter",
        srcs = ["src/inputs.rs", "test/public_package_main.rs", "test/public_package_manifest_mutations.rs", "test/package_state_c.rs"] + ["src/source_admission.rs"] + native.glob(["src/c_lower/**/*.rs", "src/source_capabilities/**/*.rs", "src/source_origin/**/*.rs", "src/api_manifest/**/*.rs"]),
        crate_root = "test/public_package_main.rs",
        rustc_cfg = "public_package_contract",
        deps = ["//crates/backend-c:portable_backend_c", "//crates/codegen:portable_codegen"],
    )
    sh_test(
        name = "public_package_manifest_test",
        srcs = ["test/package_state_c_test.sh"],
        args = ["$(rootpath :public_package_adapter)", "$(rootpath fixtures/public_package.rs)"],
        data = [":public_package_adapter", "fixtures/public_package.rs"],
    )
    sh_test(
        name = "public_package_negative_test",
        srcs = ["test/public_package_negative_test.sh"],
        args = ["$(rootpath :adapter)", "$(rootpath test/public_package_negative_test.py)"],
        data = [":adapter", "test/public_package_negative_test.py"],
    )
    sh_test(
        name = name,
        srcs = ["test/public_package_test.sh"],
        args = [
            "$(rootpath :adapter)",
            "$(rootpath fixtures/public_package.rs)",
            "$(rootpath test/public_package_assertions.py)",
            "$(rootpath //tools/c:zig_native_oracle)",
            "$(rootpath :rust_public_package)",
            "$(rootpath fixtures/inputs.txt)",
            "$(rootpath :generate_public_package)",
        ],
        data = [":adapter", "fixtures/public_package.rs", "test/public_package_assertions.py", "//tools/c:zig_native_oracle", ":rust_public_package", "fixtures/inputs.txt", ":generate_public_package"],
    )
