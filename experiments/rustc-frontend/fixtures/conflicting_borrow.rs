pub fn score(input: i32) -> i32 {
    let mut value = input;
    let borrowed = &value;
    value = 99;
    *borrowed + value
}
