"""Compiling fixture faults; production packages are never modified."""
import re
from short_circuit_mutations import definition


def mutate(text, entry, reciprocal, java, variant):
    match, start, end = definition(text, entry)
    parameter, = re.findall(r"\b(\w+)\s*$", match.group(1))
    if variant in ["zero_sign", "unguarded", "flipped"]:
        expression = {
            "zero_sign": f"{parameter} == 0.0",
            "unguarded": f"{reciprocal}({parameter}) < 0.0",
            "flipped": f"{parameter} == 0.0 && {reciprocal}({parameter}) > 0.0",
        }[variant]
        # Keep the private definition referenced for strict C unused-function lint.
        unused = "" if java or variant != "zero_sign" else f"(void)&{reciprocal};\n"
        return text[:start] + f"\n{unused}return {expression};\n" + text[end:]
    body = text[start:end]
    prefix = "" if java else "(void)"
    pattern = r"(?:\b\w+\.)*\b" + re.escape(reciprocal.split(".")[-1]) + r"\((\w+)\)"
    calls = list(re.finditer(pattern, body))
    assert len(calls) == 1
    call = calls[0]
    if variant == "eager":
        # Move the actual call out of the guarded branch, preserving its result.
        value = "negative_zero_eager"
        replacement = value if java else f"((void){call.group(1)}, {value})"
        body = body[:call.start()] + replacement + body[call.end():]
        body = f"\n{'final ' if java else ''}double {value} = {reciprocal}({parameter});\n" + body
    else:
        assert variant == "duplicate"
        line = body.rfind("\n", 0, call.start()) + 1
        body = body[:line] + f"{prefix}{call.group(0)};\n" + body[line:]
    return text[:start] + body + text[end:]
