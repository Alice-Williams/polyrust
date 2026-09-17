"""Separate Rust/C/Java compilation and exact observable binary64 equivalence."""
import os
from pathlib import Path
import subprocess
import sys
from binary64_inventory import inspect
from binary64_examples import export
from binary64_oracle import VALUES, LITERALS, TRANSPORT, COMPARISONS, OPERATORS, expected, traces
from binary64_consumers import c_consumer, java_consumer
from binary64_mutations import fault_comparison, fault_literal
from short_circuit_mutations import instrument


def run(command, inputs=None):
    result = subprocess.run([str(arg) for arg in command], input=inputs,
                            capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result


def main():
    java_dir, c_dir, reference, zig = [Path(arg).resolve() for arg in sys.argv[1:]]
    order, java, c, bindings, functions, c_functions, marker_ids = inspect(java_dir, c_dir)
    leaf, _, root = order
    truth = expected()
    assert run([reference], "".join(f"{bits:016x}\n" for bits in VALUES)).stdout == truth
    work_root = Path(os.environ["TEST_TMPDIR"]) / "binary64-native"
    work_root.mkdir()
    runtimes = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime = runtimes[0]
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    for is_java, directory, owners in [(True, java_dir, java), (False, c_dir, c)]:
        def member(identity):
            if not is_java:
                return c_functions[identity]["symbol"]
            path = functions[identity]["target"]["path"]
            return ".".join([path["package"], *path["owners"], path["member"]])

        def call(owner, name, arguments):
            return member(bindings[owner, "value", name][1]) + "(" + arguments + ")"

        calls = ([call(leaf, name, "") for name in LITERALS] + [call(root, "tenth", "")],
                 [call(root, name, "a") for name in TRANSPORT],
                 [call(root, name, "a,b") for name in COMPARISONS])
        markers = {key: member(identity).split(".")[-1] for key, identity in marker_ids.items()}
        for variant in ["plain", "traced", "reordered", "dropped", "zero", "subnormal"]:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in order:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if owner == root and variant != "plain":
                    text = instrument(text, markers, is_java)
                    if variant in ["reordered", "dropped"]:
                        pairs = list(zip(COMPARISONS, OPERATORS, strict=True))
                        for name, operator in pairs if variant == "reordered" else pairs[:1]:
                            identity = bindings[root, "value", name][1]
                            text = fault_comparison(text, member(identity).split(".")[-1], markers, operator, variant)
                if owner == leaf and variant in ["zero", "subnormal"]:
                    name = "negative_zero" if variant == "zero" else "minimum_subnormal"
                    text = fault_literal(text, member(bindings[leaf, "value", name][1]).split(".")[-1])
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
                results = [run([runtime / "java", "-cp", classes, "Consumer"])]
            else:
                consumer = work / "consumer.c"
                headers = "".join(f'#include "{c[owner]["header"]}"\n' for owner in order)
                consumer.write_text(c_consumer(calls, headers))
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes",
                         "-Wmissing-prototypes", "-fno-fast-math", "-ffp-contract=off",
                         "-fsigned-char", "-fno-short-enums", "-I", c_dir]
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
                        run([compiler, *objects, "-o", executable])
                        results.append(run([executable]))
            for result in results:
                actual, wanted = result.stdout.splitlines(), truth.splitlines()
                assert len(actual) == len(wanted)
                if variant in ["zero", "subnormal"]:
                    position = 1 if variant == "zero" else 2
                    assert [i for i, pair in enumerate(zip(actual, wanted, strict=True)) if pair[0] != pair[1]] == [position]
                else:
                    assert result.stdout == truth, (is_java, variant, "exact value/comparison oracle")
                trace = traces()
                if variant == "plain":
                    trace = "\n" * len(wanted)
                elif variant == "reordered":
                    trace = traces("RL")
                elif variant == "dropped":
                    prefix = len(LITERALS) + 1 + len(VALUES) * len(TRANSPORT)
                    trace = "\n" * prefix + ("R\n" + "LR\n" * 5) * len(VALUES) ** 2
                assert result.stderr == trace, (is_java, variant, "exactly-once left-to-right trace")
    export(java_dir, c_dir, work_root)
    print(f"{len(truth.splitlines())} exact Rust/C/Java results across three crates; "
          "GCC/Zig O0/O2 and strict Java21; signed-zero/subnormal/order/count faults detected")


if __name__ == "__main__":
    main()
