"""Exact two-owner bundle schemas and separately compiled native consumers."""
import json
import os
from pathlib import Path
import re
import sys

from public_constant_native import FUNCTIONS, NAMES, VALUES, driver, readonly, run


def inventory(c_dir, java_dir):
    c_index = json.loads((c_dir / "bundle.json").read_text())
    java_index = json.loads((java_dir / "bundle.json").read_text())
    assert c_index["schema_version"] == java_index["schema_version"] == 1
    assert c_index["root"] == java_index["root"]
    c = {m["root"]: json.loads((c_dir / m["manifest"]).read_text()) for m in c_index["members"]}
    java = {m["root"]: json.loads((java_dir / m["manifest"]).read_text()) for m in java_index["members"]}
    assert set(c) == set(java) and len(c) == 2
    assert {p.relative_to(c_dir).as_posix() for p in c_dir.rglob("*") if p.is_file()} == {
        "bundle.json", *[p for m in c_index["members"] for p in [
            m["manifest"], c[m["root"]]["header"], c[m["root"]]["implementation"]]]}
    assert {p.relative_to(java_dir).as_posix() for p in java_dir.rglob("*") if p.is_file()} == {
        "bundle.json", *[p for m in java_index["members"] for p in [m["manifest"], m["source"]]]}
    for identity in c:
        cc, jj = c[identity], java[identity]
        assert cc["schema_version"] == 4 and jj["schema_version"] == 2
        assert cc["root"] == jj["root"] == identity
        assert not cc["imports"] and not jj["dependencies"]
        c_bindings = {b["name"]: b["target"] for m in cc["modules"] if m["id"] == identity for b in m["bindings"]}
        j_bindings = {b["name"]: b["target"]["id"] for m in jj["modules"] if m["id"] == identity for b in m["bindings"]}
        assert c_bindings == j_bindings
        mixed = identity == c_index["root"]
        assert set(c_bindings) == set(NAMES + (FUNCTIONS if mixed else []))
        constants = {item["id"]: item for item in cc["constants"]}
        declarations = {item["id"]: item for item in jj["declarations"]}
        assert len(constants) == 10
        assert set(declarations) == set(constants) | {f["id"] for f in cc["functions"]}
        assert c_bindings["WIDE"] == c_bindings["WIDE_ALIAS"]
        for name, value in zip(NAMES, VALUES, strict=True):
            cid = c_bindings[name]
            cvalue, jvalue = constants[cid], declarations[cid]
            assert cvalue["readonly"] is jvalue["readonly"] is True
            assert cvalue["type"] == jvalue["scalar"]
            assert cvalue["value"] == jvalue["value"]
            assert int(cvalue["value"]) == value
            assert isinstance(cvalue["value"], bool if name in ["FALSE", "TRUE"] else str)
            assert jvalue["kind"] == "constant" and jvalue["externally_reachable"]
            assert jvalue["visibility"] == {"kind": "public"}
            assert jvalue["target"]["kind"] == "declaration"
        c_names = {d["id"]: d["symbol"] for d in [*cc["constants"], *cc["functions"]]}
        j_names = {cid: ".".join([d["target"]["path"]["package"], *d["target"]["path"]["owners"],
                                  d["target"]["path"]["member"]]) for cid, d in declarations.items()}
        for d in [declarations[c_bindings["FALSE"]], declarations[c_bindings["TRUE"]]]:
            assert any("constant documentation." in line for line in d["documentation"])
        yield mixed, cc, jj, c_bindings, c_names, j_names


def main():
    c_dir, java_dir, reference, zig = [Path(p).resolve() for p in sys.argv[1:]]
    truth = VALUES + VALUES[:10] + [17]
    assert run([reference]).stdout.splitlines() == list(map(str, truth))
    jdk = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
           if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(jdk) == 1
    work = Path(os.environ["TEST_TMPDIR"]) / "constant-bundles"
    work.mkdir()
    for mixed, cc, jj, bindings, cnames, jnames in inventory(c_dir, java_dir):
        expected = truth if mixed else VALUES
        for java, symbols, directory in [(False, cnames, c_dir), (True, jnames, java_dir)]:
            names = {n: symbols[bindings[n]] for n in NAMES}
            methods = {n: symbols[bindings[n]] for n in FUNCTIONS} if mixed else {}
            original = (directory / (jj["source"] if java else cc["implementation"])).read_text()
            assert "runtime" not in original.lower()
            for mutant in [False, True]:
                trial = work / f"{mixed}-{java}-{mutant}"
                trial.mkdir()
                changed, count = re.subn(r"(?<![\w])62(?![\w])", "17", original)
                assert count == 1
                generated = trial / ("Generated.java" if java else "model.c")
                generated.write_text(changed if mutant else original)
                consumer = trial / ("Consumer.java" if java else "consumer.c")
                consumer.write_text(driver(names, methods, java) + "\n")
                outputs = []
                if java:
                    classes = trial / "classes"
                    classes.mkdir()
                    flags = ["--release", "21", "-Xlint:all", "-Werror", "-implicit:none",
                             "-sourcepath", "", "-cp", classes, "-d", classes]
                    other_units = [p for p in java_dir.rglob("Generated.java")
                                   if p != java_dir / jj["source"]]
                    for unit in [*other_units, generated]:
                        run([jdk[0] / "javac", *flags, unit])
                    run([jdk[0] / "javac", *flags, consumer])
                    outputs.append(run([jdk[0] / "java", "-cp", classes, "Consumer"]).stdout)
                    if not mutant:
                        readonly(trial, True, names, jdk[0] / "javac", flags)
                else:
                    (trial / "api.h").write_bytes((c_dir / cc["header"]).read_bytes())
                    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                             "-Wstrict-prototypes", "-Wmissing-prototypes", "-I", c_dir, "-I", trial]
                    for compiler in ["gcc-14", zig]:
                        for opt in ["0", "2"]:
                            objects = []
                            other_units = [p for p in c_dir.glob("*.c")
                                           if p != c_dir / cc["implementation"]]
                            for i, source in enumerate([*other_units, generated, consumer]):
                                obj = trial / f"unit{i}.o"
                                run([compiler, *flags, "-O" + opt, "-c", source, "-o", obj])
                                objects.append(obj)
                            executable = trial / "consumer"
                            run([compiler, *objects, "-o", executable])
                            outputs.append(run([executable]).stdout)
                        if not mutant:
                            readonly(trial, False, names, compiler, flags)
                wanted = [17 if mutant and v == 62 else v for v in expected]
                for output in outputs:
                    assert output.splitlines() == list(map(str, wanted))
                    assert (output.splitlines() == list(map(str, expected))) != mutant
    print("Two exact constant owners, schemas C4/Java2, Rust/C/Java native truth, readonly and mutant proof")


if __name__ == "__main__":
    main()
