//! Identity aliases are syntax differences, not different constructors.
use std::boxed::Box as Renamed;

pub fn genuine(value: i32) -> i32 {
    *Box::new(value)
}
pub fn qualified(value: i32) -> i32 {
    *std::boxed::Box::<i32>::new(value)
}
pub fn renamed(value: i32) -> i32 {
    *Renamed::new(value)
}
pub fn unsupported(value: u32) -> u32 {
    *Box::new(value)
}
pub fn wrapper(value: i32) -> i32 {
    *imitation::new(value)
}
pub fn counterfeit(value: i32) -> i32 {
    imitation::Box::new(value).0
}

pub fn function_item(value: i32) -> i32 {
    let constructor = Box::new;
    *constructor(value)
}

pub fn callee_block(value: i32) -> i32 {
    *({
        let _ignored = value;
        Box::new
    })(value)
}

mod imitation {
    pub struct Box(pub i32);
    impl Box {
        pub fn new(value: i32) -> Self {
            Self(value)
        }
    }
    pub fn new(value: i32) -> std::boxed::Box<i32> {
        std::boxed::Box::new(value)
    }
}
