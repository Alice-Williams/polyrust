"""Exact original bindings, defining values, file inventories and visibility."""
from collections import Counter
from copy import deepcopy
import re

from constant_export_native import inventory, java_path, exact_c_aliases
from infinite_constant_source_truth import CONSTANTS, READS, reject


def inspect(c_dir, j_dir, mixed):
    root, c = inventory(c_dir)
    jroot, java = inventory(j_dir)
    assert root == jroot and set(c) == set(java) and len(c) == (4 if mixed else 3)
    owners = {m["defining_key"].rsplit(".", 1)[1]: owner for owner, m in java.items()}
    assert set(owners) == ({"constants", "second", "middle", "root"} if mixed
                           else {"constants", "second", "middle"})
    assert root == owners["root" if mixed else "middle"]
    def exact_schemas(cc, jj):
        for owner in cc:
            # The mixed C owner also declares the system math linkage contract.
            assert cc[owner]["schema_version"] == (9 if mixed and owner == root else 8)
            assert jj[owner]["schema_version"] == 6
    exact_schemas(c, java)
    for owner in c:
        for target, other in [(c, java), (java, c)]:
            changed = deepcopy(target)
            changed[owner]["schema_version"] -= 1
            reject(lambda: exact_schemas(changed, other) if target is c
                   else exact_schemas(other, changed))
    constants = {d["id"]: (owner, d) for owner, api in c.items() for d in api.get("constants", [])}
    jc = {d["id"]: (owner, d) for owner, api in java.items()
          for d in api["declarations"] if d["kind"] == "constant"}
    assert set(constants) == set(jc) and len(constants) == (23 if mixed else 22)
    bindings = {}
    for owner in c:
        cb = {(m["id"], b["namespace"], b["name"]): b["target"]
              for m in c[owner]["modules"] for b in m["bindings"]}
        jb = {(m["id"], b["namespace"], b["name"]): b["target"]["id"]
              for m in java[owner]["modules"] for b in m["bindings"]}
        assert cb == jb
        bindings.update(cb)
        expected_files = {"bundle.json"} | {
            path for api in c.values() for path in [api["header"], api["implementation"]]
        } | {"polyrust_" + oid.split(":")[0] + ".api.json" for oid in c}
        assert {p.relative_to(c_dir).as_posix() for p in c_dir.rglob("*") if p.is_file()} == expected_files
        expected_java = {"bundle.json"} | {api["source"] for api in java.values()} | {
            "polyrust_" + oid.split(":")[0] + ".api.json" for oid in java}
        assert {p.relative_to(j_dir).as_posix() for p in j_dir.rglob("*") if p.is_file()} == expected_java
        for path in [c_dir / c[owner]["implementation"], j_dir / java[owner]["source"]]:
            body = re.sub(r"/\*.*?\*/|//[^\n]*", "", path.read_text(), flags=re.DOTALL)
            assert "runtime" not in body.lower() and "goto " not in body

    def binding(owner, name, namespace="value"):
        return bindings[owner, namespace, name]

    producer, middle, second = [owners[name] for name in ["constants", "middle", "second"]]
    nested = binding(middle, "nested", "type")
    expected = {(producer, "value", name) for name in [*CONSTANTS, "PUBLIC_ALIAS"]}
    expected |= {(second, "value", "NEGATIVE"), (second, "value", "SAME_VALUE"), (middle, "type", "nested"),
                 (nested, "type", "cycle"), (nested, "value", "AGAIN")}
    expected |= {(middle, "value", name) for name in [*CONSTANTS, "OTHER_NEGATIVE", "OTHER_SAME"]}
    if mixed:
        expected |= {(root, "value", name) for name in [
            "NAMED_NEGATIVE", "NAMED_POSITIVE", "RENAMED", "OWN",
            *["read_" + name for name in READS]]}
    def check_bindings(actual):
        assert set(actual) == expected
    check_bindings(bindings)
    reject(lambda: check_bindings({**bindings, (root, "value", "extra"): root}))
    assert binding(nested, "cycle", "type") == nested
    assert binding(producer, "PUBLIC_ALIAS") == binding(nested, "AGAIN") == binding(producer, "NAMED_POSITIVE")
    for name in CONSTANTS:
        assert binding(producer, name) == binding(middle, name)
    assert binding(middle, "OTHER_NEGATIVE") == binding(second, "NEGATIVE") != binding(producer, "NAMED_POSITIVE")
    assert binding(middle, "OTHER_SAME") == binding(second, "SAME_VALUE") != binding(second, "NEGATIVE")
    assert binding(producer, "ALIAS_POSITIVE") != binding(producer, "NAMED_POSITIVE")
    values = {binding(producer, name): value for name, value in CONSTANTS.items()}
    values[binding(second, "NEGATIVE")] = CONSTANTS["NAMED_NEGATIVE"]
    values[binding(second, "SAME_VALUE")] = CONSTANTS["NAMED_POSITIVE"]
    if mixed:
        values[binding(root, "OWN")] = CONSTANTS["NAMED_NEGATIVE"]

    def exact_values(cc, jj):
        assert set(cc) == set(jj) == set(values)
        for identity, bits in values.items():
            owner, row = cc[identity]
            jowner, jrow = jj[identity]
            assert owner == jowner and row["type"] == jrow["scalar"] == "f64"
            assert row["value"] == jrow["value"] == f"0x{bits:016x}"
            assert jrow["readonly"] is True and jrow["externally_reachable"] is True
    exact_values(constants, jc)
    changed = deepcopy(constants)
    identity = binding(producer, "NAMED_POSITIVE")
    changed[identity][1]["value"] = f"0x{values[identity] ^ (1 << 63):016x}"
    reject(lambda: exact_values(changed, jc))

    for owner in c:
        if owner in [middle, owners.get("root")]:
            exact_c_aliases(c_dir, owner, c[owner], constants)
        else:
            assert not c[owner].get("constant_exports", []) and not java[owner].get("constant_exports", [])
        for field in ["constant_imports", "constant_exports"]:
            cr, jr = c[owner].get(field, []), java[owner].get(field, [])
            key = (lambda d: (d["module"], d["namespace"], d["name"])) if field.endswith("exports") else (lambda d: d["id"])
            assert {key(d) for d in cr} == {key(d) for d in jr}
            jrows = {key(d): d for d in jr}
            for row in cr:
                identity = row["id"]
                defining, original = constants[identity]
                jrow = jrows[key(row)]
                assert row["owner"] == jrow["owner"] == defining
                assert row["value"] == jrow["value"] == f"0x{values[identity]:016x}"
                assert row["type"] == jrow["scalar"] == "f64"
                assert row["readonly"] is jrow["readonly"] is True
                assert row["symbol"] == original["symbol"] and row["header"] == c[defining]["header"]
                assert jrow["path"] == jc[identity][1]["target"]["path"]
        imported = {d["id"] for d in c[owner].get("constant_imports", [])}
        assert imported == (set(values) - {binding(root, "OWN")} if mixed and owner == root else set())
    cfunctions = {d["id"]: d for api in c.values() for d in api["functions"]}
    jfunctions = {d["id"]: d for api in java.values() for d in api["declarations"] if d["kind"] == "function"}
    public = {binding(root, "read_" + name) for name in READS} if mixed else set()
    assert set(cfunctions) == set(jfunctions)
    assert len(cfunctions) == (len(READS) + 1 if mixed else 0)
    for identity, row in cfunctions.items():
        jrow = jfunctions[identity]
        assert jrow["result"] == "f64" and jrow["parameters"] == []
        assert jrow["externally_reachable"] == (identity in public)
        assert row["linkage"] == ("external" if identity in public else "internal")
        header = (c_dir / c[root]["header"]).read_text()
        assert (row["symbol"] in header) == (identity in public)
    docs = {d["id"]: [] for api in java.values() for d in [*api["modules"], *api["declarations"]]}
    for name, description in {
        "constants": "Original infinity constants: exact signs and compiler-evaluated expressions.",
        "second": "Independent infinity producer with an equal-valued constant.",
        "middle": "Alias-only facade; infinity constants retain their original producer.",
        "root": "Mixed infinity owner with private, local, inherent and composed reads.",
    }.items():
        if name in owners:
            docs[owners[name]] = [description]
    docs[nested] = ["Original module identity survives aliases and a finite module cycle."]
    docs[binding(producer, "NAMED_POSITIVE")] = ["Positive infinity; changing its sign must invalidate every dependent."]
    docs[binding(producer, "NAMED_NEGATIVE")] = ["Negative infinity has a distinct constant representation."]
    if mixed:
        docs[binding(root, "OWN")] = ["Root-owned negative infinity, not an alias of another crate."]
    for owner in c:
        records = [*java[owner]["modules"], *java[owner]["declarations"]]
        assert {d["id"]: [line.strip() for line in d["documentation"]] for d in records} == {d["id"]: docs[d["id"]] for d in records}
        for language, path, pattern in [
            ("c", c_dir / c[owner]["header"], r"^/\*\s+(.+?)\s+\*/$"),
            ("java", j_dir / java[owner]["source"], r"^\s*\*\s+(.+)$"),
        ]:
            expected_docs = [line for d in records for line in docs[d["id"]]]
            if language == "c":
                expected_docs = ["".join(chr(b) if 32 <= b < 127 else f"[0x{b:02X}]"
                                          for b in line.encode("utf-8")) for line in expected_docs]
            def verify_docs(source):
                assert Counter(line.strip() for line in re.findall(pattern, source, re.MULTILINE)) == Counter(expected_docs)
            source = path.read_text()
            verify_docs(source)
            for line in expected_docs:
                reject(lambda: verify_docs(source.replace(line, "wrong documentation")))
    return root, owners, c, java, bindings, constants, jc, values, cfunctions, jfunctions
