"""Independent native consumers of ordinary public constant packages."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from public_constant_entry import check as check_entry

NAMES = ["FALSE", "TRUE", "I32_MIN", "I32_MAX", "I64_MIN", "I64_MAX",
         "WIDE", "NEGATIVE_WIDE", "COMPUTED", "FORWARD", "WIDE_ALIAS"]
VALUES = [0, 1, -2147483648, 2147483647, -9223372036854775808,
          9223372036854775807, 9007199254740993, -9007199254740993, 62,
          9007199254740993, 9007199254740993]
FUNCTIONS = ["read_false", "read_true", "read_i32_min", "read_i32_max",
             "read_i64_min", "read_i64_max", "read_wide", "read_negative_wide",
             "read_computed", "read_forward", "read_private"]


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], capture_output=True,
                            text=True, timeout=120)
    assert (result.returncode == 0) == success, (command, result.stdout, result.stderr)
    return result


def contents(path):
    return {p.name: p.read_bytes() for p in path.iterdir() if p.is_file()}


def driver(names, methods, java):
    expressions = [names[name] for name in NAMES]
    expressions += [methods[name] + "()" for name in FUNCTIONS] if methods else []
    if java:
        lines = [f"System.out.println({value}" +
                 (" ? 1 : 0" if i in [0, 1, 11, 12] else "") + ");"
                 for i, value in enumerate(expressions)]
        return "public class Consumer { public static void main(String[] a) {\n" + "\n".join(lines) + "\n}}"
    lines = [f'printf("%lld\\n", (long long)({value}));' for value in expressions]
    return '#include <stdio.h>\n#include "api.h"\nint main(void) {\n' + "\n".join(lines) + "\nreturn 0; }"


def main():
    c_adapter, c_probe, java_adapter, java_probe, data, mixed, reference, zig, entry, entry_reference = [
        Path(p).resolve() for p in sys.argv[1:]]
    work = Path(os.environ["TEST_TMPDIR"]) / "public-constant-native"
    work.mkdir()
    truth = VALUES + VALUES[:10] + [17]
    assert run([reference]).stdout.splitlines() == [str(x) for x in truth]
    jdk = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
           if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(jdk) == 1
    check_entry(work / "entry", entry, entry_reference, c_adapter, c_probe,
                java_adapter, java_probe, jdk[0], zig, run)
    for label, source in [("only", data), ("mixed", mixed)]:
        case = work / label
        case.mkdir()
        declared = ["--input", data] if label == "mixed" else []
        c_dir = case / "c"
        run([c_adapter, source, c_dir, "--package", *declared])
        c_probe_dir = case / "c-probe"
        c_observed = run([c_probe, source, c_probe_dir, "--package", *declared]).stdout
        assert contents(c_dir) == contents(c_probe_dir)
        java_dir = case / "java"
        java_dir.mkdir()
        java_source = java_dir / "Generated.java"
        run([java_adapter, source, java_source, "--package", *declared])
        probe_dir = case / "java-probe"
        probe_dir.mkdir()
        observed = run([java_probe, source, probe_dir / "Generated.java", "--package", *declared]).stdout
        assert java_source.read_bytes() == (probe_dir / "Generated.java").read_bytes()
        for output in [c_observed, observed]:
            assert output.count("PUBLIC_CONSTANT_DECL\t") == 10
            assert output.count("PUBLIC_CONSTANT_READ\t") == (10 if label == "mixed" else 0)
        assert "PUBLIC_CONSTANT_CERTIFIED\t10" in observed
        assert "PUBLIC_CONSTANT_MANIFEST_MUTATIONS\t7" in c_observed
        api = json.loads((c_dir / "api.json").read_text())
        assert api["schema_version"] == 3 and len(api["constants"]) == 10
        assert set(contents(c_dir)) == {"api.json", api["header"], api["implementation"]}
        c_bindings = {b["name"]: b["target"] for m in api["modules"] for b in m["bindings"]
                      if m["id"] == api["root"]}
        java_bindings = {parts[2]: parts[3] for line in observed.splitlines()
                         if (parts := line.split("\t"))[0] == "BIND" and parts[1] == api["root"]}
        assert java_bindings == c_bindings
        expected_names = set(NAMES + (FUNCTIONS if label == "mixed" else []))
        assert set(c_bindings) == expected_names
        constants = {item["id"]: item for item in api["constants"]}
        assert c_bindings["WIDE"] == c_bindings["WIDE_ALIAS"]
        for name, expected in zip(NAMES, VALUES, strict=True):
            item = constants[c_bindings[name]]
            assert item["readonly"] is True
            assert int(item["value"]) == expected
            assert item["type"] == ("bool" if name in ["FALSE", "TRUE"] else
                                    "i32" if name in ["I32_MIN", "I32_MAX", "COMPUTED"] else "i64")
            if item["type"] != "bool":
                assert isinstance(item["value"], str)
        c_symbols = {item["id"]: item["symbol"] for item in [*api["constants"], *api["functions"]]}
        java_symbols = {p[2]: p[3] for line in observed.splitlines()
                        if (p := line.split("\t"))[0] == "TARGET"}
        assert set(c_symbols) == set(java_symbols)
        c_text = (c_dir / api["implementation"]).read_text()
        java_text = java_source.read_text()
        for text in [(c_dir / api["header"]).read_text(), java_text]:
            assert "False constant documentation." in text
            assert "True constant documentation." in text
        assert "runtime" not in c_text.lower() and "Runtime" not in java_text
        if label == "only":
            assert "static " not in c_text and "score(" not in java_text
        expected = truth if label == "mixed" else VALUES
        for java, symbols, original in [(False, c_symbols, c_text), (True, java_symbols, java_text)]:
            names = {n: symbols[c_bindings[n]] for n in NAMES}
            methods = {n: symbols[c_bindings[n]] for n in FUNCTIONS} if label == "mixed" else {}
            for mutant in [False, True]:
                trial = case / (("java" if java else "c") + ("-mutant" if mutant else "-native"))
                trial.mkdir()
                changed, count = re.subn(r"(?<![\w])62(?![\w])", "17", original)
                assert count == 1
                generated = trial / ("Generated.java" if java else "model.c")
                generated.write_text(changed if mutant else original)
                consumer = trial / ("Consumer.java" if java else "consumer.c")
                consumer.write_text(driver(names, methods, java) + "\n")
                results = []
                if java:
                    classes = trial / "classes"
                    classes.mkdir()
                    flags = ["--release", "21", "-Xlint:all", "-Werror", "-implicit:none",
                             "-sourcepath", "", "-cp", classes, "-d", classes]
                    run([jdk[0] / "javac", *flags, generated])
                    run([jdk[0] / "javac", *flags, consumer])
                    results.append(run([jdk[0] / "java", "-cp", classes, "Consumer"]).stdout)
                    if not mutant:
                        readonly(trial, java, names, jdk[0] / "javac", flags)
                else:
                    (trial / "api.h").write_bytes((c_dir / api["header"]).read_bytes())
                    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                             "-Wstrict-prototypes", "-Wmissing-prototypes", "-I", c_dir, "-I", trial]
                    for compiler in ["gcc-14", zig]:
                        for opt in ["0", "2"]:
                            objects = []
                            for i, src in enumerate([generated, consumer]):
                                obj = trial / f"object{i}.o"
                                run([compiler, *flags, "-O" + opt, "-c", src, "-o", obj])
                                objects.append(obj)
                            binary = trial / "consumer"
                            run([compiler, *objects, "-o", binary])
                            results.append(run([binary]).stdout)
                        if not mutant:
                            readonly(trial, java, names, compiler, flags)
                wanted = [17 if mutant and x == 62 else x for x in expected]
                for result in results:
                    assert result.splitlines() == [str(x) for x in wanted]
                    assert (result.splitlines() == [str(x) for x in expected]) != mutant
    print("Public constants: exact Rust/C/Java native truth, GCC/Zig O0/O2, writes rejected, value mutants detected")


def readonly(work, java, names, compiler, flags):
    # Compile one attempted write per unique object; alias is the same identity.
    names = list(dict.fromkeys(names.values()))
    assignments = "\n".join(f"{name} = {name};" for name in names)
    source = work / ("Writes.java" if java else "writes.c")
    source.write_text(("class Writes { void writes() {\n" + assignments + "\n}}") if java else
                      '#include "api.h"\nint main(void) {\n' + assignments + "\nreturn 0; }\n")
    result = run([compiler, *flags, source] if java else
                 [compiler, *flags, "-c", source, "-o", work / "writes.o"], success=False)
    assert "final variable" in result.stderr if java else (
        "read-only" in result.stderr or "const-qualified" in result.stderr or "cannot assign" in result.stderr)
    assert all(name.split(".")[-1] in result.stderr for name in names)


if __name__ == "__main__":
    main()
