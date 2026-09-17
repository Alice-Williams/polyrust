"""Guard the original ten omission fixtures against unrelated missing slots.

This audits fixed handwritten fixtures; rustc remains the actual type checker.
"""
from pathlib import Path
import re
import sys

ORIGINAL = {
    "literal_values", "scalar_comparisons", "resolved_places", "shared_borrows",
    "object_types", "record_initializers", "lexical_control", "entry_signatures",
    "direct_calls", "function_signatures",
}


def slots(text):
    found = re.findall(r"register!\(M;\s*(\w+),", text)
    assert len(found) == len(set(found)) and ORIGINAL <= set(found)
    return set(found)


def check(text, registered):
    chains = re.findall(r"Builder::new\(\)(.*?)\.build\(\);", text, re.S)
    assert len(chains) == len(ORIGINAL)
    omitted = []
    for chain in chains:
        calls = re.findall(r"\.(\w+)\(", chain)
        assert len(calls) == len(set(calls))
        assert set(calls) <= registered
        missing = registered - set(calls)
        assert len(missing) == 1, missing
        omitted.extend(missing)
    assert set(omitted) == ORIGINAL


def main():
    for fixture, builder in zip(sys.argv[1::2], sys.argv[2::2], strict=True):
        text, registered = Path(fixture).read_text(), slots(Path(builder).read_text())
        check(text, registered)
        for mutation in [
            text.replace(".floating_arithmetic(Arithmetic)", ""),
            text.replace(".floating_arithmetic(Arithmetic)", ".floating_arithmetic(Arithmetic)" * 2, 1),
        ]:
            assert mutation != text
            try:
                check(mutation, registered)
            except AssertionError:
                pass
            else:
                raise AssertionError("unrelated omission/duplicate escaped")
    print("20 legacy omission chains each omit exactly one intended capability; four drift faults reject")


if __name__ == "__main__":
    main()
