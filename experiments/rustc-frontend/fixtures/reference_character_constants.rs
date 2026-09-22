//! Compile-time scalar storage and contexts, independent of target mappings.
const BOUNDARIES: [char; 19] = [
    '\0',
    '\u{1}',
    '\u{7f}',
    '\u{80}',
    '\u{ff}',
    '\u{100}',
    '\u{378}',
    '\u{7ff}',
    '\u{800}',
    '\u{d7ff}',
    '\u{e000}',
    '\u{fdd0}',
    '\u{fffe}',
    '\u{ffff}',
    '\u{10000}',
    '\u{1f980}',
    '\u{f0000}',
    '\u{10fffe}',
    char::MAX,
];
const COUNT: usize = BOUNDARIES.len() + 4096;

const fn values() -> [char; COUNT] {
    let mut result = ['\0'; COUNT];
    let mut index = 0;
    while index < BOUNDARIES.len() {
        result[index] = BOUNDARIES[index];
        index += 1;
    }
    let mut sample = 0_u32;
    while index < COUNT {
        let ordinal = (sample * 65537 + 17) % 1_112_064;
        let scalar = if ordinal < 0xd800 {
            ordinal
        } else {
            ordinal + 2048
        };
        result[index] = match char::from_u32(scalar) {
            Some(value) => value,
            None => panic!("constant corpus is not a scalar"),
        };
        index += 1;
        sample += 1;
    }
    result
}
static VALUES: [char; COUNT] = values();

mod source {
    pub const MODULE: char = '\u{1f980}';
    const PRIVATE: char = '\u{fdd0}';
    pub const FORWARDED: char = PRIVATE;
    pub use MODULE as ALIAS;
}

struct Owner;
impl Owner {
    const MAXIMUM: char = char::MAX;
    const SELECTED: char = if Self::MAXIMUM > '\u{ffff}' {
        '\u{e000}'
    } else {
        'x'
    };
}
const COMPUTED: char = match char::from_u32(65535 + 1) {
    Some(value) => value,
    None => panic!("computed constant is not a scalar"),
};
const INVALID: [Option<char>; 4] = [
    char::from_u32(0xd800),
    char::from_u32(0xdfff),
    char::from_u32(0x110000),
    char::from_u32(u32::MAX),
];

fn contexts() -> [char; 12] {
    const LOCAL: char = '\u{378}';
    const LOCAL_ALIAS: char = source::ALIAS;
    const CONTEXTS: [char; 12] = [
        source::MODULE,
        source::ALIAS,
        source::FORWARDED,
        Owner::MAXIMUM,
        Owner::SELECTED,
        COMPUTED,
        BOUNDARIES[9],
        '\'',
        '\\',
        '\n',
        LOCAL,
        LOCAL_ALIAS,
    ];
    CONTEXTS
}

fn main() {
    let contexts = contexts();
    println!("character-constants-v1 {}", VALUES.len() + contexts.len());
    for value in VALUES.iter().chain(contexts.iter()).copied() {
        let raw = u32::from(value);
        let byte = u32::from(value as u8);
        let code_unit = u32::from(value as u16);
        let replacement = if value.is_ascii() { raw } else { 0xfffd };
        let changed = raw ^ 1;
        println!("{raw:08x} {byte:08x} {code_unit:08x} {replacement:08x} {changed:08x}");
    }
    for value in INVALID {
        println!("invalid {}", u8::from(value.is_some()));
    }
}
