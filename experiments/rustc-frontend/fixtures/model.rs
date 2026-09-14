pub struct Ticket {
    value: i32,
}

pub fn score(input: i32) -> i32 {
    let original = Ticket { value: input };
    let moved = original;
    let borrowed = &moved;
    if borrowed.value > 10 {
        borrowed.value
    } else {
        10
    }
}
