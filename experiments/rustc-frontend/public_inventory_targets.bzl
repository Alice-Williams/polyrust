"""Shared compiler public export inventory and construction boundary tests."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter", "compiler_adapter_compile_fail_test")

def public_inventory_targets(name):
    """Declare native rustc inventory and private-construction checks.

    Args:
        name: The compiler-backed behavioral probe test.
    """
    sources = [
        "src/inputs.rs",
        "test/public_inventory_main.rs",
        "test/source_constant_facts_support.rs",
        "src/source_capabilities/constant_values.rs",
        "src/source_capabilities/constant_evaluation.rs",
    ] + native.glob(["src/source_origin/**/*.rs"])
    compiler_adapter(
        name = "public_inventory_probe",
        srcs = sources,
        crate_root = "test/public_inventory_main.rs",
        deps = ["//crates/codegen:portable_codegen", "//crates/binary64:portable_binary64"],
    )
    compiler_adapter_compile_fail_test(
        name = "public_inventory_private_test",
        srcs = sources,
        crate_root = "test/public_inventory_main.rs",
        rustc_cfg = "public_inventory_private",
        expected_error = "error[E0451]",
        deps = ["//crates/codegen:portable_codegen", "//crates/binary64:portable_binary64"],
    )
    sh_test(
        name = name,
        srcs = ["test/public_inventory_test.sh"],
        args = [
            "$(rootpath :public_inventory_probe)",
            "$(rootpath test/public_inventory_test.py)",
        ],
        data = [
            ":public_inventory_probe",
            "test/public_inventory_test.py",
        ],
    )
