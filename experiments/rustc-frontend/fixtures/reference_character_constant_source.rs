//! Observe original Rust without deriving expectations from target output.
fn main() {
    let values = [
        root::read_zero() as u32,
        root::read_ascii() as u32,
        root::read_same_integer() as u32,
        root::read_before_surrogates() as u32,
        root::read_after_surrogates() as u32,
        root::read_noncharacter() as u32,
        root::read_supplementary() as u32,
        root::read_maximum() as u32,
        root::read_computed() as u32,
        root::read_copied() as u32,
        root::read_other_same() as u32,
        root::read_other_maximum() as u32,
        root::read_alias() as u32,
        root::read_own() as u32,
        root::read_private() as u32,
        root::read_local() as u32,
        root::read_inherent() as u32,
        root::read_comparison() as u32,
    ];
    for value in values {
        println!("{value}");
    }
}
