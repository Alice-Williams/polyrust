//! Observe owned argument and return places before enabling call certificates.
pub fn produce(value: i32) -> Box<i32> {
    Box::new(value)
}
pub fn produce_return(value: i32) -> Box<i32> {
    let owner = Box::new(value);
    return owner;
}
pub fn consume(owner: Box<i32>) -> i32 {
    let moved = owner;
    *moved
}
pub fn relay(owner: Box<i32>) -> Box<i32> {
    let moved = owner;
    moved
}
pub fn via_producer(value: i32) -> i32 {
    let owner = produce(value);
    *owner
}
pub fn via_consumer(value: i32) -> i32 {
    let owner = Box::new(value);
    consume(owner)
}
pub fn via_relay(value: i32) -> i32 {
    let owner = Box::new(value);
    let returned = relay(owner);
    *returned
}
use produce as build;
pub fn aliased(value: i32) -> i32 {
    let owner = build(value);
    *owner
}
