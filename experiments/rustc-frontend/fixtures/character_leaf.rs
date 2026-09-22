//! Original Unicode scalar producer, with private implementation and mixed fields.
#![forbid(unsafe_code)]

struct Packet {
    character: char,
    #[expect(
        dead_code,
        reason = "Observe initializer evaluation even when this field is unread"
    )]
    marker: i32,
}

struct SecondPacket {
    #[expect(
        dead_code,
        reason = "Observe initializer evaluation even when this field is unread"
    )]
    character: char,
    marker: i32,
}

fn hidden(value: char) -> char {
    value
}

fn hidden_marker(value: i32) -> i32 {
    value
}

/// Return the original Unicode scalar without byte or UTF-16 conversion.
pub fn identity(value: char) -> char {
    hidden(value)
}

pub use identity as scalar_alias;

/// Preserve a scalar through an immutable local.
pub fn local(value: char) -> char {
    let saved = identity(value);
    hidden(saved)
}

/// Select without evaluating or encoding a text representation.
pub fn select(condition: bool, left: char, right: char) -> char {
    if condition { left } else { right }
}

/// Keep source character and signed integer field identities distinct.
pub fn record_character(value: char, marker: i32) -> char {
    let packet = Packet {
        marker: hidden_marker(marker),
        character: identity(value),
    };
    packet.character
}

/// Read the signed integer field from the same source shape.
pub fn record_marker(value: char, marker: i32) -> i32 {
    let packet = SecondPacket {
        character: identity(value),
        marker: hidden_marker(marker),
    };
    packet.marker
}

/// Six source comparisons are deliberately separate typed functions.
pub mod compare {
    pub fn equal(left: char, right: char) -> bool {
        left == right
    }
    pub fn not_equal(left: char, right: char) -> bool {
        left != right
    }
    pub fn less(left: char, right: char) -> bool {
        left < right
    }
    pub fn less_equal(left: char, right: char) -> bool {
        left <= right
    }
    pub fn greater(left: char, right: char) -> bool {
        left > right
    }
    pub fn greater_equal(left: char, right: char) -> bool {
        left >= right
    }
}

/// Literal boundaries include unassigned, noncharacter and supplementary scalars.
pub mod literals {
    pub fn nul() -> char {
        '\0'
    }
    pub fn one() -> char {
        '\u{1}'
    }
    pub fn ascii_end() -> char {
        '\u{7f}'
    }
    pub fn non_ascii() -> char {
        '\u{80}'
    }
    pub fn byte_end() -> char {
        '\u{ff}'
    }
    pub fn above_byte() -> char {
        '\u{100}'
    }
    pub fn unassigned() -> char {
        '\u{378}'
    }
    pub fn two_byte_end() -> char {
        '\u{7ff}'
    }
    pub fn three_byte_start() -> char {
        '\u{800}'
    }
    pub fn before_surrogates() -> char {
        '\u{d7ff}'
    }
    pub fn after_surrogates() -> char {
        '\u{e000}'
    }
    pub fn noncharacter() -> char {
        '\u{fdd0}'
    }
    pub fn bmp_noncharacter() -> char {
        '\u{fffe}'
    }
    pub fn bmp_end() -> char {
        '\u{ffff}'
    }
    pub fn supplementary() -> char {
        '\u{10000}'
    }
    pub fn crab() -> char {
        '\u{1f980}'
    }
    pub fn private_use() -> char {
        '\u{f0000}'
    }
    pub fn penultimate() -> char {
        '\u{10fffe}'
    }
    pub fn maximum() -> char {
        '\u{10ffff}'
    }
}
