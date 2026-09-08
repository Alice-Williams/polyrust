//! A record must not erase the equatability of its recursively contained fields.
//!
//! Interface storage remains supported, but equality of the containing record
//! must fail at Rust compile time, not panic in the private checked-IR bridge.
//!
//! ```compile_fail
//! use portable_build::{Bool, field, parameter, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("interface_record_equality"), |builder| {
//!     builder.interface(portable_name!("View"), typed_list![], |builder, view| {
//!         builder.record(portable_name!("Holder"),
//!             typed_list![field(portable_name!("view"), view.ty())], |builder, holder| {
//!             builder.function(portable_name!("compare"),
//!                 typed_list![parameter(portable_name!("left"), holder.ty()),
//!                             parameter(portable_name!("right"), holder.ty())],
//!                 Bool::TYPE, |body, args| {
//!                     let left = body.read(args.head);
//!                     let right = body.read(args.tail.head);
//!                     body.equal(left, right)
//!                 }).builder
//!         })
//!     })
//! });
//! ```
//!
//! Neither optional/list nesting nor another record can recover equality.
//!
//! ```compile_fail
//! use portable_build::{Bool, field, list_type, option_type, parameter, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("nested_interface_equality"), |builder| {
//!     builder.interface(portable_name!("View"), typed_list![], |builder, view| {
//!         builder.record(portable_name!("Inner"),
//!             typed_list![field(portable_name!("views"), option_type(list_type(view.ty())))],
//!             |builder, inner| builder.record(portable_name!("Outer"),
//!                 typed_list![field(portable_name!("inner"), inner.ty())], |builder, outer| {
//!                     builder.function(portable_name!("compare"),
//!                         typed_list![parameter(portable_name!("left"), outer.ty()),
//!                                     parameter(portable_name!("right"), outer.ty())],
//!                         Bool::TYPE, |body, args| {
//!                             let left = body.read(args.head);
//!                             let right = body.read(args.tail.head);
//!                             body.not_equal(left, right)
//!                         }).builder
//!                 }))
//!     })
//! });
//! ```

use super::{AliasValue, Cons, Nil, TypedEquatable};

mod search;
#[cfg(test)]
mod tests;

mod sealed {
    pub trait Fields {}
}

/// A recursive field-type list whose values all admit portable equality.
/// The trait is sealed; callers cannot label an interface field equatable.
pub trait EquatableFields: sealed::Fields {}

impl sealed::Fields for Nil {}
impl EquatableFields for Nil {}
impl<Head: TypedEquatable, Tail: EquatableFields> sealed::Fields for Cons<Head, Tail> {}
impl<Head: TypedEquatable, Tail: EquatableFields> EquatableFields for Cons<Head, Tail> {}

// A transparent alias preserves its target's equality capability; it cannot
// erase an interface or turn an enum comparison into general record equality.
impl<T: TypedEquatable> super::sealed::Equatable for AliasValue<'_, '_, T> {}
impl<T: TypedEquatable> TypedEquatable for AliasValue<'_, '_, T> {
    type EqualityCapability = T::EqualityCapability;
}
