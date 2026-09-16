"""Cross-crate references, separate native compilation, and producer mutation."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys

FUNCTIONS = ["false", "true", "i32_min", "i32_max", "i64_min", "i64_max",
             "wide", "negative_wide", "computed", "forward", "private",
             "local", "left", "right"]
TRUTH = [0, 1, -2147483648, 2147483647, -9223372036854775808,
         9223372036854775807, 9007199254740993, -9007199254740993,
         62, 9007199254740993, 17, 29, 9007199254740993, 9007199254740993]


def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True,
                            text=True, timeout=120)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result.stdout


def manifests(directory):
    index = json.loads((directory / "bundle.json").read_text())
    owners = {m["root"]: json.loads((directory / m["manifest"]).read_text())
              for m in index["members"]}
    assert index["schema_version"] == 1 and len(owners) == 4
    return index["root"], owners


def inventories(c_dir, java_dir):
    root, c = manifests(c_dir)
    jroot, java = manifests(java_dir)
    assert root == jroot and set(c) == set(java)
    assert len(list(c_dir.rglob("*.c"))) == len(list(c_dir.rglob("*.h"))) == 4
    assert len(list(java_dir.rglob("Generated.java"))) == 4
    assert len([p for p in c_dir.rglob("*") if p.is_file()]) == 13
    assert len([p for p in java_dir.rglob("*") if p.is_file()]) == 9
    producers = [oid for oid, m in c.items() if not m["functions"]]
    assert len(producers) == 1
    producer = producers[0]
    constants = {item["id"]: item for item in c[producer]["constants"]}
    assert len(constants) == 10
    jconstants = {d["id"]: d for d in java[producer]["declarations"]}
    assert set(constants) == set(jconstants)
    for oid in c:
        cc, jj = c[oid], java[oid]
        ci = {d["id"]: d for d in cc.get("constant_imports", [])}
        ji = {d["id"]: d for d in jj.get("constant_imports", [])}
        assert set(ci) == set(ji)
        assert len(ci) == (0 if oid == producer else 10 if oid == root else 1)
        assert cc["schema_version"] == (4 if oid == producer else 5)
        assert jj["schema_version"] == (2 if oid == producer else 3)
        assert len(cc["functions"]) == (0 if oid == producer else len(FUNCTIONS) if oid == root else 1)
        if oid == root:
            assert len(cc["constants"]) == 1 and len(cc["imports"]) == 2
        elif oid != producer:
            assert len(cc["constants"]) == 1 and not cc["imports"]
        for cid, imported in ci.items():
            original = constants[cid]
            assert imported["owner"] == ji[cid]["owner"] == producer
            assert imported["header"] == c[producer]["header"]
            assert imported["symbol"] == original["symbol"]
            assert imported["value"] == ji[cid]["value"] == original["value"]
            assert imported["type"] == ji[cid]["scalar"] == original["type"]
            assert imported["readonly"] is ji[cid]["readonly"] is True
            assert ji[cid]["path"] == jconstants[cid]["target"]["path"]
            assert isinstance(imported["value"], bool if imported["type"] == "bool" else str)
        expected_dependencies = set(c) - {root} if oid == root else ({producer} if ci else set())
        assert set(jj["dependencies"]) == expected_dependencies
    cbindings = {b["name"]: b["target"] for m in c[root]["modules"]
                 if m["id"] == root for b in m["bindings"]}
    jbindings = {b["name"]: b["target"]["id"] for m in java[root]["modules"]
                 if m["id"] == root for b in m["bindings"]}
    assert cbindings == jbindings
    assert set(cbindings) == {"LOCAL", *["read_" + name for name in FUNCTIONS]}
    cnames = {f["id"]: f["symbol"] for f in c[root]["functions"]}
    jnames = {d["id"]: ".".join([d["target"]["path"]["package"],
              *d["target"]["path"]["owners"], d["target"]["path"]["member"]])
              for d in java[root]["declarations"]}
    return root, producer, c, java, cbindings, cnames, jnames


def main():
    c_dir, java_dir, reference, zig = [Path(p).resolve() for p in sys.argv[1:]]
    assert run([reference]).splitlines() == list(map(str, TRUTH))
    root, producer, c, java, bindings, cnames, jnames = inventories(c_dir, java_dir)
    jdk = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
           if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(jdk) == 1
    work = Path(os.environ["TEST_TMPDIR"]) / "foreign-constants"
    work.mkdir()
    for target, original, names in [("c", c_dir, cnames), ("java", java_dir, jnames)]:
        for mutant in [False, True]:
            trial = work / f"{target}-{mutant}"
            trial.mkdir()
            # Compile from a private copy; never mutate Bazel-generated artifacts.
            for source in original.rglob("*"):
                if source.is_file():
                    output = trial / source.relative_to(original)
                    output.parent.mkdir(parents=True, exist_ok=True)
                    output.write_bytes(source.read_bytes())
            source_name = java[producer]["source"] if target == "java" else c[producer]["implementation"]
            source = trial / source_name
            contents = source.read_text()
            assert "runtime" not in contents.lower()
            changed, count = re.subn(r"(?<![\w])62(?![\w])", "17", contents)
            assert count == 1
            if mutant:
                source.write_text(changed)
            expressions = [names[bindings["read_" + name]] + "()" for name in FUNCTIONS]
            # Repeat every call to exercise stable imports and multiple body transitions.
            expressions *= 2
            outputs = []
            if target == "java":
                body = "\n".join("System.out.println(" + expression +
                    (" ? 1 : 0" if i % len(FUNCTIONS) < 2 else "") + ");"
                    for i, expression in enumerate(expressions))
                consumer = trial / "Consumer.java"
                consumer.write_text("public class Consumer { public static void main(String[] a) {\n" + body + "\n}}")
                classes = trial / "classes"
                classes.mkdir()
                flags = ["--release", "21", "-Xlint:all", "-Werror", "-implicit:none",
                         "-sourcepath", "", "-cp", classes, "-d", classes]
                # Producer, intermediate owners, then root; recompile against each mutant.
                order = [producer, *sorted(set(c) - {producer, root}), root]
                for oid in order:
                    run([jdk[0] / "javac", *flags, trial / java[oid]["source"]])
                run([jdk[0] / "javac", *flags, consumer])
                outputs.append(run([jdk[0] / "java", "-cp", classes, "Consumer"]))
            else:
                consumer = trial / "consumer.c"
                body = "\n".join(f'printf("%lld\\n", (long long)({e}));' for e in expressions)
                consumer.write_text('#include <stdio.h>\n#include "' + c[root]["header"] +
                                    '"\nint main(void) {\n' + body + "\nreturn 0; }\n")
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                         "-Wstrict-prototypes", "-Wmissing-prototypes", "-I", trial]
                for compiler in ["gcc-14", zig]:
                    for opt in ["0", "2"]:
                        objects = []
                        for i, unit in enumerate([trial / c[oid]["implementation"] for oid in sorted(c)] + [consumer]):
                            obj = trial / f"unit{i}.o"
                            run([compiler, *flags, "-O" + opt, "-c", unit, "-o", obj])
                            objects.append(obj)
                        executable = trial / "consumer"
                        run([compiler, *objects, "-o", executable])
                        outputs.append(run([executable]))
            expected = [17 if mutant and n == 62 else n for n in TRUTH] * 2
            for output in outputs:
                assert output.splitlines() == list(map(str, expected))
                assert (output.splitlines() == list(map(str, TRUTH * 2))) != mutant
    print("Four-crate constants/function diamond: Rust, GCC/Zig O0/O2, Java21, exact imports and producer mutation")


if __name__ == "__main__":
    main()
