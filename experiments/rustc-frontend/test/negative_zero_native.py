"""Native/upstream differential proof of an ordinary Rust composition."""
import os
from pathlib import Path
import subprocess
import sys
from negative_zero_inventory import inspect
from negative_zero_oracle import VALUES, VARIANTS, expected, inputs, traces
from negative_zero_consumers import java_consumer, c_consumer
from negative_zero_mutations import mutate
from negative_zero_examples import export
from short_circuit_mutations import instrument
from remainder_source_privacy import java_privacy, c_privacy


def run(command, data=None):
    result = subprocess.run(list(map(str, command)), input=data, capture_output=True,
                            text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout[:1000], result.stderr[:2000])
    return result


def main():
    java_dir, c_dir, rust, traced_rust, zig, upstream, original = map(
        lambda path: Path(path).resolve(), sys.argv[1:])
    java, c, public, private, functions, native = inspect(java_dir, c_dir)
    data, truth = inputs(), expected()
    assert run(["node", "--version"]).stdout.strip() == "v24.20.0"
    upstream_result = run(["node", Path(__file__).with_name("negative_zero_upstream.mjs"), upstream], data)
    assert upstream_result.stdout == truth and upstream_result.stderr == ""
    for reference, variant in [(rust, "plain"), (traced_rust, "traced")]:
        result = run([reference], data)
        assert result.stdout == truth and result.stderr == traces(variant)
    native_trace = run([traced_rust], data).stderr
    assert native_trace != traces("eager") and native_trace != traces("duplicate")
    for variant in ["zero_sign", "unguarded", "flipped"]:
        assert expected(variant) != truth
    work_root = Path(os.environ["TEST_TMPDIR"]) / "negative-zero"
    work_root.mkdir()
    runtimes = [path / "bin" for path in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in path.name and (path / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime = runtimes[0]
    assert run([runtime / "javac", "-version"]).stdout.startswith("javac 21.")
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"

    def java_member(identity):
        path = functions[identity]["target"]["path"]
        return ".".join([path["package"], *path["owners"], path["member"]])

    for is_java, directory, api in [(True, java_dir, java), (False, c_dir, c)]:
        member = java_member(public) if is_java else native[public]["symbol"]
        reciprocal = java_member(private) if is_java else native[private]["symbol"]
        original_text = (directory / api["source" if is_java else "implementation"]).read_text()
        assert "Runtime" not in original_text and "#include <math.h>" not in original_text
        for variant in VARIANTS:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            text = original_text
            if variant != "plain":
                text = instrument(text, {"R": reciprocal.split(".")[-1]}, is_java)
            if variant not in ["plain", "traced"]:
                text = mutate(text, member.split(".")[-1], reciprocal, is_java, variant)
            source = work / ("Generated.java" if is_java else "generated.c")
            source.write_text(text)
            if is_java:
                classes = work / "classes"
                classes.mkdir()
                client = work / "Consumer.java"
                client.write_text(java_consumer(member))
                flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
                run([runtime / "javac", *flags, source])
                run([runtime / "javac", *flags, client])
                results = [run([runtime / "java", "-cp", classes, "Consumer"], data)]
                if variant == "plain":
                    java_privacy(run, work, runtime, classes, source, functions[private])
            else:
                client = work / "consumer.c"
                client.write_text(c_consumer(api["header"], member))
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes",
                         "-Wmissing-prototypes", "-fno-fast-math", "-ffp-contract=off",
                         "-fsigned-char", "-fno-short-enums", "-fno-builtin", "-I", directory]
                results = []
                for compiler in ["gcc-14", zig]:
                    for optimization in ["0", "2"]:
                        label = Path(compiler).name + optimization
                        selected = [*flags, "-O" + optimization]
                        obj, consumer_obj = work / (label + ".o"), work / (label + "-client.o")
                        run([compiler, *selected, "-c", source, "-o", obj])
                        run([compiler, *selected, "-c", client, "-o", consumer_obj])
                        executable = work / label
                        # No -lm or generated runtime is needed.
                        run([compiler, obj, consumer_obj, "-o", executable])
                        results.append(run([executable], data))
                        if variant == "plain":
                            c_privacy(run, work, compiler, selected, source, obj, reciprocal,
                                      label, argument="1.0", expected="1.0")
            for result in results:
                assert result.stdout == expected(variant), (is_java, variant, "value")
                assert result.stderr == traces(variant), (is_java, variant, "per-input trace")
                if variant == "traced":
                    assert result.stderr == native_trace
    export(java_dir, c_dir, original, work_root, len(VALUES))
    print(f"{len(VALUES)} native Rust/upstream/bit-oracle/C/Java observations per run; "
          "per-input traces; five compiling faults; external privacy controls")


if __name__ == "__main__":
    main()
