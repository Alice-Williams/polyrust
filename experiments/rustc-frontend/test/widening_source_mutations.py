"""Mutate actual generated cast/call statements, retaining unrelated body bytes."""
import re
from short_circuit_mutations import definition

VARIANTS = ["plain", "traced", "zero_extend", "narrow", "zero", "dropped", "duplicated"]


def mutate(text, entry, member, java, variant):
    _, start, end = definition(text, entry)
    body = text[start:end]
    declarations = list(re.finditer(r"^[ \t]*(?:final )?(?:int|int32_t)\s+(\w+)\s*=\s*([^;]+);", body, re.MULTILINE))
    assert len(declarations) == 3, body
    argument, call, temporary = declarations
    assert call.group(2) == f"{member}({argument.group(1)})" and temporary.group(2) == call.group(1)
    if variant == "dropped":
        body = body[:call.start(2)] + argument.group(1) + body[call.end(2):]
    elif variant == "duplicated":
        extra = ("" if java else "(void)") + call.group(2) + ";\n"
        body = body[:call.start()] + extra + body[call.start():]
    else:
        matches = list(re.finditer(r"return\s+([^;]+);", body))
        match, = matches
        value = temporary.group(1)
        assert re.fullmatch(r"\(*\((?:long|int64_t)\)\s*\(*" + re.escape(value) + r"\)*", match.group(1)), body
        if variant == "zero_extend":
            changed = f"((long){value}) & 4294967295L" if java else f"(int64_t)(uint32_t){value}"
        elif variant == "narrow":
            changed = (f"(long)(short){value}" if java else
                       f"((uint32_t){value} & 65535U) <= 32767U ? (int64_t)((uint32_t){value} & 65535U) : -1 - (int64_t)(65535U - ((uint32_t){value} & 65535U))")
        else:
            assert variant == "zero"
            changed = f"(long)({value} & 0)" if java else f"(int64_t)({value} & 0)"
        body = body[:match.start(1)] + changed + body[match.end(1):]
    assert body != text[start:end]
    return text[:start] + body + text[end:]
