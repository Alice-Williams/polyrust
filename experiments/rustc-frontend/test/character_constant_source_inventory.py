"""Exact source constants, target inventories and original alias identities."""
from copy import deepcopy
from collections import Counter
import re

from constant_export_native import inventory, exact_c_aliases
from character_constant_source_truth import CONSTANTS, READS, reject


def inspect(c_dir, j_dir, mixed):
    root, c = inventory(c_dir)
    jroot, java = inventory(j_dir)
    assert root == jroot and set(c) == set(java)
    owners = {api["defining_key"].rsplit(".", 1)[1]: owner for owner, api in java.items()}
    assert set(owners) == ({"constants", "second", "middle", "root"} if mixed
                           else {"constants", "second", "middle"})
    assert root == owners["root" if mixed else "middle"]
    bindings = {}
    for owner in c:
        cb = {(m["id"], b["namespace"], b["name"]): b["target"]
              for m in c[owner]["modules"] for b in m["bindings"]}
        jb = {(m["id"], b["namespace"], b["name"]): b["target"]["id"]
              for m in java[owner]["modules"] for b in m["bindings"]}
        assert cb == jb
        bindings.update(cb)
    def binding(owner, name, namespace="value"):
        return bindings[owner, namespace, name]
    producer, second, middle = [owners[name] for name in ["constants", "second", "middle"]]
    nested = binding(middle, "nested", "type")
    assert binding(nested, "cycle", "type") == nested
    maximum = binding(producer, "MAXIMUM")
    assert binding(producer, "PUBLIC_ALIAS") == binding(nested, "AGAIN") == maximum
    values = {binding(producer, name): value for name, value in CONSTANTS.items()}
    values[binding(second, "SAME")] = 65
    values[binding(second, "MAXIMUM")] = 0x10ffff
    if mixed:
        values[binding(root, "OWN")] = 0x1f980
        assert binding(root, "RENAMED") == maximum
    integer = binding(producer, "SAME_INTEGER")
    for name in CONSTANTS:
        assert binding(middle, name) == binding(producer, name)
    assert binding(middle, "OTHER_SAME") == binding(second, "SAME") != binding(producer, "ASCII")
    constants = {d["id"]: (owner, d) for owner, api in c.items() for d in api.get("constants", [])}
    jc = {d["id"]: (owner, d) for owner, api in java.items()
          for d in api["declarations"] if d["kind"] == "constant"}
    assert set(constants) == set(jc) == set(values)
    def check_facts(cc, jj):
        for owner in cc:
            owned = {identity: value for identity, value in values.items()
                     if constants[identity][0] == owner}
            if not owned:
                assert "source_types" not in cc[owner] and "source_types" not in jj[owner]
                continue
            assert cc[owner]["schema_version"] == 11
            assert jj[owner]["schema_version"] == 8
            cf = {row["id"]: row for row in cc[owner]["source_types"]["constants"]}
            jf = {row["id"]: row for row in jj[owner]["source_types"]["constants"]}
            expected = {identity: dict(id=identity, scalar="i32" if identity == integer else "char",
                                       value=str(value)) for identity, value in owned.items()}
            assert cf == jf == expected
    check_facts(c, java)
    for language, original in [("c", c), ("java", java)]:
        for field, replacement in [("scalar", "i32"), ("value", "1114110"),
                                    ("id", integer)]:
            changed = deepcopy(original)
            row = next(row for row in changed[producer]["source_types"]["constants"]
                       if row["id"] == maximum)
            row[field] = replacement
            reject(lambda: check_facts(changed, java) if language == "c" else check_facts(c, changed))
    for identity, value in values.items():
        owner, row = constants[identity]
        jowner, jrow = jc[identity]
        assert owner == jowner
        assert row["type"] == ("i32" if identity == integer else "u32")
        assert jrow["scalar"] == "i32"
        assert row["value"] == jrow["value"] == str(value)
        assert jrow["readonly"] and jrow["externally_reachable"]
    for owner in c:
        if owner in [middle, owners.get("root")]:
            exact_c_aliases(c_dir, owner, c[owner], constants)
        else:
            assert not c[owner].get("constant_exports", [])
            assert not java[owner].get("constant_exports", [])
        for field in ["constant_imports", "constant_exports"]:
            cr, jr = c[owner].get(field, []), java[owner].get(field, [])
            key = (lambda row: (row["module"], row["namespace"], row["name"])) if field.endswith("exports") else (lambda row: row["id"])
            assert {key(row) for row in cr} == {key(row) for row in jr}
            jrows = {key(row): row for row in jr}
            for row in cr:
                identity = row["id"]
                defining, target = constants[identity]
                other = jrows[key(row)]
                assert row["owner"] == other["owner"] == defining
                assert row["value"] == other["value"] == str(values[identity])
                assert row["symbol"] == target["symbol"]
                assert other["path"] == jc[identity][1]["target"]["path"]
    cf = {d["id"]: d for api in c.values() for d in api["functions"]}
    jf = {d["id"]: d for api in java.values() for d in api["declarations"] if d["kind"] == "function"}
    assert set(cf) == set(jf)
    assert len(cf) == (len(READS) + 1 if mixed else 0)
    if mixed:
        for name in READS:
            identity = binding(root, "read_" + name)
            assert jf[identity]["externally_reachable"]
            assert jf[identity]["parameters"] == []
        private, = [identity for identity in jf if not jf[identity]["externally_reachable"]]
        assert cf[private]["linkage"] == "internal"
        assert cf[private]["symbol"] not in (c_dir / c[root]["header"]).read_text()
    for directory, apis, language in [(c_dir, c, "c"), (j_dir, java, "java")]:
        expected = {"bundle.json"} | {"polyrust_" + owner.split(":")[0] + ".api.json" for owner in apis}
        for api in apis.values():
            expected.update([api["header"], api["implementation"]] if language == "c" else [api["source"]])
        assert {p.relative_to(directory).as_posix() for p in directory.rglob("*") if p.is_file()} == expected
        assert not any("runtime" in path.lower() for path in expected)
    docs = {
        owners["constants"]: ["Constant-only Unicode producer: original scalar identities, not UTF-16."],
        owners["second"]: ["Independent equal-valued producer; values cannot substitute for identity."],
        owners["middle"]: ["Alias-only facade; no copied constant storage."],
        nested: ["Nested aliases retain their original owner."],
        maximum: ["Maximum Unicode scalar 🦀; changes must invalidate dependent packages."],
    }
    if mixed:
        docs[root] = ["Mixed constant owner, private helper and evaluated constant contexts."]
        docs[binding(root, "OWN")] = ["Root-owned scalar, not an alias."]
    for owner in c:
        declarations = [*java[owner]["modules"], *java[owner]["declarations"]]
        assert {d["id"]: [line.strip() for line in d["documentation"]] for d in declarations} == {
            d["id"]: docs.get(d["id"], []) for d in declarations}
        for language, path, pattern in [
            ("c", c_dir / c[owner]["header"], r"^/\*\s+(.+?)\s+\*/$"),
            ("java", j_dir / java[owner]["source"], r"^\s*\*\s+(.+)$"),
        ]:
            expected = [line for d in declarations for line in docs.get(d["id"], [])]
            if language == "c":
                expected = ["".join(chr(b) if 32 <= b < 127 else f"[0x{b:02X}]"
                                    for b in line.encode("utf-8")) for line in expected]
            def verify(source):
                assert Counter(line.strip() for line in re.findall(pattern, source, re.MULTILINE)) == Counter(expected)
            source = path.read_text()
            verify(source)
            for line in expected:
                reject(lambda: verify(source.replace(line, "changed documentation")))
    return root, owners, c, java, bindings, constants, jc, values, cf, jf
