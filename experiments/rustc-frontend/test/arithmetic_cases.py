"""Deterministic category/boundary cross-product and unbiased bit-pair corpus."""
from arithmetic_oracle import SIGN

BOUNDARIES = [
    0, 1, 2, 3, 0x0007ffffffffffff, 0x0008000000000000,
    0x000fffffffffffff, 0x0010000000000000, 0x0010000000000001,
    0x3c90000000000000, 0x3ca0000000000000, 0x3fe0000000000000,
    0x3fefffffffffffff, 0x3ff0000000000000, 0x3ff0000000000001,
    0x3ff0000000000002, 0x4000000000000000, 0x4008000000000000,
    0x4330000000000000, 0x7c90000000000000, 0x7feffffffffffffe,
    0x7fefffffffffffff, 0x7ff0000000000000, 0x7ff0000000000001,
    0x7ff8000000000000, 0x7fffffffffffffff,
]
BOUNDARIES += [bits | SIGN for bits in BOUNDARIES]


def random_pairs():
    state = 0x17624a392ef58bc1
    for _ in range(512):
        pair = []
        for _ in range(2):
            state = (state * 6364136223846793005 + 1442695040888963407) & ((1 << 64) - 1)
            pair.append(state)
        yield tuple(pair)


PAIRS = [(left, right) for left in BOUNDARIES for right in BOUNDARIES] + list(random_pairs())
