"""Independent source inventory and result schemas for the unit fixture."""
import json
from pathlib import Path
from java_fixture_native import exports

SIGNATURES = {
    "unit_leaf": {
        "empty": ([], "unit"),
        "explicit": ([], "unit"),
        "first": (["i32"], "i32"),
        "second": (["i32"], "i32"),
        "predicate": (["bool"], "bool"),
        "observe": (["i32", "i32", "bool"], "unit"),
    },
    "unit_relay": {"relay": (["i32", "bool"], "unit")},
    "unit_root": {
        "execute": (["i32", "bool"], "i32"),
        "tail": (["i32"], "unit"),
        "branch": (["bool"], "unit"),
    },
}

def inventory(java_bundle, c_bundle):
    def members(bundle, java):
        index = json.loads((bundle / "bundle.json").read_text())
        assert index["schema_version"] == 1 and len(index["members"]) == 3
        apis = [json.loads((bundle / item["manifest"]).read_text()) for item in index["members"]]
        assert len({item["root"] for item in apis}) == 3
        expected = {"bundle.json", *(item["manifest"] for item in index["members"])}
        for api in apis:
            expected.update([api["source"]] if java else [api["header"], api["implementation"]])
            assert api["schema_version"] == (5 if java else 7)
        assert expected == {str(path.relative_to(bundle)) for path in bundle.rglob("*") if path.is_file()}
        return {api["root"]: api for api in apis}
    java = members(java_bundle, True)
    c = members(c_bundle, False)
    assert java.keys() == c.keys()
    named = {}
    for root, api in java.items():
        label = api["defining_key"].removeprefix("proof.").removesuffix(".v1")
        assert label in SIGNATURES and label not in named
        native = c[root]
        bindings = exports(api, True)
        c_bindings = {key: (("declaration" if kind == "constant" else kind), identity)
                      for key, (kind, identity) in exports(native, False).items()}
        assert bindings == c_bindings
        declarations = {item["id"]: item for item in api["declarations"] if item["kind"] == "function"}
        functions = {item["id"]: item for item in native["functions"]}
        assert declarations.keys() == functions.keys()
        expected = set(SIGNATURES[label])
        if label in ("unit_leaf", "unit_root"):
            expected.add("MARK")
        assert {key[2] for key in bindings} == expected
        calls = {}
        for name, (parameters, result) in SIGNATURES[label].items():
            identity = bindings[root, "value", name][1]
            description, function = declarations[identity], functions[identity]
            assert description["parameters"] == function["parameters"] == parameters
            assert description["result"] == function["return"] == result
            assert description["externally_reachable"] and function["linkage"] == "external"
            path = description["target"]["path"]
            calls[name] = (".".join([path["package"], *path["owners"], path["member"]]), function["symbol"])
        private = [item for item in declarations.values() if not item["externally_reachable"]]
        assert len(private) == (1 if label == "unit_root" else 0)
        for item in private:
            assert functions[item["id"]]["linkage"] == "internal"
            assert item["result"] == functions[item["id"]]["return"] == "unit"
        for is_java, bundle, owner in [(True, java_bundle, api), (False, c_bundle, native)]:
            text = (bundle / (owner["source"] if is_java else owner["implementation"])).read_text()
            for forbidden in ["Runtime", "runtime.", "java.lang.Void", "poly_unit", "goto "]:
                assert forbidden not in text
            if not is_java:
                assert "struct " not in text and "typedef " not in text
        named[label] = (api, native, calls)
    return named
