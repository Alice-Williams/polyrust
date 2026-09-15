#![allow(dead_code)]

pub fn method(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = original.clone();
    *cloned
}
pub fn qualified(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = <Box<i32> as Clone>::clone(&original);
    *cloned
}
pub fn inferred(value: i32) -> i32 {
    let original = Box::new(value);
    let cloned = Clone::clone(&original);
    *original + *cloned
}
pub fn reference(value: i32) -> i32 {
    let original = Box::new(value);
    let borrowed = &original;
    let cloned = <&Box<i32> as Clone>::clone(&borrowed);
    **cloned
}
pub fn wrong_payload(value: bool) -> bool {
    let original = Box::new(value);
    let cloned = original.clone();
    *cloned
}
pub fn replacement(value: i32) -> i32 {
    let original = Box::new(value);
    let mut destination = Box::new(0);
    destination.clone_from(&original);
    *destination
}
struct Custom(i32);
impl Custom {
    fn clone(&self) -> Box<i32> {
        Box::new(self.0)
    }
}
pub fn custom(value: i32) -> i32 {
    let original = Custom(value);
    let cloned = original.clone();
    *cloned
}

type Alias = Box<i32>;
pub fn alias(value: Alias) -> Alias {
    value.clone()
}
pub fn shared(value: &Box<i32>) -> Box<i32> {
    value.clone()
}
pub fn temporary(value: i32) -> Box<i32> {
    Box::new(value).clone()
}
pub fn adjusted(value: Box<i32>) -> Box<dyn std::fmt::Debug> {
    value.clone()
}
pub fn generic<T: Clone>(value: T) -> T {
    value.clone()
}
pub fn indirect(value: Box<i32>) -> Box<i32> {
    let function: fn(&Box<i32>) -> Box<i32> = Clone::clone;
    function(&value)
}
pub struct Holder {
    value: Box<i32>,
}
impl Clone for Holder {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}
pub fn field(value: Holder) -> Box<i32> {
    value.value.clone()
}
pub fn preborrow(value: Box<i32>) -> Box<i32> {
    let reference = &value;
    Clone::clone(reference)
}
pub fn custom_trait(value: Holder) -> Holder {
    value.clone()
}
mod imitation {
    pub trait Impostor {
        fn clone(&self) -> Box<i32>;
    }
    impl Impostor for Box<i32> {
        fn clone(&self) -> Box<i32> {
            Box::new(**self)
        }
    }
}
pub fn same_signature(value: Box<i32>) -> Box<i32> {
    <Box<i32> as imitation::Impostor>::clone(&value)
}
