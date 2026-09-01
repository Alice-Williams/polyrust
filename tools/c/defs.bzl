"""Strict native tests for a generated PolyRust C17 package."""

load("@rules_cc//cc:defs.bzl", "cc_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def generated_c_tests():
    """Defines strict native, style, ASan, and UBSan tests for generated C17."""
    common = [
        ":generated/c/src/generated.c",
        ":generated/c/src/generated.h",
        ":generated/c/src/runtime.c",
        ":generated/c/src/runtime.h",
    ]
    options = [
        "-std=c17",
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-Werror",
    ]
    cc_test(
        name = "c_generated_test",
        srcs = common + [":generated/c/tests/generated_test.c"],
        copts = options,
        includes = ["generated/c/src"],
    )
    sh_test(
        name = "c_sanitizer_test",
        srcs = ["//tools/c:test_sanitizers.sh"],
        data = common + [":generated/c/tests/generated_test.c"],
    )
    sh_test(
        name = "c_style_test",
        srcs = ["//tools/c:test_style.sh"],
        data = common + [
            ":generated/c/tests/conformance_test.c",
            ":generated/c/tests/generated_test.c",
        ],
    )
    cc_test(
        name = "c_conformance_test",
        srcs = common[2:] + [":generated/c/tests/conformance_test.c"],
        copts = options,
        includes = ["generated/c/src"],
    )
