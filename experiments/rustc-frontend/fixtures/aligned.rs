#[repr(align(64))]
struct Ticket {
    value: i32,
}
pub fn score(input: i32) -> i32 {
    let value = Ticket { value: input };
    value.value
}
