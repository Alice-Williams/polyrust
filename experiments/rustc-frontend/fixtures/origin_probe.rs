mod hidden {
    /// Hidden ticket docs.
    pub struct Ticket {
        pub value: i32,
    }
}

/// Public ticket docs.
pub struct Ticket {
    value: i32,
}

/// Score docs.
pub fn score(input: i32) -> i32 {
    let first = hidden::Ticket { value: input };
    let second = Ticket { value: first.value };
    let moved = second;
    let borrowed = &moved;
    if borrowed.value > 10 {
        borrowed.value
    } else {
        10
    }
}
