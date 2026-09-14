struct Ticket {
    value: i32,
}
type Alias = Ticket;
pub fn score(input: i32) -> i32 {
    let value = Alias { value: input };
    value.value
}
