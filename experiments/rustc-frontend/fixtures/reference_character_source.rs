//! Execute original source owners; only this consumer decodes/encodes integers.
use character_leaf::{compare, literals};
use character_root as original;
use std::io::{self, Read, Write};

fn main() {
    let literals = [
        literals::nul(),
        literals::one(),
        literals::ascii_end(),
        literals::non_ascii(),
        literals::byte_end(),
        literals::above_byte(),
        literals::unassigned(),
        literals::two_byte_end(),
        literals::three_byte_start(),
        literals::before_surrogates(),
        literals::after_surrogates(),
        literals::noncharacter(),
        literals::bmp_noncharacter(),
        literals::bmp_end(),
        literals::supplementary(),
        literals::crab(),
        literals::private_use(),
        literals::penultimate(),
        literals::maximum(),
    ];
    let mut data = Vec::new();
    io::stdin().read_to_end(&mut data).unwrap();
    let (packets, remainder) = data.as_chunks::<12>();
    assert!(remainder.is_empty(), "partial character packet");
    let mut output = io::BufWriter::new(io::stdout().lock());
    for value in literals {
        output.write_all(&u32::from(value).to_le_bytes()).unwrap();
    }
    for packet in packets {
        let left = char::from_u32(u32::from_le_bytes(packet[..4].try_into().unwrap())).unwrap();
        let right = char::from_u32(u32::from_le_bytes(packet[4..8].try_into().unwrap())).unwrap();
        let marker = i32::from_le_bytes(packet[8..].try_into().unwrap());
        let flags = u32::from(compare::equal(left, right))
            | (u32::from(compare::not_equal(left, right)) << 1)
            | (u32::from(compare::less(left, right)) << 2)
            | (u32::from(compare::less_equal(left, right)) << 3)
            | (u32::from(compare::greater(left, right)) << 4)
            | (u32::from(compare::greater_equal(left, right)) << 5);
        for value in [
            u32::from(original::identity(left)),
            u32::from(original::local(left)),
            u32::from(original::scalar_alias(left)),
            u32::from(original::select(true, left, right)),
            u32::from(original::select(false, left, right)),
            u32::from(original::record_character(left, marker)),
            u32::from_ne_bytes(original::record_marker(left, marker).to_ne_bytes()),
            flags,
            u32::from(original::nested_less(left, right)),
        ] {
            output.write_all(&value.to_le_bytes()).unwrap();
        }
    }
}
