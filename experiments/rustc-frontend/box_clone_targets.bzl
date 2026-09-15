"""Typed scalar Box clone identity and registration proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def box_clone_targets(name):
    """Declare the typed scalar Box clone capability proof.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob(["src/owned_source/*.rs", "test/box_clone/*.rs"])
    compiler_adapter(name = "box_clone_probe", srcs = sources, crate_root = "test/box_clone/main.rs")
    adapter_format_test(name = "box_clone_format_test", srcs = sources + ["fixtures/owned_clone.rs"])
    for case, error, count in [
        ("missing", "error[E0277]", 1),
        ("missing_base", "error[E0599]", 1),
        ("duplicate", "error[E0599]", 1),
        ("wrong_capability", "error[E0271]", 1),
        ("wrong_context", "error[E0271]", 2),
        ("wrong_output", "error[E0271]", 2),
        ("wrong_input", "error[E0308]", 1),
        ("private_input", "error[E0451]", 1),
    ]:
        compiler_adapter_compile_fail_test(name = "box_clone_" + case + "_test", srcs = sources, crate_root = "test/box_clone/main.rs", rustc_cfg = "box_clone_" + case, expected_error = error, expected_errors = count)
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/box_clone_test.py)", "$(rootpath :box_clone_probe)", "$(rootpath fixtures/owned_clone.rs)"], data = ["test/box_clone_test.py", ":box_clone_probe", "fixtures/owned_clone.rs"])
