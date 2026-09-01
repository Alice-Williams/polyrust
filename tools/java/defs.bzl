"""Strict native tests for a generated PolyRust Java package."""

load("@rules_java//java:defs.bzl", "java_test")

def generated_java_tests():
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
    java_test(
        name = "java_conformance_test",
        srcs = common + [
            ":generated/java/src/test/java/org/polyrust/generated/ConformanceTest.java",
        ],
        javacopts = options,
        main_class = "org.polyrust.generated.ConformanceTest",
        use_testrunner = False,
    )
