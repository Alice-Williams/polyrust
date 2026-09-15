//! Direct local-call identity acceptance and rejection fixtures.
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

pub fn qualified(value: i32) -> Box<i32> {
    crate::produce(value)
}
pub fn result_adjusted(value: i32) -> Box<dyn std::fmt::Debug> {
    produce(value)
}
pub fn reads_ref(owner: &i32) -> i32 {
    *owner
}
pub fn argument_adjusted(owner: Box<i32>) -> i32 {
    reads_ref(&owner)
}
pub fn indirect(value: i32) -> Box<i32> {
    let f = produce;
    f(value)
}
pub fn computed(value: i32) -> Box<i32> {
    (if value > 0 { produce } else { produce_return })(value)
}
pub fn closure(value: i32) -> Box<i32> {
    let f = |v| produce(v);
    f(value)
}
pub fn external(owner: Box<i32>) -> Box<i32> {
    std::convert::identity(owner)
}
pub fn generic<const N: usize>(owner: Box<i32>) -> Box<i32> {
    owner
}
pub fn call_generic(owner: Box<i32>) -> Box<i32> {
    generic::<1>(owner)
}
pub fn pair(first: i32, second: i32) -> Box<i32> {
    Box::new(first + second)
}
pub fn wrong_arity(value: i32) -> Box<i32> {
    pair(value, value)
}
pub fn plain(value: i32) -> i32 {
    value
}
pub fn wrong_result(value: i32) -> i32 {
    plain(value)
}
pub extern "C" fn foreign(value: i32) -> Box<i32> {
    Box::new(value)
}
pub fn call_foreign(value: i32) -> Box<i32> {
    foreign(value)
}
pub fn unsigned(value: u32) -> Box<u32> {
    Box::new(value)
}
pub fn wrong_payload(value: u32) -> Box<u32> {
    unsigned(value)
}
pub mod left {
    pub fn produce(value: i32) -> Box<i32> {
        Box::new(value)
    }
}
pub mod right {
    pub fn produce(value: i32) -> Box<i32> {
        Box::new(value)
    }
}
pub fn same_named_left(value: i32) -> Box<i32> {
    left::produce(value)
}
pub fn same_named_right(value: i32) -> Box<i32> {
    right::produce(value)
}
// The signature matches Relay, but its body replaces the allocation.
// I-01 authenticates this call identity; it must not infer relay effects.
pub fn replacement(owner: Box<i32>) -> Box<i32> {
    Box::new(*owner)
}
pub fn unproven(owner: Box<i32>) -> Box<i32> {
    replacement(owner)
}
pub struct Maker;
impl Maker {
    pub fn make(value: i32) -> Box<i32> {
        Box::new(value)
    }
    pub fn instance(&self, value: i32) -> Box<i32> {
        Box::new(value)
    }
}
pub fn associated(value: i32) -> Box<i32> {
    Maker::make(value)
}
pub fn method(value: i32) -> Box<i32> {
    Maker.instance(value)
}
pub fn pointer(value: i32) -> Box<i32> {
    let function: fn(i32) -> Box<i32> = produce;
    function(value)
}
