"""Actual Rust -> C/Java: independent modular values, measured traces and privacy."""
import os
from pathlib import Path
import subprocess
import sys
from addition_source_inventory import inspect
from addition_source_consumers import consumer
from addition_source_mutations import variants, mutate
from addition_source_privacy import java_privacy, c_privacy
from addition_source_examples import export
from short_circuit_mutations import instrument
OPERATION = sys.argv[6] if len(sys.argv) == 7 else "addition"
assert OPERATION in ["addition", "subtraction", "multiplication"] and len(sys.argv) in [6, 7]
if OPERATION == "subtraction":
    from wrapping_sub_oracle import CASES, inputs, result, faulty
elif OPERATION == "multiplication":
    from wrapping_mul_oracle import CASES, inputs, result, faulty
else:
    from wrapping_add_oracle import CASES, inputs, result, faulty
VARIANTS = variants(OPERATION)


def run(command, data=None):
    output = subprocess.run(list(map(str, command)), input=data, capture_output=True,
                            text=True, timeout=120)
    assert output.returncode == 0, (command, output.stdout[:1000], output.stderr[:5000])
    return output


def expected(variant="plain", copies=2):
    def value(width, left, right):
        if variant == "wrong_operand":
            return result(left, left, width)
        if variant in ["carryless", "narrow", "add", "reverse_values"]:
            fault = "reverse" if variant == "reverse_values" else variant
            if OPERATION == "multiplication" and fault == "narrow":
                fault = "narrow_result"
            return faulty(left, right, width, fault)
        return result(left, right, width)
    return "".join(f"{value(w, a, b)}\n" for w, a, b in CASES for _ in range(copies))


def traces(variant="traced", copies=2):
    if variant == "plain":
        return ""
    chunks = []
    for width, _, _ in CASES:
        left, right = "AB" if width == 32 else "CD"
        chunk = {"dropped": right, "duplicated": left + left + right,
                 "reversed": right + left}.get(variant, left + right)
        chunks.append(chunk * copies)
    return "".join(chunks)


def main():
    java_dir, c_dir, reference, traced_reference, zig = [Path(arg).resolve() for arg in sys.argv[1:6]]
    order, module, java, c, bindings, functions, native, private = inspect(java_dir, c_dir, OPERATION)
    data, truth = inputs(), expected()
    rust = run([reference], data)
    assert rust.stdout == expected(copies=1) and rust.stderr == ""
    traced = run([traced_reference], data)
    assert traced.stdout == rust.stdout and traced.stderr == traces(copies=1)
    # Replicate each measured input's two original calls at the two target entry points.
    measured = "".join(traced.stderr[index:index + 2] * 2 for index in range(0, len(traced.stderr), 2))
    assert measured == traces()
    expectations = {variant: expected(variant) for variant in VARIANTS}
    value_faults = (["carryless", "wrong_operand", "narrow"] if OPERATION == "addition" else
                    ["add", "wrong_operand", "narrow"] if OPERATION == "multiplication" else
                    ["add", "reverse_values", "wrong_operand", "narrow"])
    for variant in value_faults:
        assert expectations[variant] != truth
        for width in [32, 64]:
            indices = [index for index, case in enumerate(CASES) if case[0] == width]
            actual = expectations[variant].splitlines()
            correct = truth.splitlines()
            assert any(actual[2 * index] != correct[2 * index] for index in indices)
    for variant in ["dropped", "duplicated", "reversed"]:
        assert expectations[variant] == truth and traces(variant) != measured
    work_root = Path(os.environ["TEST_TMPDIR"]) / (OPERATION + "-source")
    work_root.mkdir()
    runtimes = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    runtime, = runtimes
    assert run([runtime / "javac", "-version"]).stdout.startswith("javac 21.")
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    for is_java, directory, owners in [(True, java_dir, java), (False, c_dir, c)]:
        def member(owner, name):
            identity = bindings[owner, "value", name][1]
            if not is_java:
                return native[identity]["symbol"]
            path = functions[identity]["target"]["path"]
            return ".".join([path["package"], *path["owners"], path["member"]])
        calls = {width: [member(owner, OPERATION + str(width)) for owner in [module, order[-1]]]
                 for width in [32, 64]}
        markers = {marker: member(order[0], side + str(width)).split(".")[-1]
                   for width, labels in [(32, "AB"), (64, "CD")]
                   for side, marker in zip(["left", "right"], labels, strict=True)}
        for variant in VARIANTS:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in order:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if owner == order[0] and variant != "plain":
                    text = instrument(text, markers, is_java)
                if owner == order[1] and variant not in ["plain", "traced"]:
                    for width in [32, 64]:
                        text = mutate(text, member(module, OPERATION + str(width)).split(".")[-1],
                                      member(order[0], "left" + str(width)), member(order[0], "right" + str(width)),
                                      width, is_java, variant, OPERATION)
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
                results = [run([runtime / "java", "-cp", classes, "Consumer"], data)]
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
                assert output.stdout == expectations[variant], (is_java, variant, "exact signed modular values")
                assert output.stderr == traces(variant), (is_java, variant, "ordered original operand evaluation")
                if variant == "traced":
                    assert output.stderr == measured
    export(java_dir, c_dir, work_root, OPERATION)
    print(f"{len(CASES)} Rust cases; {len(CASES) * 2} target observations/run; three source crates; "
          f"GCC/Zig O0/O2, UBSan and strict Java21; {len(value_faults) + 3} compiling value/trace faults")


if __name__ == "__main__":
    main()
