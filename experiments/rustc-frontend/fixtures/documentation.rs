#![doc = "CRATE_FIRST"]
#![doc = ""]
#![doc = concat!("CRATE_", "LAST")]

pub mod first {
    //! MODULE_FIRST
    /// FIRST_RECORD
    #[doc = include_str!("documentation.md")]
    pub struct Ticket {
        /// FIRST_FIELD
        pub value: i32,
    }
}

mod second {
    #![doc = "MODULE_SECOND"]
    #[doc = " SECOND_RECORD"]
    pub struct Ticket {
        #[doc = " SECOND_FIELD"]
        pub value: i32,
    }
}

/// SCORE_FIRST
#[doc = "SCORE_LINE_ONE\r\nSCORE_LINE_TWO"]
#[doc = "HOSTILE /* inner */ ??/ \\\n#define NOT_CODE λ"]
pub fn score(input: i32) -> i32 {
    // ORDINARY_COMMENT_MUST_NOT_APPEAR
    let original = first::Ticket { value: input };
    let fallback = second::Ticket { value: 1 };
    let borrowed = &original;
    if borrowed.value < 0 {
        fallback.value
    } else {
        borrowed.value
    }
}
