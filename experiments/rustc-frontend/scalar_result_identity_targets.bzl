"""Backend-independent checked Result identity observation."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def scalar_result_identity_targets(name):
    """Declare observation/private-construction/format gates.

    Args:
        name: Compiler-backed behavioral test.
    """
    sources = [
        "src/inputs.rs",
        "test/scalar_result_identity/main.rs",
        "test/scalar_result_identity/shape.rs",
    ]
    compiler_adapter(
        name = "scalar_result_identity_probe",
        srcs = sources,
        crate_root = "test/scalar_result_identity/main.rs",
    )
    compiler_adapter_compile_fail_test(
        name = "scalar_result_identity_private_test",
        srcs = sources,
        crate_root = "test/scalar_result_identity/main.rs",
        rustc_cfg = "scalar_result_forge",
        expected_error = "error[E0451]",
    )
    adapter_format_test(name = "scalar_result_identity_format_test", srcs = sources)
    sh_test(
        name = name,
        srcs = ["test/public_inventory_test.sh"],
        args = [
            "$(rootpath :scalar_result_identity_probe)",
            "$(rootpath test/scalar_result_identity_test.py)",
        ],
        data = [
            ":scalar_result_identity_probe",
            "test/scalar_result_identity_test.py",
        ],
    )
