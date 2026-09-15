//! Built-in bool Not in ordinary runtime-free Rust-source packages.
#![allow(clippy::nonminimal_bool)] // Preserve nested operator cases for translation.

struct Flags {
    enabled: bool,
}

fn positive(value: i32) -> bool {
    value > 0
}

fn identity(value: bool) -> bool {
    value
}

pub fn invert(value: bool) -> bool {
    !value
}

pub fn literal_true() -> bool {
    !true
}

pub fn literal_false() -> bool {
    !false
}

pub fn nested(value: bool) -> bool {
    !!value
}

pub fn comparison(value: i32) -> bool {
    !(value > 0)
}

pub fn call(value: i32) -> bool {
    !positive(value)
}

pub fn field(value: bool) -> bool {
    let flags = Flags { enabled: value };
    !flags.enabled
}

pub fn shared(value: bool) -> bool {
    let reference = &value;
    !*reference
}

pub fn shadow(value: bool) -> bool {
    let value = !value;
    !value
}

pub fn conditional(value: i32) -> i32 {
    if !(value > 0) { -1 } else { 1 }
}

pub fn nested_call(value: i32) -> bool {
    !invert(positive(value))
}

pub fn argument(value: i32) -> bool {
    identity(!(value > 0))
}
