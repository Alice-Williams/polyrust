"""Exact original exports, signatures, privacy, payloads and math linkage."""
from copy import deepcopy
from collections import Counter
import json
from java_fixture_native import exports, check_inventory_oracle
from remainder_source_docs import inspect_docs, must_reject


def unique_records(records, expected_count, kind=None):
    assert len(records) == expected_count
    assert len({item["id"] for item in records}) == expected_count
    if kind is not None:
        assert all(item["kind"] == kind for item in records)


def exact_imports(records, wanted):
    assert Counter(item["id"] for item in records) == Counter(wanted)


def link_options(api, needs_math):
    assert api["schema_version"] == (9 if needs_math else 8)
    assert api.get("system_libraries", []) == (["m"] if needs_math else [])
    return [{"m": "-lm"}[library] for library in api.get("system_libraries", [])]


def bundle(directory, java):
    index = json.loads((directory / "bundle.json").read_text())
    assert index["schema_version"] == 1 and len(index["members"]) == 3
    owners, files = {}, {"bundle.json"}
    for member in index["members"]:
        api = json.loads((directory / member["manifest"]).read_text())
        assert api["root"] == member["root"] and api["root"] not in owners
        assert api["schema_version"] in ([6] if java else [8, 9])
        owners[api["root"]] = api
        files.add(member["manifest"])
        files.update([api["source"]] if java else [api["header"], api["implementation"]])
    assert files == {str(p.relative_to(directory)) for p in directory.rglob("*") if p.is_file()}
    assert len(files) == (7 if java else 10)
    return index["root"], owners


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    middle, = java[root]["dependencies"]
    leaf, = java[middle]["dependencies"]
    assert java[leaf]["dependencies"] == []
    order, bindings = [leaf, middle, root], {}
    for owner in order:
        assert {item["owner"] for item in c[owner]["imports"]} == set(java[owner]["dependencies"])
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
        link_options(c[owner], owner != leaf)
        text = (c_dir / c[owner]["implementation"]).read_text()
        assert ("#include <math.h>" in text) == (owner == middle)
        assert "runtime" not in text.lower()
        assert "Runtime" not in (java_dir / java[owner]["source"]).read_text()
    kind, module = bindings[middle, "type", "operations"]
    assert kind == "module"
    expected = {(leaf, "value", name) for name in ["left", "right"]}
    expected.add((middle, "type", "operations"))
    expected.update((owner, "value", "remainder") for owner in [module, root])
    check_inventory_oracle(bindings, expected)
    declarations = [d for api in java.values() for d in api["declarations"]]
    native_declarations = [d for api in c.values() for d in api["functions"]]
    for records, kind in [(declarations, "function"), (native_declarations, None)]:
        unique_records(records, 5, kind)
        must_reject(lambda: unique_records([*records, records[0]], 5, kind))
        must_reject(lambda: unique_records([records[0], *records[:-1]], 5, kind))
    wrong_kind = deepcopy(declarations)
    wrong_kind[0]["kind"] = "field"
    must_reject(lambda: unique_records(wrong_kind, 5, "function"))
    functions = {d["id"]: d for d in declarations}
    native = {d["id"]: d for d in native_declarations}
    assert len(functions) == 5 and set(functions) == set(native)
    public_ids = {value[1] for value in bindings.values() if value[0] == "declaration"}
    leaf_ids = {d["id"] for d in java[leaf]["declarations"] if d["kind"] == "function"}
    for identity, declaration in functions.items():
        assert declaration["result"] == "f64"
        assert declaration["parameters"] == ["f64"] * (1 if identity in leaf_ids else 2)
        assert declaration["externally_reachable"] == (identity in public_ids)
        assert native[identity]["parameters"] == declaration["parameters"]
        assert native[identity]["return"] == declaration["result"]
        assert native[identity]["linkage"] == ("external" if identity in public_ids else "internal")
    private, = set(functions) - public_ids
    inspect_docs(java_dir, c_dir, java, c, order, module, bindings, private)
    assert native[private]["symbol"] not in (c_dir / c[leaf]["header"]).read_text()
    for owner, dependency, names in [(middle, leaf, ["left", "right"]), (root, module, ["remainder"])]:
        wanted = {bindings[dependency, "value", name][1] for name in names}
        records = c[owner]["imports"]
        exact_imports(records, wanted)
        must_reject(lambda: exact_imports([*records, records[0]], wanted))
        must_reject(lambda: exact_imports(records[1:], wanted))
        for imported in c[owner]["imports"]:
            declaration = functions[imported["id"]]
            assert imported["parameters"] == declaration["parameters"] and imported["return"] == declaration["result"]
            assert imported["header"] == c[imported["owner"]]["header"]
            assert imported["symbol"] == native[imported["id"]]["symbol"]
    for malformed in [None, [], ["unknown"], ["m", "m"], ["m", "-Wl,unexpected"]]:
        changed = deepcopy(c[root])
        if malformed is None:
            del changed["system_libraries"]
        else:
            changed["system_libraries"] = malformed
        try:
            link_options(changed, True)
        except (KeyError, AssertionError):
            pass
        else:
            raise AssertionError("malformed certified library inventory escaped")
    return order, module, java, c, bindings, functions, native
