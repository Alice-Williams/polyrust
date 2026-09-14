pub struct Ticket {
    value: i32,
    negative: bool,
}

pub fn score(input: i32) -> i32 {
    let original = Ticket {
        value: input,
        negative: input < 0,
    };
    let moved = original;
    let borrowed = &moved;
    if borrowed.negative {
        -7
    } else if borrowed.value == 10 {
        99
    } else {
        borrowed.value
    }
}
