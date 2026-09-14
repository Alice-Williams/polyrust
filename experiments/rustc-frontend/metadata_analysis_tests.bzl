"""Real Bazel analysis rejects malformed record fields before action emission."""

load(":metadata_output.bzl", "rust_source_metadata")

def _rejection_impl(ctx):
    target = ctx.attr.target_under_test[0]
    messages = []
    if AnalysisFailureInfo in target:
        messages = [cause.message for cause in target[AnalysisFailureInfo].causes.to_list()]
    expected = "Rust metadata record contains a control character"
    return [AnalysisTestResultInfo(
        success = any([expected in message for message in messages]),
        message = "expected metadata framing rejection; actual: " + "\n".join(messages),
    )]

_rejection_test = rule(
    implementation = _rejection_impl,
    analysis_test = True,
    attrs = {
        "target_under_test": attr.label(cfg = analysis_test_transition(settings = {
            "//command_line_option:allow_analysis_failures": True,
        })),
    },
)

def metadata_analysis_tests(name):
    """Declare intentional invalid fixtures, executed only by rejection tests.

    Args:
        name: Aggregate analysis rejection test suite.
    """
    tests = []
    for case, changes in [
        ("input", {"inputs": {"fixtures/documentation.md": "doc.md\n--input\n/etc/passwd\netc/passwd"}}),
        ("source_name", {"source_name": "lib.rs\n--input\n/etc/passwd\netc/passwd"}),
        ("crate_name", {"crate_name": "fixture\r\n--root"}),
        ("crate_key", {"crate_key": "fixture\n--root"}),
        ("alias", {"dependencies": {"renamed\n--input": ":generated_leaf_metadata"}}),
        ("tab", {"source_name": "lib\t.rs"}),
    ]:
        attributes = dict(
            name = "invalid_metadata_" + case,
            source = "fixtures/crate_identity.rs",
            crate_name = "fixture",
            crate_key = "fixture.key",
            # Intentional negative fixtures must not be direct wildcard builds.
            tags = ["manual"],
        )
        attributes.update(changes)
        rust_source_metadata(**attributes)
        _rejection_test(
            name = "metadata_" + case + "_rejection_test",
            target_under_test = ":invalid_metadata_" + case,
        )
        tests.append(":metadata_" + case + "_rejection_test")
    native.test_suite(name = name, tests = tests)
