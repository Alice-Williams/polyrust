#[path = "owned_calls.rs"]
mod source;
pub fn producer_moved(value: i32) -> i32 {
    let owner = source::produce_moved(value);
    let moved = owner;
    *moved
}
pub fn consumer_direct(value: i32) -> i32 {
    let owner = Box::new(value);
    source::consume_direct(owner)
}
pub fn relay_direct(value: i32) -> i32 {
    let owner = Box::new(value);
    let moved = owner;
    let returned = source::relay_direct(moved);
    *returned
}
fn replacement(owner: Box<i32>) -> Box<i32> {
    Box::new(*owner)
}
pub fn unproven(value: i32) -> i32 {
    let owner = Box::new(value);
    let returned = replacement(owner);
    *returned
}
fn recursive(owner: Box<i32>) -> Box<i32> {
    recursive(owner)
}
pub fn cycle(value: i32) -> i32 {
    let owner = Box::new(value);
    let returned = recursive(owner);
    *returned
}
pub fn extra(value: i32) -> i32 {
    let owner = source::produce(value);
    let returned = source::relay(owner);
    *returned
}
pub fn nested(value: i32) -> i32 {
    let owner = { source::produce(value) };
    *owner
}
pub fn borrowed(value: i32) -> i32 {
    let owner = source::produce(value);
    let reference = &owner;
    **reference
}
pub fn mutable(value: i32) -> i32 {
    let mut owner = source::produce(value);
    *owner = value;
    *owner
}
pub fn expression(value: i32) -> i32 {
    let owner = source::produce(value + 1);
    *owner
}
pub fn conditional(value: i32) -> i32 {
    let owner = if value == 0 {
        source::produce(value)
    } else {
        source::produce_return(value)
    };
    *owner
}
pub fn extra_owner(value: i32) -> i32 {
    let owner = Box::new(value);
    let returned = source::produce(value);
    *returned + *owner
}
fn long_relay(owner: Box<i32>) -> Box<i32> {
    source::relay(owner)
}
pub fn longer_chain(value: i32) -> i32 {
    let owner = Box::new(value);
    let returned = long_relay(owner);
    *returned
}
pub fn scalar_binding(value: i32) -> i32 {
    let owner = Box::new(value);
    let result = source::consume(owner);
    result
}
pub fn nested_exit(value: i32) -> i32 {
    let owner = source::produce(value);
    { *owner }
}
fn wrong_producer(_value: i32) -> Box<i32> {
    Box::new(3)
}
pub fn wrong_producer_entry(value: i32) -> i32 {
    let owner = wrong_producer(value);
    *owner
}
fn wrong_consumer(_owner: Box<i32>) -> i32 {
    3
}
pub fn wrong_consumer_entry(value: i32) -> i32 {
    let owner = Box::new(value);
    wrong_consumer(owner)
}
