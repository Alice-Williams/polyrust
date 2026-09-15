mod record_limit_128;
mod record_limit_129;
pub fn external(a: i32, b: i32) -> std::ops::Range<Box<i32>> {
    std::ops::Range {
        start: Box::new(a),
        end: Box::new(b),
    }
}
pub struct Record {
    first: Box<i32>,
    second: Box<i32>,
}
type Alias = Record;
use Record as Renamed;

pub fn forward(a: i32, b: i32) -> Record {
    Record {
        first: Box::new(a),
        second: Box::new(b),
    }
}
pub fn reversed(a: i32, b: i32) -> Record {
    Record {
        second: Box::new(b),
        first: Box::new(a),
    }
}
pub fn alias(a: i32, b: i32) -> Alias {
    Alias {
        first: Box::new(a),
        second: Box::new(b),
    }
}
pub fn renamed(a: i32, b: i32) -> Renamed {
    Renamed {
        first: Box::new(a),
        second: Box::new(b),
    }
}
pub fn locals(a: i32, b: i32) -> Record {
    let first = Box::new(a);
    let second = Box::new(b);
    Record { second, first }
}
pub struct Single {
    owned: Box<i32>,
}
pub fn single(a: i32) -> Single {
    Single { owned: Box::new(a) }
}
pub mod left {
    pub struct Record {
        first: Box<i32>,
        second: Box<i32>,
    }
    pub fn make(a: i32, b: i32) -> Record {
        Record {
            first: Box::new(a),
            second: Box::new(b),
        }
    }
}
pub mod right {
    pub struct Record {
        first: Box<i32>,
        second: Box<i32>,
    }
    pub fn make(a: i32, b: i32) -> Record {
        Record {
            first: Box::new(a),
            second: Box::new(b),
        }
    }
}
pub struct Generic<T> {
    first: T,
}
pub fn generic(a: i32) -> Generic<Box<i32>> {
    Generic { first: Box::new(a) }
}
pub struct Scalar {
    first: i32,
}
pub fn scalar(a: i32) -> Scalar {
    Scalar { first: a }
}
pub struct Unsigned {
    first: Box<u32>,
}
pub fn unsigned(a: u32) -> Unsigned {
    Unsigned { first: Box::new(a) }
}
pub struct Empty {}
pub struct Unit;
pub fn unit() -> Unit {
    Unit {}
}
pub union Union {
    first: std::mem::ManuallyDrop<Box<i32>>,
}
pub fn union_record(a: i32) -> Union {
    Union {
        first: std::mem::ManuallyDrop::new(Box::new(a)),
    }
}
// Supplies an authentic compiler type for the allocator-identity oracle.
pub fn allocator_type(_allocator: std::alloc::System) {}
pub fn empty() -> Empty {
    Empty {}
}
pub struct Tuple(Box<i32>);
pub fn tuple(a: i32) -> Tuple {
    Tuple { 0: Box::new(a) }
}
pub enum Choice {
    Named { first: Box<i32> },
}
pub fn enumeration(a: i32) -> Choice {
    Choice::Named { first: Box::new(a) }
}
pub struct Custom {
    first: Box<i32>,
}
impl Drop for Custom {
    fn drop(&mut self) {}
}
pub fn custom(a: i32) -> Custom {
    Custom { first: Box::new(a) }
}
#[repr(C)]
pub struct Repr {
    first: Box<i32>,
}
pub fn representation(a: i32) -> Repr {
    Repr { first: Box::new(a) }
}
pub fn update(base: Record, a: i32) -> Record {
    Record {
        first: Box::new(a),
        ..base
    }
}
pub mod imitation {
    pub struct Box(pub i32);
    pub struct Record {
        pub first: Box,
    }
}
pub fn counterfeit(a: i32) -> imitation::Record {
    imitation::Record {
        first: imitation::Box(a),
    }
}
pub fn no_record(a: i32) -> i32 {
    *Box::new(a)
}
