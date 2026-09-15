"""Typed nested-record operations, limits and private registration contracts."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def nested_record_targets(name):
    """Declare independently cached nested-construction proof targets.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob(["src/owned_source/*.rs", "test/nested_record/*.rs"])
    fixtures = ["fixtures/nested_construction.rs", "fixtures/nested_budgets.rs"]
    compiler_adapter(name = "nested_record_probe", srcs = sources, crate_root = "test/nested_record/main.rs", rustc_cfg = "nested_record_proof")
    adapter_format_test(name = "nested_record_format_test", srcs = sources + fixtures)
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/nested_record_test.py)", "$(rootpath :nested_record_probe)"] + ["$(rootpath " + f + ")" for f in fixtures], data = ["test/nested_record_test.py", ":nested_record_probe"] + fixtures)
    for case, error, count in [
        ("missing", "error[E0277]", 1),
        ("missing_base", "error[E0599]", 1),
        ("duplicate", "error[E0599]", 1),
        ("wrong_capability", "error[E0271]", 1),
        ("wrong_context", "error[E0271]", 2),
        ("wrong_output", "error[E0271]", 2),
        ("wrong_input", "error[E0308]", 1),
        ("private_input", "error[E0451]", 1),
        ("private_field", "error[E0451]", 1),
        ("private_layout", "error[E0451]", 1),
        ("private_initializer", "error[E0451]", 1),
    ]:
        compiler_adapter_compile_fail_test(name = "nested_record_" + case + "_test", srcs = sources, crate_root = "test/nested_record/main.rs", rustc_cfg = "nested_" + case, expected_error = error, expected_errors = count)
