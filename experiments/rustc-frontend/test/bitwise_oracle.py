"""Exact integer truth, independent of compiler output and metadata."""
NAMES = ["invert", "and", "or", "xor", "nested", "field", "shared", "imported"]


def values(width):
    mask = (1 << width) - 1
    def signed(value):
        return value if value < (1 << (width - 1)) else value - (1 << width)
    bits = {0, mask, 1 << (width - 1), (1 << (width - 1)) - 1,
            int("55" * (width // 8), 16), int("aa" * (width // 8), 16)}
    for bit in range(width):
        bits.update([1 << bit, mask ^ (1 << bit)])
    return sorted(signed(value) for value in bits)


def cases():
    # Each individual bit is exercised against each other bit and its inverse.
    return [(width, a, b) for width in [32, 64]
            for a in values(width) for b in values(width)]


def results(a, b):
    return [~a, a & b, a | b, a ^ b, ~(a & b) ^ (a | b), ~a ^ b, ~a & b, ~a | b]


def expected(rows):
    return "".join(" ".join(map(str, results(a, b))) + "\n" for _, a, b in rows)


def expected_traces(rows, reordered=False):
    # Single-byte probes: A/B are i32 left/right, C/D are i64 left/right.
    def row(width):
        left, right = ("A", "B") if width == 32 else ("C", "D")
        pair = right + left if reordered else left + right
        return left + pair * 3 + left + right + "\n"
    return "".join(row(width) for width, _, _ in rows)
