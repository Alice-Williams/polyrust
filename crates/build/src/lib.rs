#![forbid(unsafe_code)]

//! Typed, target-independent Rust authoring API for PolyIR v0.
//!
//! The builder assigns stable node identities and logical source paths. Typed
//! handles prevent declaration-family mixups, while finalization reports
//! diagnostics rather than panicking.
//!
//! # Complete checked and evaluated module
//!
//! ```
//! use portable_build::{
//!     Expected, Invocation, ModuleBuilder, Parameter, Type, TypedValue, Value,
//!     Visibility,
//! };
//! use portable_eval::Evaluator;
//!
//! let mut module = ModuleBuilder::new("example");
//! let (message, text) = module.record(
//!     "Message",
//!     Visibility::Public,
//!     vec!["A portable message.".into()],
//!     |record| record.field("text", Type::string(), vec![]),
//! );
//! let identity = module.function(
//!     "identity",
//!     Visibility::Public,
//!     vec![],
//!     |function| {
//!         function.parameter(Parameter::new("value", Type::named(message)));
//!         function.returns(Type::named(message));
//!         function.body(|body| {
//!             let value = body.local("value");
//!             body.block([], Some(value))
//!         });
//!     },
//! );
//! module.portable_test(
//!     "identity_returns_input",
//!     Visibility::Package,
//!     vec![],
//!     Invocation::function(
//!         identity,
//!         [TypedValue::new(
//!             Type::named(message),
//!             Value::record(message, [(text, Value::string("hello"))]),
//!         )],
//!     ),
//!     Expected::value(TypedValue::new(
//!         Type::named(message),
//!         Value::record(message, [(text, Value::string("hello"))]),
//!     )),
//! );
//!
//! let checked = module.finish().expect("builder and checker accept the module");
//! assert!(Evaluator::new(&checked).run_all_tests().iter().all(|test| test.passed));
//! ```
//!
//! Record handles cannot be used where contract handles are required:
//!
//! ```compile_fail
//! use portable_build::{ModuleBuilder, Type, Visibility};
//!
//! let mut module = ModuleBuilder::new("typed");
//! let (record, ()) = module.record("R", Visibility::Public, vec![], |_| {});
//! let _wrong = Type::contract(record);
//! ```
//!
//! Contract handles cannot be used as record constructors:
//!
//! ```compile_fail
//! use portable_build::{ModuleBuilder, Value, Visibility};
//!
//! let mut module = ModuleBuilder::new("typed");
//! let (contract, ()) = module.contract("C", Visibility::Public, vec![], |_| {});
//! let _wrong = Value::record(contract, []);
//! ```

mod body;
mod handles;
mod module;
mod value;

pub use body::*;
pub use handles::*;
pub use module::*;
pub use portable_ir::v0::{Intrinsic as Operation, Visibility};
pub use value::*;

#[cfg(test)]
mod tests;
