"""Early return and enclosing continuation proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_early_targets(name):
    """Declare the separate early-return grammar proof.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs", "test/owned_returns/compatibility.rs"] + native.glob([
        "src/owned_source/*.rs",
        "src/owned_linear/**/*.rs",
        "test/owned_early/*.rs",
    ])
    compiler_adapter(name = "owned_early_probe", srcs = sources, crate_root = "test/owned_early/main.rs", rustc_cfgs = ["owned_early_proof"])
    compiler_adapter_compile_fail_test(name = "owned_early_private_test", srcs = sources, crate_root = "test/owned_early/main.rs", rustc_cfg = "owned_early_private", expected_error = "error[E0451]", expected_errors = 1)
    compiler_adapter_compile_fail_test(name = "owned_early_not_if_else_test", srcs = sources, crate_root = "test/owned_early/main.rs", rustc_cfg = "owned_early_not_if_else", expected_error = "error[E0308]", expected_errors = 1)
    adapter_format_test(name = "owned_early_format_test", srcs = sources + ["fixtures/owned_early.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/owned_early_test.py)", "$(rootpath :owned_early_probe)", "$(rootpath fixtures/owned_early.rs)"], data = ["test/owned_early_test.py", ":owned_early_probe", "fixtures/owned_early.rs"])
