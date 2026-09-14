"""Conditional owner selection and compiler cleanup proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_selection_targets(name):
    """Declare the pinned conditional ownership proof.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs", "test/owned_returns/compatibility.rs"] + native.glob([
        "src/owned_source/*.rs",
        "src/owned_linear/**/*.rs",
        "test/owned_selection/*.rs",
    ])
    compiler_adapter(name = "owned_selection_probe", srcs = sources, crate_root = "test/owned_selection/main.rs", rustc_cfgs = ["owned_selection_proof"])
    adapter_format_test(name = "owned_selection_format_test", srcs = sources + ["fixtures/owned_selection.rs"])
    for contract, error in [
        ("private_body", "error[E0451]"),
        ("private_path", "error[E0451]"),
        ("private_flag", "error[E0451]"),
        ("not_whole", "error[E0308]"),
        ("not_guarded", "error[E0308]"),
    ]:
        compiler_adapter_compile_fail_test(name = "owned_selection_" + contract + "_test", srcs = sources, crate_root = "test/owned_selection/main.rs", rustc_cfg = "selection_" + contract, expected_error = error, expected_errors = 1)
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/owned_selection_test.py)", "$(rootpath :owned_selection_probe)", "$(rootpath fixtures/owned_selection.rs)"], data = ["test/owned_selection_test.py", ":owned_selection_probe", "fixtures/owned_selection.rs"])
