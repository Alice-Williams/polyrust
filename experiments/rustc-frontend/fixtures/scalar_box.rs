//! Source operation identity, not whole-body payload ownership.
mod scalar_box_limits;
pub struct Record {
    number: i32,
    enabled: bool,
}
type Alias = Record;
use Record as Renamed;
pub fn mixed(a: i32, flag: bool) -> Box<Record> {
    Box::new(Record {
        number: a,
        enabled: flag,
    })
}
pub fn alias(a: i32) -> Box<Alias> {
    Box::new(Alias {
        enabled: true,
        number: a,
    })
}
pub fn renamed(a: i32) -> Box<Renamed> {
    Box::new(Renamed {
        number: a,
        enabled: false,
    })
}
pub fn parameter(record: Record) -> Box<Record> {
    Box::new(record)
}
pub fn qualified(record: Record) -> Box<Record> {
    std::boxed::Box::new(record)
}
pub fn renamed_box(record: Record) -> Box<Record> {
    use std::boxed::Box as Heap;
    Heap::new(record)
}
pub mod left {
    pub struct Record {
        value: i32,
    }
    pub fn wrap(record: Record) -> Box<Record> {
        Box::new(record)
    }
}
pub mod right {
    pub struct Record {
        value: i32,
    }
    pub fn wrap(record: Record) -> Box<Record> {
        Box::new(record)
    }
}
pub struct Boolean {
    value: bool,
}
pub fn boolean(record: Boolean) -> Box<Boolean> {
    Box::new(record)
}
pub fn scalar(value: i32) -> Box<i32> {
    Box::new(value)
}
pub struct Owned {
    value: Box<i32>,
}
pub fn owned(value: Owned) -> Box<Owned> {
    Box::new(value)
}
pub fn old_record(value: i32) -> Owned {
    Owned {
        value: Box::new(value),
    }
}
pub struct Nested {
    record: Record,
}
pub fn nested(value: Nested) -> Box<Nested> {
    Box::new(value)
}
pub struct Unsigned {
    value: u32,
}
pub fn unsigned(value: Unsigned) -> Box<Unsigned> {
    Box::new(value)
}
pub struct Generic<T> {
    value: T,
}
pub fn generic(value: Generic<i32>) -> Box<Generic<i32>> {
    Box::new(value)
}
pub struct Empty {}
pub fn empty(value: Empty) -> Box<Empty> {
    Box::new(value)
}
pub struct Unit;
pub fn unit(value: Unit) -> Box<Unit> {
    Box::new(value)
}
pub struct Tuple(i32);
pub fn tuple(value: Tuple) -> Box<Tuple> {
    Box::new(value)
}
pub enum Choice {
    Value { value: i32 },
}
pub fn enumeration(value: Choice) -> Box<Choice> {
    Box::new(value)
}
pub union Union {
    value: i32,
}
pub fn union_record(value: Union) -> Box<Union> {
    Box::new(value)
}
pub fn external(value: std::time::Duration) -> Box<std::time::Duration> {
    Box::new(value)
}
pub struct Custom {
    value: i32,
}
impl Drop for Custom {
    fn drop(&mut self) {}
}
pub fn custom(value: Custom) -> Box<Custom> {
    Box::new(value)
}
#[repr(C)]
pub struct Repr {
    value: i32,
}
pub fn representation(value: Repr) -> Box<Repr> {
    Box::new(value)
}
pub fn indirect(record: Record) -> Box<Record> {
    let make = Box::new;
    make(record)
}
pub fn block_callee(record: Record) -> Box<Record> {
    (if true { Box::new } else { Box::new })(record)
}
pub fn new(record: Record) -> Box<Record> {
    Box::new(record)
}
pub fn wrapper(record: Record) -> Box<Record> {
    new(record)
}
pub mod imitation {
    pub struct Box<T>(pub T);
    impl<T> Box<T> {
        pub fn new(value: T) -> Self {
            Self(value)
        }
    }
}
pub fn counterfeit(record: Record) -> imitation::Box<Record> {
    imitation::Box::new(record)
}
pub trait Marker {}
impl Marker for Record {}
pub fn adjusted(record: Record) -> Box<dyn Marker> {
    Box::new(record)
}
