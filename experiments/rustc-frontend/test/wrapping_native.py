"""Real two-crate Rust/C/Java truth, strict compilers and native receiver traces."""
import os
from pathlib import Path
import subprocess
import sys

from wrapping_inventory import inspect
from wrapping_oracle import NAMES, cases, expected, traces
from wrapping_consumers import consumer, mutate
from short_circuit_mutations import instrument
from wrapping_examples import export


def run(command, inputs=None):
    result = subprocess.run([str(x) for x in command], input=inputs, capture_output=True,
                            text=True, timeout=120)
    assert result.returncode == 0, (command, result.stdout[:1000], result.stderr[:4000])
    return result


def main():
    java_dir, c_dir, reference, zig = [Path(p).resolve() for p in sys.argv[1:]]
    root, leaf, java, c, bindings, functions, native, marker_ids = inspect(java_dir, c_dir)
    rows = cases()
    inputs = "".join(f"{w} {value}\n" for w, value in rows)
    truth = expected(rows)
    assert run([reference], inputs).stdout == truth
    work_root = Path(os.environ["TEST_TMPDIR"]) / "wrapping-native"
    work_root.mkdir()
    runtimes = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime, = runtimes
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    for is_java, directory, owners in [(True, java_dir, java), (False, c_dir, c)]:
        def member(identity):
            if not is_java:
                return native[identity]["symbol"]
            path = functions[identity]["target"]["path"]
            return ".".join([path["package"], *path["owners"], path["member"]])

        names = {name + str(w): member(bindings[root, "value", name + str(w)][1])
                 for w in [32, 64] for name in NAMES}
        markers = {width: member(identity).split(".")[-1] for width, identity in marker_ids.items()}
        for variant in ["plain", "traced", "drop", "duplicate"]:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in [leaf, root]:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if owner == root and variant != "plain":
                    text = instrument(text, {"A": markers[32], "B": markers[64]}, is_java)
                    if variant in ["drop", "duplicate"]:
                        for width in [32, 64]:
                            text = mutate(text, names["method" + str(width)].split(".")[-1],
                                          markers[width], is_java, variant)
                source_dir = work / owner.split(":")[0]
                source_dir.mkdir()
                source = source_dir / ("Generated.java" if is_java else "generated.c")
                source.write_text(text, encoding="utf-8")
                sources.append(source)
            driver = work / ("Consumer.java" if is_java else "consumer.c")
            headers = "".join(f'#include "{c[owner]["header"]}"\n' for owner in [leaf, root])
            driver.write_text(consumer(names, is_java, headers), encoding="utf-8")
            if is_java:
                classes = work / "classes"
                classes.mkdir()
                flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
                for source in [*sources, driver]:
                    run([runtime / "javac", *flags, source])
                results = [run([runtime / "java", "-cp", classes, "Consumer"], inputs)]
            else:
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fno-fast-math",
                         "-ffp-contract=off", "-fsigned-char", "-fno-short-enums", "-I", c_dir]
                results = []
                configurations = [(compiler, optimization, []) for compiler in ["gcc-14", zig]
                                  for optimization in ["0", "2"]]
                configurations.append(("gcc-14", "2", ["-fsanitize=undefined", "-fno-sanitize-recover=all"]))
                for index, (compiler, optimization, extra) in enumerate(configurations):
                    objects = []
                    for i, source in enumerate([*sources, driver]):
                        obj = work / f"compiler{index}-{i}.o"
                        run([compiler, *flags, "-O" + optimization, *extra, "-c", source, "-o", obj])
                        objects.append(obj)
                    executable = work / f"compiler{index}"
                    run([compiler, *extra, *objects, "-o", executable])
                    results.append(run([executable], inputs))
            for result in results:
                assert result.stdout == truth, (is_java, variant, "arithmetic truth")
                assert result.stderr == traces(rows, variant), (is_java, variant, "receiver count")
                if variant in ["drop", "duplicate"]:
                    assert result.stderr != traces(rows, "traced")
    export(java_dir, c_dir, work_root)
    print(f"{len(rows)} boundary/random inputs, {len(rows) * len(NAMES)} Rust/C/Java results; "
          "GCC/Zig O0/O2 and UBSan; value-preserving dropped/duplicated receiver mutants detected")


if __name__ == "__main__":
    main()
