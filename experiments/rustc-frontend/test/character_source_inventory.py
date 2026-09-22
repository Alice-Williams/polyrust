"""Exact original/target scalar inventories, owners, bindings and domain metadata."""
from copy import deepcopy
from pathlib import Path
import re

from constant_export_native import inventory, java_path

LITERALS = ["nul", "one", "ascii_end", "non_ascii", "byte_end", "above_byte",
            "unassigned", "two_byte_end", "three_byte_start", "before_surrogates",
            "after_surrogates", "noncharacter", "bmp_noncharacter", "bmp_end",
            "supplementary", "crab", "private_use", "penultimate", "maximum"]
COMPARISONS = ["equal", "not_equal", "less", "less_equal", "greater", "greater_equal"]
DOMAIN = "Unicode scalar: 0..=0x10ffff excluding 0xd800..=0xdfff"


def rejected(check):
    try:
        check()
    except AssertionError:
        return
    raise AssertionError("metadata corruption escaped the contract")


def inspect(c_dir, j_dir):
    root, c = inventory(c_dir)
    jroot, java = inventory(j_dir)
    assert root == jroot and set(c) == set(java) and len(c) == 3
    owners = {api["defining_key"].split(".")[1].removeprefix("character_"): owner
              for owner, api in java.items()}
    assert set(owners) == {"leaf", "middle", "root"} and root == owners["root"]
    cf = {f["id"]: f for api in c.values() for f in api["functions"]}
    jf = {f["id"]: f for api in java.values() for f in api["declarations"]
          if f["kind"] == "function"}
    assert set(cf) == set(jf) and len(cf) == 44
    bindings = {}
    for owner in c:
        cc, jj = c[owner], java[owner]
        assert cc["schema_version"] == 10 and jj["schema_version"] == 7
        cb = {(m["id"], b["namespace"], b["name"]): b["target"]
              for m in cc["modules"] for b in m["bindings"]}
        jb = {(m["id"], b["namespace"], b["name"]): b["target"]["id"]
              for m in jj["modules"] for b in m["bindings"]}
        assert cb == jb
        bindings.update(cb)
        def source_types(left, right):
            assert left == right and left["char_foreign_input_domain"] == DOMAIN
            functions = {f["id"]: f for f in left["functions"]}
            assert set(functions) == {f["id"] for f in cc["functions"]}
            for identity, original in functions.items():
                assert all(p in ["char", "i32", "bool"] for p in original["parameters"])
                assert original["result"] in ["char", "i32", "bool"]
                for target, mapping, result_key in [
                    (cf[identity], dict(char="u32", i32="i32", bool="bool"), "return"),
                    (jf[identity], dict(char="i32", i32="i32", bool="bool"), "result"),
                ]:
                    assert target[result_key] == mapping[original["result"]]
                    assert target["parameters"] == [mapping[p] for p in original["parameters"]]
            fields = {f["id"]: f for f in left["fields"]}
            jfields = {f["id"]: f for f in jj["declarations"] if f["kind"] == "field"}
            assert set(fields) == set(jfields)
            assert sorted(f["scalar"] for f in fields.values()) == (["char", "char", "i32", "i32"] if owner == owners["leaf"] else [])
            assert all(f["owner"] == jfields[identity]["owner"] for identity, f in fields.items())
            assert all(field["scalar"] == "i32" for field in jfields.values())
        source_types(cc["source_types"], jj["source_types"])
        changed = deepcopy(cc["source_types"])
        changed["char_foreign_input_domain"] = "all integers"
        rejected(lambda: source_types(changed, jj["source_types"]))
        if changed["fields"]:
            changed = deepcopy(cc["source_types"])
            chars = [f for f in changed["fields"] if f["scalar"] == "char"]
            chars[0]["owner"], chars[1]["owner"] = chars[1]["owner"], chars[0]["owner"]
            rejected(lambda: source_types(changed, jj["source_types"]))
        changed = deepcopy(cc["source_types"])
        original = next(f for f in changed["functions"] if f["result"] == "char")
        original["result"] = "i32"
        rejected(lambda: source_types(changed, jj["source_types"]))
        public = {target for (module, namespace, _), target in cb.items() if namespace == "value"}
        for row in cc["functions"]:
            identity = row["id"]
            assert jf[identity]["externally_reachable"] == (identity in public)
            assert row["linkage"] == ("external" if identity in public else "internal")
            assert (row["symbol"] in (c_dir / cc["header"]).read_text()) == (identity in public)
        for language, source in [("c", c_dir / cc["implementation"]), ("java", j_dir / jj["source"])]:
            body = re.sub(r"/\*.*?\*/|//[^\n]*", "", source.read_text(), flags=re.DOTALL)
            assert "runtime" not in body.lower() and "goto " not in body
            assert ("uint32_t" if language == "c" else "int ") in body
        records = [*jj["modules"], *jj["declarations"]]
        for record in records:
            for doc in record["documentation"]:
                assert doc.strip() in (j_dir / jj["source"]).read_text()
                assert doc.strip() in (c_dir / cc["header"]).read_text()
    leaf = owners["leaf"]
    for owner, name in [(leaf, "identity"), (root, "identity")]:
        assert bindings[owner, "value", "scalar_alias"] == bindings[owner, "value", name]
    operations = bindings[owners["middle"], "type", "operations"]
    assert bindings[owners["middle"], "value", "scalar_alias"] == bindings[operations, "value", "forward"]
    for folder, apis, keys in [(c_dir, c, ["header", "implementation"]), (j_dir, java, ["source"])]:
        expected = {"bundle.json"} | {api[key] for api in apis.values() for key in keys}
        expected |= {"polyrust_" + owner.split(":")[0] + ".api.json" for owner in apis}
        assert {p.relative_to(folder).as_posix() for p in folder.rglob("*") if p.is_file()} == expected
    return root, owners, c, java, bindings, cf, jf


def expressions(evidence, language):
    root, owners, _, _, bindings, cf, jf = evidence
    def name(module, member):
        identity = bindings[module, "value", member]
        return cf[identity]["symbol"] if language == "c" else java_path(jf[identity]["target"]["path"])
    literals = bindings[owners["leaf"], "type", "literals"]
    compare = bindings[owners["leaf"], "type", "compare"]
    prefix = [name(literals, member) + "()" for member in LITERALS]
    rows = [name(root, member) + "(left)" for member in ["identity", "local", "scalar_alias"]]
    rows += [name(root, "select") + f"({condition}, left, right)" for condition in ["true", "false"]]
    rows += [name(root, member) + "(left, marker)" for member in ["record_character", "record_marker"]]
    flags = " | ".join(f"(({name(compare, member)}(left, right) ? 1 : 0) << {i})"
                       for i, member in enumerate(COMPARISONS))
    rows += [flags, name(root, "nested_less") + "(left, right) ? 1 : 0"]
    return prefix, rows
