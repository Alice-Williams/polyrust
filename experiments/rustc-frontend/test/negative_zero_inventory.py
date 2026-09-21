"""Exact single-crate API, privacy, imports and normalized documentation."""
from collections import Counter
from copy import deepcopy
import re
from java_fixture_native import manifest, exports, check_inventory_oracle

CRATE_DOC = "Numeric behavior of Apache-2.0 stdlib is-negative-zero 0.2.3."
FUNCTION_DOC = "Return true only for IEEE-754 binary64 negative zero."


def reject(check):
    try:
        check()
    except (AssertionError, KeyError):
        return
    raise AssertionError("corrupt source inventory escaped")


def check(java, c):
    root = java["root"]
    assert c["root"] == root
    assert java["schema_version"] == 6 and c["schema_version"] == 8
    assert java["defining_key"] == "proof.stdlib.negative_zero.v1"
    assert java["dependencies"] == [] and c["imports"] == []
    assert c.get("system_libraries", []) == []
    bindings = exports(java, True)
    assert bindings == exports(c, False)
    check_inventory_oracle(bindings, {(root, "value", "is_negative_zero")})
    kind, public = bindings[root, "value", "is_negative_zero"]
    assert kind == "declaration"
    assert len(java["declarations"]) == len(c["functions"]) == 2
    functions = {item["id"]: item for item in java["declarations"]}
    native = {item["id"]: item for item in c["functions"]}
    assert len(functions) == len(native) == 2 and set(functions) == set(native)
    private, = set(functions) - {public}
    for identity, item in functions.items():
        assert item["kind"] == "function" and item["module"] == root
        assert item["parameters"] == native[identity]["parameters"] == ["f64"]
        assert item["result"] == native[identity]["return"] == ("bool" if identity == public else "f64")
        assert item["externally_reachable"] == (identity == public)
        assert item["visibility"] == ({"kind": "public"} if identity == public else
                                      {"kind": "restricted_to", "module": root})
        assert native[identity]["linkage"] == ("external" if identity == public else "internal")
        assert [line.strip() for line in item["documentation"]] == ([FUNCTION_DOC] if identity == public else [])
    assert len(java["modules"]) == len(c["modules"]) == 1
    assert java["modules"][0]["id"] == root
    assert [line.strip() for line in java["modules"][0]["documentation"]] == [CRATE_DOC]
    return public, private, functions, native


def inspect(java_dir, c_dir):
    java, c = manifest(java_dir, 3), manifest(c_dir, 4)
    public, private, functions, native = check(java, c)
    for is_java, group in [(True, "declarations"), (False, "functions")]:
        for replacement in [False, True]:
            changed = deepcopy(java if is_java else c)
            if replacement:
                changed[group][1] = deepcopy(changed[group][0])
            else:
                changed[group].append(deepcopy(changed[group][0]))
            reject(lambda: check(changed, c) if is_java else check(java, changed))
    changed = deepcopy(java)
    changed["declarations"][0]["kind"] = "field"
    reject(lambda: check(changed, c))
    changed = deepcopy(c)
    changed["imports"] = [{"id": "unexpected"}]
    reject(lambda: check(java, changed))
    for index in range(2):
        changed = deepcopy(java)
        changed["declarations"][index]["externally_reachable"] = not changed["declarations"][index]["externally_reachable"]
        reject(lambda: check(changed, c))
        changed = deepcopy(c)
        changed["functions"][index]["linkage"] = "external" if changed["functions"][index]["linkage"] == "internal" else "internal"
        reject(lambda: check(java, changed))
    for group in ["modules", "declarations"]:
        for index in range(len(java[group])):
            changed = deepcopy(java)
            changed[group][index]["documentation"] = ["wrong"]
            reject(lambda: check(changed, c))
    for directory, api, is_java in [(java_dir, java, True), (c_dir, c, False)]:
        text = (directory / api["source" if is_java else "header"]).read_text()
        pattern = r"^\s*\*\s+(.+)$" if is_java else r"^/\*\s+(.+?)\s+\*/$"
        def docs(value):
            assert Counter(line.strip() for line in re.findall(pattern, value, re.MULTILINE)) == Counter([CRATE_DOC, FUNCTION_DOC])
        docs(text)
        for value in [CRATE_DOC, FUNCTION_DOC]:
            reject(lambda: docs(text.replace(value, "wrong")))
    assert native[private]["symbol"] not in (c_dir / c["header"]).read_text()
    return java, c, public, private, functions, native
