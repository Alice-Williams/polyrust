//! Safe native Rust char observations; no target implementation is imported.
use std::io::{self, Read, Write};

fn flags<T: Ord>(left: T, right: T) -> u32 {
    u32::from(left == right)
        | (u32::from(left != right) << 1)
        | (u32::from(left < right) << 2)
        | (u32::from(left <= right) << 3)
        | (u32::from(left > right) << 4)
        | (u32::from(left >= right) << 5)
}

fn value(raw: u32) -> [u32; 6] {
    let character = char::from_u32(raw);
    [
        u32::from(character.is_some()),
        character.map(u32::from).unwrap_or(0),
        u32::from(raw as u16),
        u32::from(raw as u8),
        u32::from(character.is_some() && raw <= 0xffff),
        u32::from(raw <= 0x10ffff),
    ]
}

fn pair(left: u32, right: u32) -> [u32; 6] {
    let left = char::from_u32(left).unwrap();
    let right = char::from_u32(right).unwrap();
    let mut left_buffer = [0; 2];
    let mut right_buffer = [0; 2];
    let left_utf16 = left.encode_utf16(&mut left_buffer);
    let right_utf16 = right.encode_utf16(&mut right_buffer);
    [
        flags(left, right),
        flags(right, left),
        flags(&*left_utf16, &*right_utf16),
        flags(left as u8, right as u8),
        u32::from(left),
        u32::from(right),
    ]
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();
    let (packets, remainder) = input.as_chunks::<12>();
    assert!(remainder.is_empty(), "partial input packet");
    let mut output = io::BufWriter::new(io::stdout().lock());
    for packet in packets {
        let tag = u32::from_le_bytes(packet[..4].try_into().unwrap());
        let left = u32::from_le_bytes(packet[4..8].try_into().unwrap());
        let right = u32::from_le_bytes(packet[8..].try_into().unwrap());
        let row = match tag {
            0 => {
                assert_eq!(right, 0);
                value(left)
            }
            1 => pair(left, right),
            _ => panic!("unknown input packet"),
        };
        for field in row {
            output.write_all(&field.to_le_bytes()).unwrap();
        }
    }
}
