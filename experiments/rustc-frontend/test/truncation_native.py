"""Three original source owners; exact Rust/C/Java truncation/category and call proof."""
import os
from pathlib import Path
import subprocess
import sys
from truncation_inventory import inspect
from truncation_bundle import link_options, malformed_controls
from truncation_oracle import VALUES, OPERATIONS, LITERALS, expected, traces, observed, rounding
from floating_consumers import c_consumer, java_consumer
from truncation_mutations import mutate
from short_circuit_mutations import instrument
from truncation_examples import export


def run(command, inputs=None):
    result = subprocess.run([str(arg) for arg in command], input=inputs,
                            capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result


def main():
    java_dir, c_dir, reference, zig = [Path(arg).resolve() for arg in sys.argv[1:]]
    order, java, c, bindings, functions, native, marker_id = inspect(java_dir, c_dir)
    root = order[-1]
    for api in c.values():
        malformed_controls(api)
    truth = expected()
    assert run([reference], "".join(f"{bits:016x}\n" for bits in VALUES)).stdout == truth
    work_root = Path(os.environ["TEST_TMPDIR"]) / "truncation-native"
    work_root.mkdir()
    runtimes = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime = runtimes[0]
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    for is_java, directory, owners in [(True, java_dir, java), (False, c_dir, c)]:
        def member(identity):
            if not is_java:
                return native[identity]["symbol"]
            path = functions[identity]["target"]["path"]
            return ".".join([path["package"], *path["owners"], path["member"]])

        def named(name):
            return member(bindings[root, "value", name][1])

        literals = [named(name) + "()" for name in LITERALS]
        calls = [named(name) + "(a)" for name in OPERATIONS]
        marker = member(marker_id).split(".")[-1]
        ordinary = member(bindings[order[0], "value", "trunc"][1])
        relay = member(bindings[order[1], "value", "relay"][1]).split(".")[-1]
        for variant in ["plain", "traced", "zero_sign", "floor", "ceil", "dropped", "duplicated", "ordinary_builtin", "ordinary_duplicated"]:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in order:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if owner == order[0] and variant != "plain":
                    text = instrument(text, {"B": ordinary.split(".")[-1]}, is_java)
                if owner == order[1] and variant in ["ordinary_builtin", "ordinary_duplicated"]:
                    fault = "dropped" if variant == "ordinary_builtin" else "duplicated"
                    text = mutate(text, relay, ordinary, is_java, fault)
                    if not is_java and variant == "ordinary_builtin":
                        # The test-only substituted call introduces its own dependency.
                        # The original middle owner correctly needs no math header.
                        assert "#include <math.h>" not in text
                        text = "#include <math.h>\n" + text
                if owner == root and variant != "plain":
                    text = instrument(text, {"A": marker}, is_java)
                    if variant not in ["traced", "ordinary_builtin", "ordinary_duplicated"]:
                        target = "direct" if variant in ["zero_sign", "floor", "ceil"] else "local"
                        text = mutate(text, named(target).split(".")[-1], marker, is_java, variant)
                location = work / owner.split(":")[0]
                location.mkdir()
                source = location / ("Generated.java" if is_java else "generated.c")
                source.write_text(text, encoding="utf-8")
                sources.append(source)
            if is_java:
                consumer = work / "Consumer.java"
                consumer.write_text(java_consumer(literals, calls, VALUES))
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
                consumer.write_text(c_consumer(literals, calls, headers, VALUES))
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
                        if compiler == "gcc-14" and optimization == "0" and variant == "plain":
                            missing = subprocess.run([compiler, *objects, "-o", str(executable)],
                                                     capture_output=True, text=True, timeout=90)
                            assert missing.returncode != 0 and "trunc" in missing.stderr
                        run([compiler, *objects, *link_options(c[root]), "-o", executable])
                        results.append(run([executable]))
            for result in results:
                wanted = truth.splitlines()
                if variant in ["zero_sign", "floor", "ceil"]:
                    original = list(wanted)
                    for index, bits in enumerate(VALUES):
                        wanted[2 + index * len(OPERATIONS)] = observed(rounding(bits, variant))
                    assert wanted != original
                assert result.stdout.splitlines() == wanted, (is_java, variant, "exact truncation oracle")
                assert result.stderr == traces(variant), (is_java, variant, "single operand call")
                if variant in ["dropped", "duplicated", "ordinary_builtin", "ordinary_duplicated"]:
                    assert result.stdout == truth and result.stderr != traces("traced")
    export(java_dir, c_dir, work_root)
    print(f"{len(truth.splitlines())} exact Rust/C/Java integral/category results; three crates, "
          "GCC/Zig O0/O2, Java21 strict lint; seven value/trace faults detected")


if __name__ == "__main__":
    main()
