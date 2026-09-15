"""Typed source record construction and capability registration proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_record_targets(name):
    """Declare independently cached record-construction contracts.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob(["src/owned_source/*.rs", "test/owned_record/*.rs"])
    fixtures = ["fixtures/owned_record.rs", "fixtures/record_limit_128.rs", "fixtures/record_limit_129.rs"]
    compiler_adapter(name = "owned_record_probe", srcs = sources, crate_root = "test/owned_record/main.rs", rustc_cfg = "owned_record_proof")
    adapter_format_test(name = "owned_record_format_test", srcs = sources + fixtures)
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/owned_record_test.py)", "$(rootpath :owned_record_probe)"] + ["$(rootpath " + f + ")" for f in fixtures], data = ["test/owned_record_test.py", ":owned_record_probe"] + fixtures)
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
    ]:
        compiler_adapter_compile_fail_test(name = "owned_record_" + case + "_test", srcs = sources, crate_root = "test/owned_record/main.rs", rustc_cfg = "record_" + case, expected_error = error, expected_errors = count)
