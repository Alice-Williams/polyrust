"""Original three-crate source against independent truth, native traces and faults."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
from widening_source_inventory import inspect
from widening_source_consumers import consumer
from widening_source_mutations import VARIANTS, mutate
from widening_oracle import CASES, inputs, result, faulty
from addition_source_privacy import java_privacy, c_privacy
from short_circuit_mutations import instrument
from constant_export_scratch import writable_copy


def run(command, data=None):
    output = subprocess.run(list(map(str, command)), input=data, capture_output=True, text=True, timeout=120)
    assert output.returncode == 0, (command, output.stdout[:1000], output.stderr[:5000])
    return output


def expected(variant="plain", copies=2):
    return "".join(f"{faulty(v, variant) if variant in ['zero_extend', 'narrow', 'zero'] else result(v)}\n"
                   for v in CASES for _ in range(copies))


def traces(variant="traced", copies=2):
    count = {"plain": 0, "dropped": 0, "duplicated": 2}.get(variant, 1)
    return "A" * (count * copies * len(CASES))


def contents(directory):
    return {str(p.relative_to(directory)): p.read_bytes() for p in directory.rglob("*") if p.is_file()}


def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "signed-widening"
    destination.mkdir()
    writable_copy(java, destination / "java")
    writable_copy(c, destination / "c")
    clients = destination / "clients"
    clients.mkdir()
    for parent, filename in [("java-plain", "Consumer.java"), ("c-plain", "consumer.c")]:
        shutil.copy2(work / parent / filename, clients / filename)
    sources = destination / "rust-source"
    sources.mkdir()
    fixtures = Path(__file__).resolve().parent.parent / "fixtures"
    for name in ["widening_leaf.rs", "widening_middle.rs", "widening_root.rs", "reference_widening_source.rs"]:
        shutil.copy2(fixtures / name, sources / name)
    (destination / "README.md").write_text(
        "# Checked Rust signed widening\n\nActual three-crate C and Java packages, original Rust, and handwritten external clients. "
        "No custom runtime. C uses int32_t-to-int64_t conversion; Java uses int-to-long cast.\n\n"
        f"Proof: //experiments/rustc-frontend:widening_native_test. {len(CASES):,} native Rust and independent integer inputs; "
        f"{2 * len(CASES):,} target observations/run. GCC14/Zig O0/O2, GCC UBSan, strict Java21 normal and -Xint. "
        "Measured native/target operand traces; three compiling value faults and two value-preserving call faults, "
        "plus exact API/docs/import inventories and external privacy controls.\n")


def main():
    java_dir, c_dir, reference, traced_reference, zig = [Path(arg).resolve() for arg in sys.argv[1:]]
    originals = [contents(java_dir), contents(c_dir)]
    order, module, java, c, bindings, functions, native, private = inspect(java_dir, c_dir)
    data, truth = inputs(), expected()
    rust = run([reference], data)
    assert rust.stdout == expected(copies=1) and rust.stderr == ""
    traced = run([traced_reference], data)
    assert traced.stdout == rust.stdout and traced.stderr == traces(copies=1)
    measured = "".join(marker * 2 for marker in traced.stderr)
    assert measured == traces()
    expectations = {variant: expected(variant) for variant in VARIANTS}
    for variant in ["zero_extend", "narrow", "zero"]:
        assert expectations[variant] != truth
    for variant in ["dropped", "duplicated"]:
        assert expectations[variant] == truth and traces(variant) != measured
    work_root = Path(os.environ["TEST_TMPDIR"]) / "widening-source"
    work_root.mkdir()
    runtime, = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert run([runtime / "javac", "-version"]).stdout.startswith("javac 21.")
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    for is_java, directory, owners in [(True, java_dir, java), (False, c_dir, c)]:
        def member(owner, name):
            identity = bindings[owner, "value", name][1]
            if not is_java:
                return native[identity]["symbol"]
            path = functions[identity]["target"]["path"]
            return ".".join([path["package"], *path["owners"], path["member"]])
        calls = [member(owner, "widen") for owner in [module, order[-1]]]
        markers = {"A": member(order[0], "input").split(".")[-1]}
        for variant in VARIANTS:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in order:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if owner == order[0] and variant != "plain":
                    text = instrument(text, markers, is_java)
                if owner == order[1] and variant not in ["plain", "traced"]:
                    text = mutate(text, member(module, "widen").split(".")[-1],
                                  member(order[0], "input"), is_java, variant)
                location = work / owner.split(":")[0]
                location.mkdir()
                source = location / ("Generated.java" if is_java else "generated.c")
                source.write_text(text, encoding="utf-8")
                sources.append(source)
            driver = work / ("Consumer.java" if is_java else "consumer.c")
            headers = "".join(f'#include "{c[owner]["header"]}"\n' for owner in order)
            driver.write_text(consumer(calls, is_java, headers))
            if is_java:
                classes = work / "classes"
                classes.mkdir()
                flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
                for source in [*sources, driver]:
                    run([runtime / "javac", *flags, source])
                results = [run([runtime / "java", *options, "-cp", classes, "Consumer"], data)
                           for options in [[], ["-Xint"]]]
                if variant == "plain":
                    for identity in sorted(private):
                        java_privacy(run, work, runtime, classes, sources[0], functions[identity])
            else:
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fno-fast-math",
                         "-ffp-contract=off", "-fsigned-char", "-fno-short-enums", "-I", c_dir]
                configurations = [(compiler, opt, []) for compiler in ["gcc-14", zig] for opt in ["0", "2"]]
                configurations.append(("gcc-14", "2", ["-fsanitize=undefined", "-fno-sanitize-recover=all"]))
                results = []
                for config, (compiler, opt, extra) in enumerate(configurations):
                    objects = []
                    for index, source in enumerate([*sources, driver]):
                        obj = work / f"{config}-{index}.o"
                        run([compiler, *flags, "-O" + opt, *extra, "-c", source, "-o", obj])
                        objects.append(obj)
                    executable = work / f"compiler{config}"
                    run([compiler, *extra, *objects, "-o", executable])
                    results.append(run([executable], data))
                    if variant == "plain" and opt == "0":
                        for owner in order:
                            header = work / ("header-" + owner.split(":")[0] + ".c")
                            header.write_text(f'#include "{c[owner]["header"]}"\nint main(void) {{ return 0; }}\n')
                            run([compiler, *flags, "-c", header, "-o", work / "header.o"])
                        for identity in sorted(private):
                            c_privacy(run, work, compiler, flags, sources[0], objects[0], native[identity],
                                      int(functions[identity]["result"][1:]), str(config))
            for output in results:
                assert output.stdout == expectations[variant], (is_java, variant, "exact signed widening values")
                assert output.stderr == traces(variant), (is_java, variant, "ordered original operand evaluation")
                if variant == "traced":
                    assert output.stderr == measured
    assert originals == [contents(java_dir), contents(c_dir)]
    export(java_dir, c_dir, work_root)
    print(f"{len(CASES)} native source inputs; {2*len(CASES)} target observations/run; "
          "five compiling value/evaluation faults; exact three-owner API and external privacy")


if __name__ == "__main__":
    main()
