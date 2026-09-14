//! Public aliases preserve private definition ownership.
mod hidden {
    pub mod alias_a {
        pub use super::alias_b as next;
        pub use crate::score as invoke;
    }
    pub mod alias_b {
        pub use super::alias_a as next;
    }
    /// Public only through re-exports.
    pub struct Visible {
        pub value: i32,
        secret: i32,
    }
    pub struct Unexported {
        pub value: i32,
    }
    pub(crate) struct CrateOnly {
        pub(crate) value: i32,
    }
    pub(super) struct ParentOnly {
        pub(super) value: i32,
    }
    pub(in crate::hidden) struct Scoped {
        pub(in crate::hidden) value: i32,
    }
    struct Private {
        value: i32,
    }

    pub fn score(input: i32) -> i32 {
        let visible = Visible {
            value: input,
            secret: 0,
        };
        let hidden = Unexported { value: input };
        let crate_only = CrateOnly { value: input };
        let parent_only = ParentOnly { value: input };
        let scoped = Scoped { value: input };
        let private = Private { value: input };
        if visible.value < visible.secret {
            if hidden.value == crate_only.value {
                parent_only.value
            } else {
                0
            }
        } else if scoped.value == private.value {
            visible.value
        } else {
            0
        }
    }
}

pub use hidden::alias_a as alias_only;
pub use hidden::{Visible as Exported, score};
pub use score as entry;
pub mod api {
    pub use crate::api as again;
    pub use crate::api as r#type;
    pub use crate::hidden::Visible as Renamed;
    pub use crate::same as peer;
    pub use crate::score as eval;
}
pub use api as mirror;

pub use core::cmp as dependency;
pub use core::cmp::min as foreign_function;

#[path = "export_graph_out.rs"]
pub mod external;

#[macro_export]
macro_rules! public_marker {
    () => {
        1
    };
}

pub mod same {
    pub use crate::api as peer;
    pub use crate::public_marker as dual;
    pub struct Name {
        pub value: i32,
    }
    pub fn name(input: i32) -> i32 {
        input
    }
    pub use Name as dual;
    pub use name as dual;
}
