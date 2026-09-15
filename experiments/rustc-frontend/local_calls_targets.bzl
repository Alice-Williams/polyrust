"""Typed direct-local-call identity and registration proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def local_calls_targets(name):
    """Declare the typed direct-local-call capability proof.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob(["src/owned_source/*.rs", "test/local_calls/*.rs"])
    compiler_adapter(name = "local_calls_probe", srcs = sources, crate_root = "test/local_calls/main.rs")
    adapter_format_test(name = "local_calls_format_test", srcs = sources + ["fixtures/local_calls.rs"])
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
        compiler_adapter_compile_fail_test(name = "local_calls_" + case + "_test", srcs = sources, crate_root = "test/local_calls/main.rs", rustc_cfg = "local_call_" + case, expected_error = error, expected_errors = count)
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/local_calls_test.py)", "$(rootpath :local_calls_probe)", "$(rootpath fixtures/local_calls.rs)"], data = ["test/local_calls_test.py", ":local_calls_probe", "fixtures/local_calls.rs"])
