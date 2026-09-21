"""Actual Rust crates -> C/Java packages: values, visibility, imports and traces."""
import os
from pathlib import Path
import subprocess
import sys
from remainder_source_inventory import inspect, link_options
from remainder_source_oracle import PAIRS, VARIANTS, expected, inputs, traces
from arithmetic_source_consumers import c_consumer, java_consumer
from remainder_source_mutations import mutate
from short_circuit_mutations import instrument
from remainder_source_examples import export
from remainder_source_privacy import c_privacy, java_privacy


def run(command, data=None):
    result = subprocess.run([str(arg) for arg in command], input=data,
                            capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result


def main():
    java_dir, c_dir, reference, traced_reference, zig = [Path(arg).resolve() for arg in sys.argv[1:]]
    order, module, java, c, bindings, functions, native = inspect(java_dir, c_dir)
    private, = [identity for identity, item in functions.items() if not item["externally_reachable"]]
    truth, data = expected(), inputs()
    rust = run([reference], data)
    assert rust.stdout == expected(copies=1) and rust.stderr == ""
    traced_rust = run([traced_reference], data)
    assert traced_rust.stdout == rust.stdout
    assert traced_rust.stderr == "AB" * len(PAIRS)
    native_trace = traced_rust.stderr * 2
    assert native_trace == traces("traced")
    work_root = Path(os.environ["TEST_TMPDIR"]) / "remainder-source"
    work_root.mkdir()
    runtimes = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime = runtimes[0]
    assert run([runtime / "javac", "-version"]).stdout.startswith("javac 21.")
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    expectations = {variant: expected(variant) for variant in VARIANTS}
    for variant, wanted in expectations.items():
        if variant in ["nearest", "swapped", "zero_sign"]:
            assert wanted != truth, variant
        else:
            assert wanted == truth
        if variant in ["dropped", "duplicated", "reversed"]:
            assert traces(variant) != traces("traced")
    for is_java, directory, owners in [(True, java_dir, java), (False, c_dir, c)]:
        def member(owner, name):
            identity = bindings[owner, "value", name][1]
            if not is_java:
                return native[identity]["symbol"]
            path = functions[identity]["target"]["path"]
            return ".".join([path["package"], *path["owners"], path["member"]])

        left, right = [member(order[0], name) for name in ["left", "right"]]
        calls = [member(owner, "remainder") + "(left, right)" for owner in [module, order[-1]]]
        for variant in VARIANTS:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in order:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if owner == order[0] and variant != "plain":
                    text = instrument(text, {"A": left.split(".")[-1], "B": right.split(".")[-1]}, is_java)
                if owner == order[1] and variant not in ["plain", "traced"]:
                    name = "remainder"
                    text = mutate(text, member(module, name).split(".")[-1], left, right, is_java, variant)
                location = work / owner.split(":")[0]
                location.mkdir()
                source = location / ("Generated.java" if is_java else "generated.c")
                source.write_text(text, encoding="utf-8")
                sources.append(source)
            if is_java:
                consumer = work / "Consumer.java"
                consumer.write_text(java_consumer(calls))
                classes = work / "classes"
                classes.mkdir()
                flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
                for source in [*sources, consumer]:
                    run([runtime / "javac", *flags, source])
                results = [run([runtime / "java", "-cp", classes, "Consumer"], data)]
                if variant == "plain":
                    java_privacy(run, work, runtime, classes, sources[0], functions[private])
            else:
                consumer = work / "consumer.c"
                headers = "".join(f'#include "{c[owner]["header"]}"\n' for owner in order)
                consumer.write_text(c_consumer(calls, headers))
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes",
                         "-Wmissing-prototypes", "-fno-fast-math", "-ffp-contract=off",
                         "-fsigned-char", "-fno-short-enums", "-fno-builtin", "-I", c_dir]
                results = []
                for compiler in ["gcc-14", zig]:
                    for optimization in ["0", "2"]:
                        label = Path(compiler).name + optimization
                        objects = []
                        for index, source in enumerate([*sources, consumer]):
                            obj = work / (label + str(index) + ".o")
                            run([compiler, *flags, "-O" + optimization, "-c", source, "-o", obj])
                            objects.append(obj)
                        executable = work / label
                        if variant == "plain":
                            c_privacy(run, work, compiler, [*flags, "-O" + optimization],
                                      sources[0], objects[0], native[private]["symbol"], label)
                        libraries = link_options(c[order[-1]], True)
                        if variant == "plain" and compiler == "gcc-14" and optimization == "0":
                            missing = subprocess.run([compiler, *map(str, objects), "-o", str(executable)],
                                                     capture_output=True, text=True, timeout=90)
                            assert missing.returncode != 0 and "fmod" in missing.stderr
                        run([compiler, *objects, *libraries, "-o", executable])
                        results.append(run([executable], data))
            for result in results:
                wanted = expectations[variant]
                if result.stdout != wanted:
                    actual, target = result.stdout.splitlines(), wanted.splitlines()
                    mismatch = next((i for i, pair in enumerate(zip(actual, target)) if pair[0] != pair[1]),
                                    min(len(actual), len(target)))
                    raise AssertionError((is_java, variant, mismatch, PAIRS[min(mismatch // 2, len(PAIRS) - 1)],
                                          actual[mismatch:mismatch + 1], target[mismatch:mismatch + 1]))
                assert result.stderr == traces(variant), (is_java, variant, "call order/multiplicity")
                if variant == "traced":
                    assert result.stderr == native_trace
    export(java_dir, c_dir, work_root)
    print(f"{len(PAIRS)} Rust cases; {len(truth.splitlines())} C/Java observations/run; three source crates; "
          "GCC/Zig O0/O2, strict Java21; six compiling value/trace faults")


if __name__ == "__main__":
    main()
