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
pub fn produce_moved(value: i32) -> Box<i32> {
    let owner = Box::new(value);
    let moved = owner;
    moved
}
pub fn consume_return(owner: Box<i32>) -> i32 {
    let moved = owner;
    return *moved;
}
pub fn consume_direct(owner: Box<i32>) -> i32 {
    *owner
}
pub fn relay_return(owner: Box<i32>) -> Box<i32> {
    let moved = owner;
    return moved;
}
pub fn relay_direct(owner: Box<i32>) -> Box<i32> {
    owner
}
pub fn via_consumer_return(value: i32) -> i32 {
    let owner = Box::new(value);
    let moved = owner;
    return consume_return(moved);
}
pub fn via_relay_return(value: i32) -> i32 {
    let owner = Box::new(value);
    let returned = relay_return(owner);
    let moved = returned;
    return *moved;
}
pub fn via_producer_return(value: i32) -> i32 {
    let owner = produce_return(value);
    let moved = owner;
    return *moved;
}
