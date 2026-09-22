"""Measured actual generated calls and compiling width/order corruption controls."""
import re
from dataclasses import dataclass

from character_source_inventory import COMPARISONS
from character_source_truth import LITERAL_PREFIX, OUTPUT, cases, row, trace
from character_oracle import flags


@dataclass(frozen=True)
class Function:
    parameters: str
    body: str
    start: int
    end: int


def function(text, symbol):
    matches = list(re.finditer(r"\b" + re.escape(symbol) + r"\(([^)]*)\)\s*\{", text))
    assert len(matches) == 1, symbol
    header = matches[0]
    depth = 1
    # Test-only body location: aggregate initializers contain nested braces.
    # Skip quoted tokens so instrumentation strings cannot affect balancing.
    tokens = r'"(?:\\.|[^"\\])*"|\x27(?:\\.|[^\x27\\])*\x27|[{}]'
    for token in re.finditer(tokens, text[header.end():]):
        if token[0] == "{":
            depth += 1
        elif token[0] == "}":
            depth -= 1
            if depth == 0:
                end = header.end() + token.start()
                return Function(header[1], text[header.end():end], header.end(), end)
    raise AssertionError(("missing function end", symbol))


def modify_body(text, symbol, change):
    match = function(text, symbol)
    body = change(match.body)
    assert body != match.body, symbol
    return text[:match.start] + body + text[match.end:]


def instrument(directory, language, evidence):
    _, owners, c, java, bindings, cf, jf = evidence
    leaf = owners["leaf"]
    apis = c if language == "c" else java
    source = directory / apis[leaf]["implementation" if language == "c" else "source"]
    original = source.read_text()
    text = original
    identity = bindings[leaf, "value", "identity"]
    signatures = c[leaf]["source_types"]["functions"]
    marker, = [f["id"] for f in signatures if f["parameters"] == ["i32"] and f["result"] == "i32"]
    inserted = []
    for id_, label in [(identity, "I"), (marker, "M")]:
        symbol = cf[id_]["symbol"] if language == "c" else jf[id_]["target"]["path"]["member"]
        match = function(text, symbol)
        argument = match.parameters.split()[-1]
        if language == "c":
            format_ = "PRIu32" if label == "I" else "PRId32"
            addition = f'\n    (void) fprintf(stderr, "{label}:%" {format_} ";", {argument});'
        else:
            addition = f'\n    System.err.print("{label}:" + {argument} + ";");'
        text = text[:match.start] + addition + text[match.start:]
        inserted.append(addition)
    prefix = '#include <stdio.h>\n#include <inttypes.h>\n' if language == "c" else ""
    restored = text
    for addition in inserted:
        assert restored.count(addition) == 1
        restored = restored.replace(addition, "")
    assert restored == original
    source.write_text(prefix + text)


def mutate(directory, language, evidence, variant):
    _, owners, c, java, bindings, cf, jf = evidence
    leaf = owners["leaf"]
    source = directory / (c[leaf]["implementation"] if language == "c" else java[leaf]["source"])
    before = text = source.read_text()
    def symbol(id_):
        return cf[id_]["symbol"] if language == "c" else jf[id_]["target"]["path"]["member"]
    if variant in ["byte", "utf16"]:
        mask = 255 if variant == "byte" else 65535
        def narrow(body):
            body, count = re.subn(r"\breturn\s+([^;]+);", lambda m: f"return ({m[1]} & {mask});", body)
            assert count == 1
            return body
        text = modify_body(text, symbol(bindings[leaf, "value", "identity"]), narrow)
    elif variant == "reverse":
        compare = bindings[leaf, "type", "compare"]
        for member in COMPARISONS[2:]:
            def reverse(body):
                result, count = re.subn(r"<=|>=|<|>", lambda m: {"<": ">", ">": "<", "<=": ">=", ">=": "<="}[m[0]], body)
                assert count == 1
                return result
            text = modify_body(text, symbol(bindings[compare, "value", member]), reverse)
    else:
        assert variant == "field_order"
        def reordered(body):
            # Move the actual independent field evaluation before the other.
            # Preserve both bodies and their once-only calls, not a fake trace.
            lines = body.splitlines(keepends=True)
            calls = [i for i, line in enumerate(lines)
                     if (" = " in line and ("poly_fn_" in line if language == "c" else ".fn" in line))]
            assert len(calls) == 2
            start = calls[0] - 1
            end = calls[1] + (2 if language == "java" else 1)
            middle = calls[1] - 1
            assert start >= 0 and start < middle < end
            return "".join(lines[:start] + lines[middle:end] + lines[start:middle] + lines[end:])
        text = modify_body(text, symbol(bindings[leaf, "value", "record_character"]), reordered)
    assert text != before
    source.write_text(text)


def expected(variant):
    output = bytearray(LITERAL_PREFIX)
    traces = bytearray()
    for left, right, marker in cases(False):
        values = list(row(left, right, marker))
        observed = trace(left, right, marker)
        if variant in ["byte", "utf16"]:
            mask = 255 if variant == "byte" else 65535
            for index in [0, 1, 2, 3, 4, 5]:
                values[index] &= mask
            values[8] = int((left & mask) < (right & mask))
            identity, other, field = f"I:{left};", f"I:{right};", f"M:{marker};"
            narrowed = f"I:{left & mask};"
            observed = (identity * 3 + (identity + other) * 2
                        + identity + field + narrowed
                        + identity + narrowed + field + identity + other).encode()
        elif variant == "reverse":
            values[7] = flags(right, left)
            values[8] = int(left > right)
        elif variant == "field_order":
            identity, field = f"I:{left};".encode(), f"M:{marker};".encode()
            # Distinct source row; only record_character's field order changes.
            prefix = (identity * 3 + (identity + f"I:{right};".encode()) * 2 + identity)
            assert observed.startswith(prefix + field + identity)
            observed = prefix + identity + field + observed[len(prefix + field + identity):]
        else:
            assert variant == "trace"
        output.extend(OUTPUT.pack(*values))
        traces.extend(observed)
    return bytes(output), bytes(traces)
