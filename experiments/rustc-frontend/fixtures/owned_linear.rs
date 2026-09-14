//! Closed linear ownership shapes and valid-Rust exclusions.
pub fn zero(value: i32) -> i32 {
    let owner = Box::new(value);
    *owner
}
pub fn one(value: i32) -> i32 {
    let owner = Box::new(value);
    let next = owner;
    *next
}
pub fn many(value: i32) -> i32 {
    let first = Box::new(value);
    let second = first;
    let third = second;
    let fourth = third;
    *fourth
}
pub fn shadow(value: i32) -> i32 {
    let value = Box::new(value);
    let value = value;
    *value
}
pub fn renamed(input: i32) -> i32 {
    let completely_different = Box::new(input);
    *completely_different
}
pub fn bad_extra(value: i32) -> i32 {
    let first = Box::new(value);
    let second = Box::new(value);
    *first + *second
}
pub fn bad_literal(_value: i32) -> i32 {
    let owner = Box::new(7);
    *owner
}
pub fn bad_early(value: i32) -> i32 {
    let owner = Box::new(value);
    return *owner;
}
pub fn bad_branch(value: i32) -> i32 {
    let owner = Box::new(value);
    if value > 0 { *owner } else { *owner }
}
pub fn bad_unrelated(value: i32) -> i32 {
    let scalar = value;
    let owner = Box::new(scalar);
    *owner
}
pub fn bad_parameter(value: i32, _other: i32) -> i32 {
    let owner = Box::new(value);
    *owner
}
pub fn bad_payload(value: i32) -> i32 {
    let owner = Box::new(value as u32);
    *owner as i32
}
