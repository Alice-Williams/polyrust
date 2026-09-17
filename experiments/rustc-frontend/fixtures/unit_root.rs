//! Mixed unit/scalar API with private functions and block continuation.
#![allow(clippy::unused_unit)]
pub use unit_leaf::MARK;
fn private(value: i32) {
    unit_leaf::observe(value, value, true);
}
/// Continue after statement-position effects.
pub fn execute(value: i32, flag: bool) -> i32 {
    unit_relay::relay(value, flag);
    {
        private(value);
    }
    if flag {
        unit_leaf::empty();
    } else {
        unit_leaf::explicit();
    }
    value
}
/// Unit tail call.
pub fn tail(value: i32) -> () {
    unit_relay::relay(value, true)
}
/// Unit conditional tail.
pub fn branch(flag: bool) {
    if flag {
        unit_leaf::empty()
    } else {
        unit_leaf::explicit()
    }
}
