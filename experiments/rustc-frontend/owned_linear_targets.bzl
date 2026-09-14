"""Closed linear ownership correspondence proof without target code."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_linear_targets(name):
    """Declare the separately cached correspondence experiment.

    Args:
        name: Runtime assertion test target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob([
        "src/owned_source/*.rs",
        "src/owned_linear/*.rs",
        "test/owned_linear/*.rs",
    ])
    compiler_adapter(name = "owned_linear_probe", srcs = sources, crate_root = "test/owned_linear/main.rs", rustc_cfgs = ["owned_linear_proof"])
    compiler_adapter_compile_fail_test(
        name = "owned_linear_private_test",
        srcs = sources,
        crate_root = "test/owned_linear/main.rs",
        rustc_cfg = "owned_linear_private",
        expected_error = "error[E0451]",
        expected_errors = 1,
    )
    adapter_format_test(name = "owned_linear_format_test", srcs = sources + ["fixtures/owned_linear.rs"])
    sh_test(
        name = name,
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/owned_linear_test.py)", "$(rootpath :owned_linear_probe)", "$(rootpath fixtures/owned_linear.rs)"],
        data = ["test/owned_linear_test.py", ":owned_linear_probe", "fixtures/owned_linear.rs"],
    )
