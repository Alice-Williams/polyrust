"""Strict native tests for a generated PolyRust C++20 package."""

load("@rules_cc//cc:defs.bzl", "cc_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def generated_cpp_tests():
    """Defines strict native, style, ASan, and UBSan tests for generated C++."""
    common = [
        ":generated/cpp/src/generated.cc",
        ":generated/cpp/src/generated.hpp",
        ":generated/cpp/src/runtime.hpp",
    ]
    options = [
        "-std=c++20",
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-Werror",
    ]
    cc_test(
        name = "cpp_generated_test",
        srcs = common + [":generated/cpp/tests/generated_test.cc"],
        copts = options,
        includes = ["generated/cpp/src"],
    )
    sh_test(
        name = "cpp_sanitizer_test",
        srcs = ["//tools/cpp:test_sanitizers.sh"],
        data = [
            ":generated/cpp/src/generated.cc",
            ":generated/cpp/src/generated.hpp",
            ":generated/cpp/src/runtime.hpp",
            ":generated/cpp/tests/generated_test.cc",
        ],
    )
    sh_test(
        name = "cpp_style_test",
        srcs = ["//tools/cpp:test_style.sh"],
        data = [
            ":generated/cpp/src/generated.cc",
            ":generated/cpp/src/generated.hpp",
            ":generated/cpp/src/runtime.hpp",
            ":generated/cpp/tests/conformance_test.cc",
            ":generated/cpp/tests/generated_test.cc",
        ],
    )
    cc_test(
        name = "cpp_conformance_test",
        srcs = common[1:] + [":generated/cpp/tests/conformance_test.cc"],
        copts = options,
        includes = ["generated/cpp/src"],
    )
