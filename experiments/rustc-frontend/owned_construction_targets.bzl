"""Independent compiler constructor capability proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_construction_targets(name):
    """Declare typed constructor admission tests.

    Args:
        name: Test target name.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob([
        "src/owned_source/*.rs",
        "test/owned_construction/*.rs",
    ])
    compiler_adapter(name = "owned_construction_probe", srcs = sources, crate_root = "test/owned_construction/main.rs")
    adapter_format_test(name = "owned_construction_format_test", srcs = sources + ["fixtures/owned_construction.rs"])
    sh_test(
        name = name,
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/owned_construction_test.py)", "$(rootpath :owned_construction_probe)", "$(rootpath fixtures/owned_construction.rs)"],
        data = ["test/owned_construction_test.py", ":owned_construction_probe", "fixtures/owned_construction.rs"],
    )
    contracts = []
    for case, error, count in [
        ("missing", "error[E0599]", 1),
        ("duplicate", "error[E0599]", 1),
        ("wrong_capability", "error[E0271]", 1),
        ("wrong_context", "error[E0271]", 2),
        ("wrong_output", "error[E0271]", 2),
        ("wrong_input", "error[E0308]", 1),
        ("private_input", "error[E0451]", 1),
    ]:
        compiler_adapter_compile_fail_test(
            name = "owned_construction_" + case + "_test",
            crate_root = "test/owned_construction/main.rs",
            srcs = sources,
            rustc_cfg = "owned_contract_" + case,
            expected_error = error,
            expected_errors = count,
        )
        contracts.append(":owned_construction_" + case + "_test")
    native.test_suite(name = "owned_construction_contract_test", tests = contracts)
