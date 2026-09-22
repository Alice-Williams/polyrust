"""Exact original identities, public/private signatures, docs and file inventory."""
from collections import Counter
from copy import deepcopy
import re
from addition_source_inventory import bundle, reject
from java_fixture_native import exports, check_inventory_oracle


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    middle, = java[root]["dependencies"]
    leaf, = java[middle]["dependencies"]
    assert java[leaf]["dependencies"] == []
    order, bindings = [leaf, middle, root], {}
    for owner in order:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
        assert {d["owner"] for d in c[owner]["imports"]} == set(java[owner]["dependencies"])
        for path in [c_dir / c[owner]["implementation"], java_dir / java[owner]["source"]]:
            text = path.read_text()
            assert "runtime" not in text.lower() and "goto " not in text and "Math." not in text
    kind, module = bindings[middle, "type", "operations"]
    assert kind == "module"
    check_inventory_oracle(bindings, {(leaf, "value", "input"), (middle, "type", "operations"),
                                     (module, "value", "widen"), (root, "value", "widen")})
    rows = [d for api in java.values() for d in api["declarations"]]
    c_rows = [d for api in c.values() for d in api["functions"]]
    functions, native = {d["id"]: d for d in rows}, {d["id"]: d for d in c_rows}
    assert len(rows) == len(c_rows) == len(functions) == len(native) == 4
    assert set(functions) == set(native)
    public = {v[1] for v in bindings.values() if v[0] == "declaration"}
    private = set(functions) - public
    hidden, = private
    input_id = bindings[leaf, "value", "input"][1]
    results = {identity: ("i32" if identity in [hidden, input_id] else "i64") for identity in functions}

    def declarations(actual, c_actual):
        assert set(actual) == set(c_actual) == set(results)
        for identity, result in results.items():
            d, n = actual[identity], c_actual[identity]
            assert d["kind"] == "function" and d["parameters"] == ["i32"] and d["result"] == result
            assert d["externally_reachable"] == (identity in public)
            assert n["linkage"] == ("external" if identity in public else "internal")
    declarations(functions, native)
    for identity in functions:
        for field, wrong in [("result", "bool"), ("parameters", []),
                             ("externally_reachable", not functions[identity]["externally_reachable"])]:
            changed = deepcopy(functions)
            changed[identity][field] = wrong
            reject(lambda: declarations(changed, native))
    for owner in order:
        header = (c_dir / c[owner]["header"]).read_text()
        body = (c_dir / c[owner]["implementation"]).read_text()
        for d in c[owner]["functions"]:
            identity, symbol = d["id"], d["symbol"]
            def signature(text, is_header):
                found = re.findall(r"^(static )?(int32_t|int64_t) " + re.escape(symbol)
                                   + r"\(([^()]*)\)\s*[;{]", text, re.MULTILINE)
                count = (int(identity in public) if is_header else (1 if identity in public else 2))
                assert len(found) == count
                for storage, result, parameters in found:
                    assert bool(storage) == (identity in private)
                    assert result == "int" + results[identity][1:] + "_t"
                    assert parameters.strip().split()[0] == "int32_t" and "," not in parameters
            signature(header, True)
            signature(body, False)
            reject(lambda: signature(body.replace("int" + results[identity][1:] + "_t " + symbol,
                                                   "_Bool " + symbol), False))
    for owner, dependency, name in [(middle, leaf, "input"), (root, module, "widen")]:
        wanted = bindings[dependency, "value", name][1]
        def imports(rows):
            assert len(rows) == 1 and rows[0]["id"] == wanted
            d = rows[0]
            assert d["parameters"] == ["i32"] and d["return"] == results[wanted]
            assert d["symbol"] == native[wanted]["symbol"] and d["header"] == c[d["owner"]]["header"]
        imports(c[owner]["imports"])
        reject(lambda: imports([]))
        reject(lambda: imports(c[owner]["imports"] * 2))
    docs = {leaf: ["Original signed operand and private implementation."],
            middle: ["Checked signed widening in its original public module."],
            root: ["Public forwarding without copying dependency implementations."],
            module: ["Widening preserves source module identity."], hidden: [],
            input_id: ["Original signed 32-bit operand."],
            bindings[module, "value", "widen"][1]: ["Preserve the exact signed value across widths."],
            bindings[root, "value", "widen"][1]: []}
    for owner in order:
        records = [*java[owner]["modules"], *java[owner]["declarations"]]
        assert {d["id"]: [s.strip() for s in d["documentation"]] for d in records} == {d["id"]: docs[d["id"]] for d in records}
        wanted_docs = Counter(line for d in records for line in docs[d["id"]])
        for path, pattern in [(java_dir / java[owner]["source"], r"^\s*\*\s+(.+)$"),
                              (c_dir / c[owner]["header"], r"^/\*\s+(.+?)\s+\*/$")]:
            def rendered(text):
                assert Counter(line.strip() for line in re.findall(pattern, text, re.MULTILINE)) == wanted_docs
            text = path.read_text()
            rendered(text)
            for document in wanted_docs:
                reject(lambda: rendered(text.replace(document, "changed")))
    return order, module, java, c, bindings, functions, native, private
