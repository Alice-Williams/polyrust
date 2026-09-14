"""Independently cached Linux atomic no-replace publisher and platform tests."""

load("@rules_cc//cc:defs.bzl", "cc_binary")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library", "rust_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def publication_probe_targets(name):
    """Declare the Linux publisher and its native filesystem proof.

    Args:
        name: Native filesystem behavior test target.
    """
    cc_binary(
        name = "directory_publisher",
        srcs = ["platform/rename_noreplace.c"],
        copts = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror"],
    )
    rust_library(
        name = "directory_publication",
        srcs = native.glob(["publication/*.rs"]),
        crate_root = "publication/lib.rs",
        crate_name = "portable_directory_publication",
        edition = "2024",
    )
    rust_test(
        name = "directory_publication_unit_test",
        crate = ":directory_publication",
    )
    rust_binary(
        name = "directory_tree_contract",
        srcs = ["test/directory_tree_main.rs"],
        edition = "2024",
        deps = [":directory_publication"],
    )
    sh_test(
        name = "directory_tree_test",
        srcs = ["test/metadata_action_test.sh"],
        args = ["$(rootpath test/directory_tree_test.py)", "$(rootpath :directory_tree_contract)", "$(rootpath :directory_publisher)"],
        data = ["test/directory_tree_test.py", ":directory_tree_contract", ":directory_publisher"],
    )
    rust_binary(
        name = "directory_publication_contract",
        srcs = ["test/directory_publication_main.rs", "src/output/publication.rs"],
        crate_root = "test/directory_publication_main.rs",
        edition = "2024",
        deps = [":directory_publication"],
    )
    rust_test(
        name = "bundle_budget_test",
        srcs = ["test/bundle_budget.rs", "src/output/bundle_budget.rs"],
        crate_root = "test/bundle_budget.rs",
        edition = "2024",
    )
    rust_clippy_test(
        name = "directory_publication_clippy_test",
        targets = [":directory_publication_contract", ":bundle_budget_test", ":directory_publication", ":directory_publication_unit_test", ":directory_tree_contract"],
    )
    sh_test(
        name = "directory_publication_test",
        srcs = ["test/metadata_action_test.sh"],
        args = ["$(rootpath test/directory_publication_test.py)", "$(rootpath :directory_publication_contract)", "$(rootpath :directory_publisher)"],
        data = ["test/directory_publication_test.py", ":directory_publication_contract", ":directory_publisher"],
    )
    sh_test(
        name = name,
        srcs = ["test/metadata_action_test.sh"],
        args = ["$(rootpath test/rename_noreplace_test.py)", "$(rootpath :directory_publisher)"],
        data = ["test/rename_noreplace_test.py", ":directory_publisher"],
    )
