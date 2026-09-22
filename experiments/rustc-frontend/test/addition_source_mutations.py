"""Compiling fixed-fixture faults distinguish values from ordered call effects."""
import re
from short_circuit_mutations import definition

VARIANTS = ["plain", "traced", "carryless", "wrong_operand", "narrow", "dropped", "duplicated", "reversed"]

def variants(operation):
    assert operation in ["addition", "subtraction", "multiplication"]
    if operation == "multiplication":
        return ["plain", "traced", "add", "wrong_operand", "narrow", "dropped", "duplicated", "reversed"]
    return (VARIANTS if operation == "addition" else
            ["plain", "traced", "add", "reverse_values", "wrong_operand", "narrow", "dropped", "duplicated", "reversed"])


def mutate(text, entry, left, right, width, java, variant, operation="addition"):
    assert variant in variants(operation)
    _, start, end = definition(text, entry)
    body = text[start:end]
    declarations = list(re.finditer(r"^[ \t]*(?:final )?(?:int|long|int32_t|int64_t)\s+(\w+)\s*=\s*([^;]+);", body, re.MULTILINE))
    assert len(declarations) == 6, (entry, body)
    a0, a1, a2, b0, b1, b2 = declarations
    for argument, call, temporary, member in [(a0, a1, a2, left), (b0, b1, b2, right)]:
        assert call.group(2) == f"{member}({argument.group(1)})"
        assert temporary.group(2) == call.group(1)
    if variant == "dropped":
        body = body[:a1.start(2)] + a0.group(1) + body[a1.end(2):]
    elif variant == "duplicated":
        statement = ("" if java else "(void)") + a1.group(2) + ";\n"
        body = body[:a1.start()] + statement + body[a1.start():]
    elif variant == "reversed":
        body = body[:a0.start()] + body[b0.start():b2.end()] + body[a2.end():b0.start()] + body[a0.start():a2.end()] + body[b2.end():]
    elif variant in ["carryless", "wrong_operand", "add", "reverse_values"]:
        pattern = r"return\s+([^;]+);" if java else r"uint" + str(width) + r"_t\s+\w+\s*=\s*([^;]+);"
        matches = list(re.finditer(pattern, body))
        assert len(matches) == 1
        match, = matches
        old = match.group(1)
        operator = {"addition": "+", "subtraction": "-", "multiplication": "*"}[operation]
        assert old.count(operator) == 1 and old.count(a2.group(1)) == old.count(b2.group(1)) == 1
        if variant == "carryless":
            changed = old.replace("+", "^")
        elif variant == "add":
            changed = old.replace(operator, "+")
        elif variant == "reverse_values":
            replacements = {a2.group(1): b2.group(1), b2.group(1): a2.group(1)}
            pattern = r"\b(?:" + "|".join(map(re.escape, replacements)) + r")\b"
            changed, count = re.subn(pattern, lambda m: replacements[m[0]], old)
            assert count == 2
        else:
            changed = re.sub(r"\b" + re.escape(b2.group(1)) + r"\b", a2.group(1), old)
        body = body[:match.start(1)] + changed + body[match.end(1):]
        if variant == "wrong_operand" and not java:
            body = body[:b2.end()] + f"\n(void){b2.group(1)};" + body[b2.end():]
    else:
        assert variant == "narrow"
        matches = list(re.finditer(r"return\s+([^;]+);", body))
        assert len(matches) == 1
        match, = matches
        if java:
            changed = f"return ({'short' if width == 32 else 'int'})({match.group(1)});"
        else:
            mask, maximum = (1 << (width // 2)) - 1, (1 << (width // 2 - 1)) - 1
            suffix = "U" if width == 32 else "ULL"
            changed = (f"int{width}_t fullResult = {match.group(1)};\n"
                       f"uint{width}_t narrowResult = ((uint{width}_t)fullResult) & {mask}{suffix};\n"
                       f"return narrowResult <= {maximum}{suffix} ? (int{width}_t)narrowResult : "
                       f"-1 - (int{width}_t)({mask}{suffix} - narrowResult);")
        body = body[:match.start()] + changed + body[match.end():]
    assert body != text[start:end]
    return text[:start] + body + text[end:]
