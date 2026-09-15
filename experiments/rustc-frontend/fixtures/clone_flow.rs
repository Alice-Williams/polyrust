#![allow(dead_code, unused_variables)]

pub fn method(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    *cloned
}
pub fn bounded(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    *cloned
}
pub fn qualified(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = <Box<i32> as Clone>::clone(&original);
    *cloned
}
pub fn read_original(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    *original
}
pub fn return_original(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = Clone::clone(&original);
    return *original;
}
pub fn return_clone(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    return *cloned;
}
pub fn moves_before(value: i32) -> i32 {
    let first = Box::new(value);
    let second = first;
    let third = second;
    let cloned = third.clone();
    *cloned
}
pub fn moves_after(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    let moved = cloned;
    *moved
}
pub fn interleaved(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    let next_original = original;
    let next_cloned = cloned;
    let last_original = next_original;
    *next_cloned
}
pub fn shadow_original(value: i32) -> i32 {
    let owner = Box::new(value);
    let cloned = owner.clone();
    let owner = owner;
    *cloned
}
pub fn shadow_clone(value: i32) -> i32 {
    let owner = Box::new(value);
    let owner = owner.clone();
    *owner
}
pub fn no_clone(value: i32) -> i32 {
    let original = Box::new(value);
    *original
}
pub fn second_clone(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    let extra = original.clone();
    *cloned
}
pub fn second_allocation(value: i32) -> i32 {
    let original = Box::new(value);
    let extra = Box::new(value);
    let cloned = original.clone();
    *cloned
}
pub fn sum(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    *original + *cloned
}
pub fn nested(value: i32) -> i32 {
    let original = Box::new(value);
    {
        let cloned = original.clone();
        *cloned
    }
}
pub fn mutable(value: i32) -> i32 {
    let mut original = Box::new(value);
    let cloned = original.clone();
    *original = 1;
    *cloned
}
pub fn borrowed(value: i32) -> i32 {
    let original = Box::new(value);
    let reference = &original;
    let cloned = Clone::clone(reference);
    *cloned
}
pub fn temporary(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = Box::new(value).clone();
    *cloned
}
pub fn conditional(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = if value > 0 {
        original.clone()
    } else {
        original.clone()
    };
    *cloned
}
pub fn wrong_payload(value: bool) -> bool {
    let original = Box::new(value);
    let cloned = original.clone();
    *cloned
}
pub fn owner_return(value: i32) -> Box<i32> {
    let original = Box::new(value);
    let cloned = original.clone();
    cloned
}
pub fn extra_call(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    let moved = std::convert::identity(cloned);
    *moved
}

struct Container {
    owner: Box<i32>,
}
pub fn field_receiver(value: i32) -> i32 {
    let original = Box::new(value);
    let container = Container { owner: original };
    let cloned = container.owner.clone();
    *cloned
}
