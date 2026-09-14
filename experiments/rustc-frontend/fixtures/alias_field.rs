type Score = i32;
struct Ticket {
    value: Score,
}
pub fn score(input: i32) -> i32 {
    let value = Ticket { value: input };
    value.value
}
