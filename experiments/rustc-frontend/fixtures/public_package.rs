//! Public package root documentation.

/// A zero-parameter public operation.
pub fn zero() -> i32 {
    42
}

/// Mixed scalar parameters and return type.
pub fn choose(value: i32, take: bool, fallback: i32, other: bool) -> i32 {
    if take {
        hidden::identity(value)
    } else if other {
        fallback
    } else {
        zero()
    }
}

/// A scalar boolean result.
pub fn positive(value: i32) -> bool {
    value > 0
}

pub use hidden::exported;
pub use hidden::exported as alias;

mod hidden {
    //! Private ancestor documentation.
    /// A private concrete representation.
    struct Record {
        /// The field stays private in C despite Rust pub visibility.
        pub value: i32,
    }
    /// Public operation behind a private ancestor.
    pub fn exported(value: i32) -> i32 {
        let original = Record { value };
        let moved = original;
        let shared = &moved;
        identity(shared.value)
    }
    /// Crate-restricted helper documentation.
    pub(crate) fn identity(value: i32) -> i32 {
        scoped::outer(value)
    }
    mod scoped {
        //! Restricted helper-module documentation.
        pub(super) fn outer(value: i32) -> i32 {
            restricted(value)
        }
        pub(in crate::hidden) fn restricted(value: i32) -> i32 {
            self_only(value)
        }
        #[expect(
            clippy::needless_pub_self,
            reason = "explicit visibility mapping fixture"
        )]
        pub(self) fn self_only(value: i32) -> i32 {
            value
        }
    }
}

pub mod api {
    //! Public alias-module documentation.
    pub use super::api as again;
    pub use super::hidden::exported as invoke;
}
