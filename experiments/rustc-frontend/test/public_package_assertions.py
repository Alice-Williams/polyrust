"""Independent manifest/schema/API assertions and a handwritten C consumer."""
import json
from pathlib import Path
import sys

directory, consumer = map(Path, sys.argv[1:])
manifest = json.loads((directory / "api.json").read_text())
assert set(manifest) == {"schema_version", "root", "header", "implementation", "modules", "functions"}
assert manifest["schema_version"] == 1
assert sorted(path.name for path in directory.iterdir()) == sorted([
    "api.json", manifest["header"], manifest["implementation"],
])
modules = {module["id"]: module["bindings"] for module in manifest["modules"]}
assert len(modules) == len(manifest["modules"]) == 2
root = {binding["name"]: binding for binding in modules[manifest["root"]]}
assert set(root) == {"zero", "choose", "positive", "exported", "alias", "api"}
api = {binding["name"]: binding for binding in modules[root["api"]["target"]]}
assert set(api) == {"invoke", "again"}
assert api["again"]["target"] == root["api"]["target"]
assert root["exported"]["target"] == root["alias"]["target"] == api["invoke"]["target"]
functions = {function["id"]: function for function in manifest["functions"]}
assert len(functions) == len(manifest["functions"]) == 8
public = {binding["target"] for bindings in modules.values() for binding in bindings if binding["kind"] == "function"}
assert len(public) == 4
assert {identity for identity, function in functions.items() if function["linkage"] == "external"} == public
for identity, function in functions.items():
    assert function["implementation"] == manifest["implementation"]
    assert function["primary"] == manifest["header" if identity in public else "implementation"]
    assert identity.split(":")[0] == manifest["root"].split(":")[0]
header = (directory / manifest["header"]).read_text()
source = (directory / manifest["implementation"]).read_text()
for text in ["Public package root documentation.", "Public alias-module documentation.",
             "Public operation behind a private ancestor."]:
    assert header.count(text) == 1, text
    assert text not in source, text
for text in ["Private ancestor documentation.", "A private concrete representation.", "Restricted helper-module documentation.",
             "Crate-restricted helper documentation.", "The field stays private in C despite Rust pub visibility."]:
    assert source.count(text) == 1, text
    assert text not in header, text
assert source.count(f'#include "{manifest["header"]}"') == 1
assert "struct " not in header
symbols = {name: functions[root[name]["target"]]["symbol"] for name in ["zero", "choose", "positive", "exported"]}
consumer.with_suffix(".symbols").write_text("\n".join(sorted(functions[identity]["symbol"] for identity in public)) + "\n")
private = next(function["symbol"] for identity, function in functions.items() if identity not in public)
consumer.with_suffix(".private.c").write_text(f'#include "{manifest["header"]}"\nint main(void) {{ return {private}(0); }}\n')
body = f'''int main(void) {{
    const int32_t values[] = {{INT32_MIN, -1, 0, 1, 42, INT32_MAX}};
    if ({symbols["zero"]}() != 42) {{ return 1; }}
    for (unsigned int i = 0; i < sizeof(values)/sizeof(values[0]); ++i) {{
        const int32_t value = values[i];
        if ({symbols["exported"]}(value) != value) {{ return 2; }}
        if ({symbols["positive"]}(value) != (value > 0)) {{ return 3; }}
        if ({symbols["choose"]}(value, 1, -1, 0) != value) {{ return 4; }}
        if ({symbols["choose"]}(value, 0, -1, 1) != -1) {{ return 5; }}
        if ({symbols["choose"]}(value, 0, -1, 0) != 42) {{ return 6; }}
    }}
    int32_t value = 0;
    while (scanf("%" SCNd32, &value) == 1) {{
        printf("%" PRId32 " %" PRId32 " %" PRId32 " %" PRId32 " %d %" PRId32 " %" PRId32 " %" PRId32 "\\n",
            {symbols["zero"]}(), {symbols["exported"]}(value), {symbols["exported"]}(value), {symbols["exported"]}(value),
            {symbols["positive"]}(value), {symbols["choose"]}(value, 1, -1, 0),
            {symbols["choose"]}(value, 0, -1, 1), {symbols["choose"]}(value, 0, -1, 0));
    }}
    return 0;
}}
'''
generated = f'#include "{manifest["header"]}"\n'
standard = '#include <inttypes.h>\n#include <stdio.h>\n'
for order, preamble in {
    "header-first": generated + generated + standard,
    "standards-first": standard + generated + generated,
    "interleaved": '#include <inttypes.h>\n' + generated + '#include <stdio.h>\n' + generated,
}.items():
    consumer.with_suffix(f".{order}.c").write_text(preamble + body)
