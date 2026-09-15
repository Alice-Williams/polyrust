"""Exhaustive Boolean truth and call traces, independent of generated metadata."""
from itertools import product

NAMES = ["and", "or", "xor", "nested", "lazy_left", "lazy_right", "lazy_outer",
         "eager_outer", "field", "shared", "imported", "call_args", "condition"]


def expected(variant="traced"):
    values, traces = [], []
    for a, b, c in product([False, True], repeat=3):
        # Explicit truth operations; trace expectations independently model order.
        row = [a and b, a or b, a != b, (not (a and b)) != c,
               (a and b) or c, a and (b or c), a and (b != c),
               (a or b) and c, (b != c) if a else b, (not a) and b,
               (a and b) != c, (a and b) and c, c if a != b else not c]
        trace = ["AB", "AB", "AB", "ABC", "A" + ("B" if a else "") + "C",
                 "AB" + ("C" if not b else ""), "A" + ("BC" if a else ""),
                 "AB" + ("C" if a or b else ""), "", "", "ABDC", "ABCD", "ABC"]
        if variant == "lazy":
            trace[0], trace[1] = "A" + ("B" if a else ""), "A" + ("B" if not a else "")
        elif variant == "duplicate":
            trace[:3] = ["ABB"] * 3
        elif variant == "reordered":
            trace[:3] = ["BA"] * 3
        elif variant == "wrong":
            row[:3] = [a or b, a != b, a and b]
        elif variant == "plain":
            trace = [""] * len(NAMES)
        else:
            assert variant == "traced"
        values.extend(str(int(value)) for value in row)
        traces.extend(trace)
    return "\n".join(values) + "\n", "\n".join(traces) + "\n"
