"""Compiling test-only body faults, independent of production rendering."""
import re
from short_circuit_mutations import definition

def mutate(text, member, marker, java, variant):
    match, start, end = definition(text, member)
    p = re.search(r"\b(\w+)\s*$", match.group(1)).group(1)
    call = lambda name: f"{'java.lang.Math.' if java else ''}{name}({p})"
    trunc = f"({p} < 0.0 ? {call('ceil')} : {call('floor')})" if java else call("trunc")
    if variant in ["floor", "ceil"]:
        body = f"\nreturn {call(variant)};\n"
    elif variant == "zero_sign":
        body = f"\nreturn {trunc} == 0.0 ? 0.0 : {trunc};\n"
    elif variant == "dropped":
        body = f"\nreturn {trunc};\n"
    else:
        assert variant == "duplicated"
        prefix = "" if java else "(void)"
        body = f"\n{prefix}{marker}({p});\n" + text[start:end]
    return text[:start] + body + text[end:]
