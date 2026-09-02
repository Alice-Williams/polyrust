"""Strict native tests for a generated PolyRust Java package."""

load("@rules_java//java:defs.bzl", "java_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def generated_java_tests():
    """Defines strict positive, negative-type, and conformance Java tests."""
    common = [
        ":generated/java/src/main/java/org/polyrust/generated/Generated.java",
        ":generated/java/src/main/java/org/polyrust/generated/Runtime.java",
    ]
    options = [
        "-Werror",
        "-Xlint:all",
    ]
    java_test(
        name = "java_generated_test",
        srcs = common + [
            ":generated/java/src/test/java/org/polyrust/generated/GeneratedTest.java",
        ],
        javacopts = options,
        main_class = "org.polyrust.generated.GeneratedTest",
        use_testrunner = False,
    )
    sh_test(
        name = "java_negative_type_test",
        srcs = ["//tools/java:negative_compile_test.sh"],
        args = [
            "$(location :generated/java/src/main/java/org/polyrust/generated/Runtime.java)",
            "$(location :generated/java/negative/InvalidTypes.java)",
        ],
        data = [
            ":generated/java/negative/InvalidTypes.java",
            ":generated/java/src/main/java/org/polyrust/generated/Runtime.java",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ],
    )
    java_test(
        name = "java_conformance_test",
        srcs = common + [
            ":generated/java/src/test/java/org/polyrust/generated/ConformanceTest.java",
        ],
        javacopts = options,
        main_class = "org.polyrust.generated.ConformanceTest",
        use_testrunner = False,
    )
