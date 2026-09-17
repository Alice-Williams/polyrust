"""Java-owned compiler adapter and single-crate source proof."""

load("@rules_cc//cc:defs.bzl", "cc_binary")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def java_source_targets(name):
    """Declare the compiler-to-Java adapter.

    Args:
        name: Adapter target name.
    """
    compiler_adapter(
        name = name,
        crate_root = "src/java_main.rs",
        srcs = [
            "src/inputs.rs",
            "src/java_main.rs",
        ] + ["src/source_admission.rs"] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ]),
        deps = [
            ":compiler_configuration",
            "//crates/backend-java:portable_backend_java",
            "//crates/binary64:portable_binary64",
            "//crates/codegen:portable_codegen",
            "//crates/diagnostics:portable_diagnostics",
        ],
    )
    rust_binary(
        name = "rust_boolean_order",
        srcs = ["fixtures/reference_boolean_order.rs", "fixtures/boolean_order.rs"],
        crate_root = "fixtures/reference_boolean_order.rs",
        edition = "2024",
    )
    rust_clippy_test(name = "boolean_order_clippy_test", targets = [":rust_boolean_order"])
    native.genrule(
        name = "generate_boolean_order",
        srcs = ["fixtures/boolean_order.rs"],
        outs = ["generated/boolean_order.c"],
        cmd = "$(location :adapter) $(location fixtures/boolean_order.rs) $@",
        tools = [":adapter"],
    )
    sh_test(
        name = "java_source_negative_test",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/java_source_negative.py)", "$(rootpath :" + name + ")", "$(rootpath fixtures/model.rs)"],
        data = [":" + name, "test/java_source_negative.py"] + native.glob(["fixtures/*"]),
    )
    sh_test(
        name = "java_call_negative_test",
        srcs = ["test/java_call_negative_test.sh"],
        args = ["$(rootpath :" + name + ")"],
        data = [":" + name],
    )
    sh_test(
        name = "source_admission_budget_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/source_admission_test.py)", "$(rootpath :adapter)", "$(rootpath :" + name + ")"],
        data = [":adapter", ":" + name, "test/source_admission_test.py"],
    )
    sh_test(
        name = "source_admission_nested_budget_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/source_admission_nested_test.py)", "$(rootpath :adapter)", "$(rootpath :" + name + ")"],
        data = [":adapter", ":" + name, "test/source_admission_nested_test.py"],
    )
    for fixture, rust, generation, extras in [
        ("model", ":rust_reference", ":generate", []),
        ("alternate", ":rust_alternate", ":generate_alternate", []),
        ("scopes", ":rust_scopes", ":generate_scopes", []),
        ("mapping_inventory", ":rust_mapping_inventory", ":generate_mapping_inventory", []),
        ("documentation", ":rust_documentation", ":generate_documentation", ["fixtures/documentation.md"]),
        ("direct_calls", ":rust_direct_calls", ":generate_direct_calls", ["fixtures/direct_calls_out.rs"]),
        ("boolean_order", ":rust_boolean_order", ":generate_boolean_order", []),
    ]:
        source = "fixtures/" + fixture + ".rs"
        native.genrule(
            name = "generate_java_" + fixture,
            srcs = [source] + extras,
            outs = ["generated/java/" + fixture + "/Generated.java"],
            cmd = "$(location :" + name + ") $(location " + source + ") $@" + "".join([" --input $(location " + extra + ")" for extra in extras]),
            tools = [":" + name],
        )
        c_targets = []
        for optimization in ["0", "2"]:
            target = "java_oracle_c_" + fixture + "_o" + optimization
            cc_binary(
                name = target,
                srcs = [generation, "fixtures/consumer.c"],
                copts = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes", "-Wmissing-prototypes", "-O" + optimization],
            )
            c_targets.append(":" + target)
        sh_test(
            name = "java_source_" + fixture + "_test",
            size = "medium",
            srcs = ["test/java_source_native.sh"],
            args = [
                "$(rootpath test/java_source_native.py)",
                "$(rootpath :" + name + ")",
                "$(rootpath " + rust + ")",
                "$(rootpath " + c_targets[0] + ")",
                "$(rootpath " + c_targets[1] + ")",
                "$(rootpath fixtures/" + fixture + ".rs)",
                "$(rootpath fixtures/inputs.txt)",
                "$(rootpath test/JavaSourceConsumer.java.in)",
                "$(rootpath :generate_java_" + fixture + ")",
            ] + ["$(rootpath " + extra + ")" for extra in extras],
            data = [
                ":" + name,
                rust,
                "fixtures/" + fixture + ".rs",
                "fixtures/inputs.txt",
                "test/java_source_native.py",
                "test/JavaSourceConsumer.java.in",
                ":generate_java_" + fixture,
                "@bazel_tools//tools/jdk:current_java_runtime",
            ] + c_targets + extras,
        )
