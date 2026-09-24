"""Source-only manifest rejection of canonical type edges before graph support."""

load("@rules_rust//rust:defs.bzl", "rust_test")

def canonical_owner_manifest_targets():
    rust_test(
        name = "c_canonical_owner_manifest_test",
        srcs = [
            "test/canonical_owner_manifest.rs",
            "//crates/backend-c:canonical_dependency_fixtures",
        ] + native.glob(["src/api_manifest/**/*.rs"]),
        crate_root = "test/canonical_owner_manifest.rs",
        edition = "2024",
        visibility = ["//:__pkg__"],
        deps = [
            "//crates/backend-c:portable_backend_c",
            "//crates/binary64:portable_binary64",
            "//crates/codegen:portable_codegen",
        ],
    )
