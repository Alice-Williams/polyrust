//! Search compares elements, so a record must not hide interface equality here.
//!
//! An alias cannot manufacture equatability for an interface target:
//!
//! ```compile_fail
//! use portable_build::{AliasValue, InterfaceValue, TypedEquatable};
//! fn require<T: TypedEquatable>() {}
//! fn forbidden<'module, 'alias, 'interface>() {
//!     require::<AliasValue<'module, 'alias, InterfaceValue<'module, 'interface>>>();
//! }
//! ```
//!
//! Private field-list evidence cannot be implemented by a client:
//!
//! ```compile_fail
//! use portable_build::EquatableFields;
//! struct Forged;
//! impl EquatableFields for Forged {}
//! ```
//!
//! ```compile_fail
//! use portable_build::{Bool, field, list_type, parameter, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("contains_interface_record"), |builder| {
//!     builder.interface(portable_name!("View"), typed_list![], |builder, view| {
//!         builder.record(portable_name!("Holder"),
//!             typed_list![field(portable_name!("view"), view.ty())], |builder, holder| {
//!             builder.function(portable_name!("contains"),
//!                 typed_list![parameter(portable_name!("list"), list_type(holder.ty())),
//!                             parameter(portable_name!("value"), holder.ty())],
//!                 Bool::TYPE, |body, args| {
//!                     let list = body.read(args.head);
//!                     let value = body.read(args.tail.head);
//!                     body.list_contains(list, value)
//!                 }).builder
//!         })
//!     })
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::{I64, field, list_type, option_type, parameter, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("index_interface_record"), |builder| {
//!     builder.interface(portable_name!("View"), typed_list![], |builder, view| {
//!         builder.record(portable_name!("Holder"),
//!             typed_list![field(portable_name!("view"), view.ty())], |builder, holder| {
//!             builder.function(portable_name!("index"),
//!                 typed_list![parameter(portable_name!("list"), list_type(holder.ty())),
//!                             parameter(portable_name!("value"), holder.ty())],
//!                 option_type(I64::TYPE), |body, args| {
//!                     let list = body.read(args.head);
//!                     let value = body.read(args.tail.head);
//!                     body.list_index_of(list, value)
//!                 }).builder
//!         })
//!     })
//! });
//! ```
//!
//! Method receivers retain field types rather than regaining blanket equality.
//!
//! ```compile_fail
//! use portable_build::{Bool, field, interface_method, method_binding, parameter, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("receiver_equality"), |builder| {
//!     builder.interface(portable_name!("View"), typed_list![], |builder, view| {
//!         builder.record(portable_name!("Holder"),
//!             typed_list![field(portable_name!("view"), view.ty())], |builder, holder| {
//!             builder.interface(portable_name!("Compare"),
//!                 typed_list![interface_method(portable_name!("compare"),
//!                     typed_list![parameter(portable_name!("other"), holder.ty())], Bool::TYPE)],
//!                 |builder, compare| {
//!                     let method = method_binding(&holder, &compare.methods().head,
//!                         portable_name!("compare"), |body, receiver, args| {
//!                             let other = body.read(args.head);
//!                             body.equal(receiver, other)
//!                         });
//!                     builder.implementation(portable_name!("CompareHolder"), &compare,
//!                         &holder, typed_list![method], |builder, _| builder)
//!                 })
//!         })
//!     })
//! });
//! ```
