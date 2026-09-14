"""Probe compiler facts and separately compile/run the generated public facade."""
import os
from pathlib import Path
import re
import subprocess
import sys
from java_source_native import run


def main():
    probe, adapter, fixture = sys.argv[1:]
    fixtures = Path(fixture).parent
    work = Path(os.environ["TEST_TMPDIR"]) / "java-ast"
    work.mkdir()
    observations = {}
    for case, flags in [
        ("model", []), ("alternate", []), ("scopes", []), ("mapping_inventory", []),
        ("documentation", ["--input", str(fixtures / "documentation.md")]),
        ("direct_calls", ["--input", str(fixtures / "direct_calls_out.rs")]),
        ("boolean_order", []), ("public_package", ["--package"]),
        ("java_same_spelling", ["--package"]),
    ]:
        output = work / case / "Generated.java"
        output.parent.mkdir()
        source = str(fixtures / (case + ".rs"))
        observed = run([probe, source, str(output), *flags])
        assert "AST_PACKAGE_CHECKED" in observed
        assert "SCOPES_CHECKED\t" in observed
        production = work / (case + "-production") / "Generated.java"
        production.parent.mkdir()
        run([adapter, source, str(production), *flags])
        assert production.read_bytes() == output.read_bytes(), case
        observations[case] = observed
    assert "BORROW_DEPTH\t2" in observations["mapping_inventory"]
    assert "FIELD_ORDER\t2" in observations["mapping_inventory"]
    assert "FIELD_ORDER\t2" in observations["direct_calls"]
    assert "CALL_ORDER\t3" in observations["direct_calls"]
    assert "COMPARISON_ORDER" in observations["direct_calls"]
    operators = {line.split("\t")[1] for line in observations["boolean_order"].splitlines()
                 if line.startswith("BOOLEAN_AST\t")}
    assert operators == {"Equal", "NotEqual", "Less", "LessEqual", "Greater", "GreaterEqual"}, operators
    observed = observations["public_package"]
    assert "DEPENDENCY_API_CHECKED\t4" in observed
    calls = {}
    private = {}
    records = {}
    for line in observed.splitlines():
        parts = line.split("\t")
        if parts[0] == "CALL":
            assert len(parts) == 4
            table = calls if parts[3] == "public" else private
            assert parts[1] not in table
            table[parts[1]] = parts[2]
        elif parts[0] == "RECORD":
            records[parts[1]] = parts[2]
    assert set(calls) == {"zero", "choose", "positive", "hidden::exported"}, calls
    assert "hidden::identity" in private
    assert set(records) == {"hidden::Record"}
    dependency_calls = {parts[1]: parts[2] for line in observed.splitlines()
                        if (parts := line.split("\t"))[0] == "DEPENDENCY_SOURCE"}
    assert dependency_calls == calls, (dependency_calls, calls)
    generated = work / "public_package" / "Generated.java"
    text = generated.read_text()
    namespace, = re.findall(r"^package (org\.polyrust\.generated\.r[0-9a-f]{16});$", text, re.MULTILINE)
    source = f"""import {namespace}.Generated;
public final class PublicConsumer {{
  public static void main(String[] args) {{
    if (Generated.{calls['zero']}() != 42) throw new AssertionError();
    for (int value : new int[] {{Integer.MIN_VALUE, -7, 0, 13, Integer.MAX_VALUE}}) {{
      if (Generated.{calls['hidden::exported']}(value) != value) throw new AssertionError();
      if (Generated.{calls['positive']}(value) != (value > 0)) throw new AssertionError();
      for (boolean take : new boolean[] {{false, true}})
        for (boolean other : new boolean[] {{false, true}})
          if (Generated.{calls['choose']}(value, take, -9, other) != (take ? value : other ? -9 : 42)) throw new AssertionError();
    }}
  }}
}}"""
    consumer = work / "PublicConsumer.java"
    consumer.write_text(source)
    runtime, = [path / "bin" for path in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in path.name and (path / "bin/javac").is_file()]
    classes = work / "classes"
    classes.mkdir()
    javac = [str(runtime / "javac"), "--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror"]
    run([*javac, "-d", str(classes), str(generated), str(consumer)])
    run([str(runtime / "java"), "-cp", str(classes), "PublicConsumer"])
    for member in ["new Generated()", f"Generated.{private['hidden::identity']}(1)",
                   f"new Generated.{records['hidden::Record']}(1)"]:
        denied = work / "PrivateConsumer.java"
        denied.write_text(f"import {namespace}.Generated; public final class PrivateConsumer {{ Object value = {member}; }}")
        result = subprocess.run([*javac, "-cp", str(classes), str(denied)], capture_output=True, text=True, check=False)
        assert result.returncode != 0 and "private access" in result.stderr, result.stderr
    same = observations["java_same_spelling"]
    assert "DEPENDENCY_API_CHECKED\t2" in same
    same_calls = {parts[1]: parts[2] for line in same.splitlines()
                  if (parts := line.split("\t"))[0] == "DEPENDENCY_SOURCE"}
    assert set(same_calls) == {"first::value", "second::value"}, same_calls
    assert len(set(same_calls.values())) == 2, same_calls
    same_generated = work / "java_same_spelling" / "Generated.java"
    same_namespace, = re.findall(r"^package (org\.polyrust\.generated\.r[0-9a-f]{16});$",
                                 same_generated.read_text(), re.MULTILINE)
    same_consumer = work / "SameSpellingConsumer.java"
    same_consumer.write_text(f"""import {same_namespace}.Generated;
public final class SameSpellingConsumer {{
  public static void main(String[] args) {{
    for (int value : new int[] {{Integer.MIN_VALUE, -7, 0, 13, Integer.MAX_VALUE}}) {{
      if (Generated.{same_calls['first::value']}(value) != value) throw new AssertionError();
      if (Generated.{same_calls['second::value']}(value) != value) throw new AssertionError();
    }}
  }}
}}""")
    same_classes = work / "same-classes"
    same_classes.mkdir()
    run([*javac, "-d", str(same_classes), str(same_generated), str(same_consumer)])
    run([str(runtime / "java"), "-cp", str(same_classes), "SameSpellingConsumer"])
    print("compiler AST/borrow/order/scope/doc/API assertions pass; same-spelled functions stay distinct and private members reject")


if __name__ == "__main__":
    main()
