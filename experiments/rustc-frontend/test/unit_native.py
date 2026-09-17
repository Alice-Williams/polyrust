"""Real three-crate Rust/C/Java unit ABI and explicit trace proof."""
import os
import json
from pathlib import Path
import re
import sys
from constant_export_scratch import writable_copy
from unit_inventory import inventory
from unit_trace import expected, instrument
from unit_consumers import compile_c, compile_java, run
from unit_examples import export

def mutate(text, calls, java, fault):
    symbol = calls["observe"][0] if java else calls["observe"][1]
    pattern = re.compile(r"\b" + re.escape(symbol) + r"\(([^()]*)\);")
    matches = list(pattern.finditer(text))
    assert len(matches) == 1, (fault, text)
    match = matches[0]
    if fault == "duplicate":
        replacement = match[0] + match[0]
    elif fault == "drop":
        replacement = "" if java else "".join(f"(void){arg.strip()};" for arg in match[1].split(","))
    else:
        assert fault == "reordered"
        first = calls["first"][0] if java else calls["first"][1]
        second = calls["second"][0] if java else calls["second"][1]
        assert text.count(first + "(") == text.count(second + "(") == 1
        return text.replace(first + "(", "TRACE_SWAP(").replace(second + "(", first + "(").replace("TRACE_SWAP(", second + "(")
    return text[:match.start()] + replacement + text[match.end():]

def schema_controls(root, java, c):
    for is_java, original in [(True, java), (False, c)]:
        for label in ["index", "owner"]:
            trial = root / (("java-" if is_java else "c-") + "schema-" + label)
            writable_copy(original, trial)
            index = json.loads((trial / "bundle.json").read_text())
            path = trial / ("bundle.json" if label == "index" else index["members"][0]["manifest"])
            data = json.loads(path.read_text())
            data["schema_version"] = 999
            path.write_text(json.dumps(data))
            try:
                inventory(trial if is_java else java, c if is_java else trial)
            except AssertionError:
                pass
            else:
                raise AssertionError("unknown emitted schema accepted")
def main():
    java_bundle, c_bundle, reference, zig = [Path(arg).resolve() for arg in sys.argv[1:]]
    owners = inventory(java_bundle, c_bundle)
    truth, traces = expected()
    assert run([reference]).stdout == truth
    root = Path(os.environ["TEST_TMPDIR"]) / "unit-native"
    root.mkdir()
    schema_controls(root, java_bundle, c_bundle)
    for java, original in [(False, c_bundle), (True, java_bundle)]:
        for variant in ["plain", "traced", "duplicate", "drop", "reordered"]:
            work = root / (("java-" if java else "c-") + variant)
            work.mkdir()
            bundle = work / "bundle"
            writable_copy(original, bundle)
            if variant != "plain":
                leaf = owners["unit_leaf"][0 if java else 1]
                path = bundle / leaf["source" if java else "implementation"]
                path.write_text(instrument(path.read_text(), owners["unit_leaf"][2], java))
            if variant not in ("plain", "traced"):
                relay = owners["unit_relay"][0 if java else 1]
                path = bundle / relay["source" if java else "implementation"]
                path.write_text(mutate(path.read_text(), owners["unit_leaf"][2], java, variant))
            results = compile_java(bundle, work, owners) if java else compile_c(bundle, work, owners, zig)
            for result in results:
                assert result.stdout == truth, (java, variant, "scalar continuation changed")
                if variant == "plain":
                    assert result.stderr == "\n" * 10
                elif variant == "traced":
                    assert result.stderr == traces, (java, result.stderr, traces)
                else:
                    assert result.stderr != traces, (java, variant, "trace mutation escaped")
    export(java_bundle, c_bundle, root)
    print("Three checked Rust crates: native void ABI, scalar continuation, exact effect traces; drop/duplicate/reorder faults detected")

if __name__ == "__main__":
    main()
