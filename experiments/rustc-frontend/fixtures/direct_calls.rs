//! Calls retain module identity, scalar signatures and lexical evaluation order.
#[path = "direct_calls_out.rs"]
mod second;

mod first {
    /// Same source spelling as second::select, deliberately different behavior.
    pub(super) fn select(flag: bool, left: i32, right: i32) -> i32 {
        if flag { left } else { right }
    }
}

use second::echo as alias;

struct Pair {
    first: i32,
    second: i32,
}

fn zero() -> i32 {
    0
}

fn negative(input: i32) -> bool {
    input < zero()
}

/// Public entry; every helper remains private or restricted in Rust.
pub fn score(input: i32) -> i32 {
    // Source field order intentionally differs from declaration order.
    let pair = Pair {
        second: second::select(negative(alias(input)), input, zero()),
        first: first::select(negative(input), input, zero()),
    };
    let borrowed = &pair;
    if negative(alias(input)) {
        first::select(false, borrowed.first, alias(borrowed.second))
    } else {
        second::select(
            false,
            first::select(true, borrowed.second, zero()),
            borrowed.first,
        )
    }
}
