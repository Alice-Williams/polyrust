//! Valid-by-construction authoring for the bounded `StaticV1` profile.
//!
//! Unlike [`crate::ModuleBuilder`], this API never exposes untyped expressions,
//! string-selected locals, or unchecked declaration handles. Its private bridge
//! to the dynamic checker is a temporary implementation detail: rejection there
//! is a PolyRust defect, not a user validation result.
//!
//! Operand types are enforced by Rust:
//!
//! ```compile_fail
//! use portable_build::{portable_name, static_program, I32, StaticV1};
//! let _ = static_program::<StaticV1>(portable_name!("mixed"), |module| {
//!     module.function0(portable_name!("bad"), I32::TYPE, |body| {
//!         body.int_add_checked(body.i32(1), body.text("not an integer"))
//!     });
//! });
//! ```
//!
//! Boolean operations accept only Boolean expressions:
//!
//! ```compile_fail
//! use portable_build::{portable_name, static_program, Bool, StaticV1};
//! let _ = static_program::<StaticV1>(portable_name!("booleans"), |module| {
//!     module.function0(portable_name!("bad"), Bool::TYPE, |body| {
//!         let left = body.bool(true);
//!         let right = body.i32(1);
//!         body.bool_and(left, right)
//!     });
//! });
//! ```
//!
//! Return types are enforced by Rust:
//!
//! ```compile_fail
//! use portable_build::{portable_name, static_program, I32, StaticV1};
//! let _ = static_program::<StaticV1>(portable_name!("returns"), |module| {
//!     module.function0(portable_name!("bad"), I32::TYPE, |body| body.bool(true));
//! });
//! ```
//!
//! Function arguments are exact:
//!
//! ```compile_fail
//! use portable_build::{portable_name, static_program, I32, StaticV1};
//! let _ = static_program::<StaticV1>(portable_name!("calls"), |module| {
//!     let identity = module.function1(
//!         portable_name!("identity"),
//!         (portable_name!("value"), I32::TYPE),
//!         I32::TYPE,
//!         |body, value| body.read(value),
//!     );
//!     module.function0(portable_name!("bad"), I32::TYPE, |body| {
//!         body.call1(identity, body.bool(true))
//!     });
//! });
//! ```
//!
//! Record constructors require every field with its declared type:
//!
//! ```compile_fail
//! use portable_build::{portable_name, static_program, I32, StaticV1};
//! let _ = static_program::<StaticV1>(portable_name!("records"), |module| {
//!     module.record2(
//!         portable_name!("Pair"),
//!         (portable_name!("left"), I32::TYPE),
//!         (portable_name!("right"), I32::TYPE),
//!         |module, pair| {
//!             module.function0(portable_name!("bad"), pair.ty(), |body| {
//!                 body.construct2(pair, body.i32(1), body.bool(false))
//!             });
//!         },
//!     );
//! });
//! ```
//!
//! A field from one record cannot be projected from another record:
//!
//! ```compile_fail
//! use portable_build::{portable_name, static_program, I32, StaticV1};
//! let _ = static_program::<StaticV1>(portable_name!("fields"), |module| {
//!     module.record1(
//!         portable_name!("Left"),
//!         (portable_name!("value"), I32::TYPE),
//!         |module, left| {
//!             module.record1(
//!                 portable_name!("Right"),
//!                 (portable_name!("value"), I32::TYPE),
//!                 |module, right| {
//!                     module.function1(
//!                         portable_name!("bad"),
//!                         (portable_name!("value"), right.ty()),
//!                         I32::TYPE,
//!                         |body, value| {
//!                             let value = body.read(value);
//!                             body.field(value, left.field())
//!                         },
//!                     );
//!                 },
//!             );
//!         },
//!     );
//! });
//! ```
//!
//! Protected names fail during constant evaluation:
//!
//! ```compile_fail
//! use portable_build::portable_name;
//! const BAD: portable_build::PortableName = portable_name!("class");
//! ```
//!
//! The proof wrapper cannot be forged:
//!
//! ```compile_fail
//! use portable_build::{StaticProgram, StaticV1};
//! let _ = StaticProgram::<StaticV1> { checked: panic!() };
//! ```

use std::{cell::Cell, marker::PhantomData};

use portable_check::v0::CheckedProgram;

use crate::{
    BodyBuilder, FunctionId, ModuleBuilder, Operation, Parameter, RecordFieldId, RecordId, Type,
    Value, Visibility,
};

mod sealed {
    pub trait Profile {}
    pub trait Equatable {}
    pub trait Ordered {}
    pub trait Integer {}
}

/// The first closed, portable feature profile.
#[derive(Clone, Copy, Debug)]
pub struct StaticV1;

impl sealed::Profile for StaticV1 {}

/// A closed static feature profile.
pub trait StaticFeatureProfile: sealed::Profile {}

impl StaticFeatureProfile for StaticV1 {}

/// Compile-time evidence that a target dialect implements a feature profile.
///
/// Deliberately has no blanket implementation.
pub trait Supports<F: StaticFeatureProfile> {}

/// Value types admitted by equality operations in a static profile.
pub trait StaticEquatable: sealed::Equatable {}
/// Value types admitted by ordered comparisons in a static profile.
pub trait StaticOrdered: sealed::Ordered {}
/// Value types admitted by integer operations in a static profile.
pub trait StaticInteger: sealed::Integer {}

/// An ASCII portable identifier proven usable by every initial target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortableName(&'static str);

impl PortableName {
    #[doc(hidden)]
    pub const fn checked(value: &'static str) -> Self {
        assert_valid_portable_name(value);
        Self(value)
    }

    fn preferred(self) -> &'static str {
        self.0
    }
}

/// Constructs a portable name and validates a literal during constant evaluation.
#[macro_export]
macro_rules! portable_name {
    ($value:literal) => {{
        const NAME: $crate::PortableName = $crate::PortableName::checked($value);
        NAME
    }};
}

const fn assert_valid_portable_name(value: &str) {
    let bytes = value.as_bytes();
    assert!(!bytes.is_empty(), "portable identifier must not be empty");
    assert!(
        is_ascii_start(bytes[0]),
        "portable identifier has an invalid first character"
    );
    let mut index = 1;
    while index < bytes.len() {
        assert!(
            is_ascii_continue(bytes[index]),
            "portable identifier contains an invalid character"
        );
        index += 1;
    }
    assert!(
        !is_protected(value),
        "portable identifier is protected by a supported language"
    );
}

const fn is_ascii_start(byte: u8) -> bool {
    byte == b'_' || (byte >= b'A' && byte <= b'Z') || (byte >= b'a' && byte <= b'z')
}

const fn is_ascii_continue(byte: u8) -> bool {
    is_ascii_start(byte) || (byte >= b'0' && byte <= b'9')
}

const fn is_protected(value: &str) -> bool {
    let protected = [
        "abstract",
        "assert",
        "boolean",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "class",
        "const",
        "continue",
        "default",
        "do",
        "double",
        "else",
        "enum",
        "extends",
        "false",
        "final",
        "finally",
        "float",
        "for",
        "goto",
        "if",
        "implements",
        "import",
        "instanceof",
        "int",
        "interface",
        "long",
        "native",
        "new",
        "null",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "short",
        "static",
        "strictfp",
        "super",
        "switch",
        "synchronized",
        "this",
        "throw",
        "throws",
        "transient",
        "true",
        "try",
        "void",
        "volatile",
        "while",
        "record",
        "sealed",
        "permits",
        "var",
        "yield",
        "_",
    ];
    let mut index = 0;
    while index < protected.len() {
        if const_str_eq(value, protected[index]) {
            return true;
        }
        index += 1;
    }
    false
}

const fn const_str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// A typed witness for a portable value type.
pub struct StaticType<T> {
    ir: Type,
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for StaticType<T> {
    fn clone(&self) -> Self {
        Self {
            ir: self.ir.clone(),
            marker: PhantomData,
        }
    }
}

/// Boolean value marker.
pub enum Bool {}
/// 32-bit signed integer marker.
pub enum I32 {}
/// 64-bit signed integer marker.
pub enum I64 {}
/// IEEE-754 binary64 marker.
pub enum F64 {}
/// Unicode string marker.
pub enum Text {}

macro_rules! primitive_type {
    ($marker:ident, $type_fn:ident) => {
        impl $marker {
            pub const TYPE: StaticType<Self> = StaticType {
                ir: Type::$type_fn(),
                marker: PhantomData,
            };
        }
    };
}

primitive_type!(Bool, bool);
primitive_type!(I32, i32);
primitive_type!(I64, i64);
primitive_type!(F64, f64);
primitive_type!(Text, string);

impl sealed::Equatable for Bool {}
impl StaticEquatable for Bool {}
impl sealed::Equatable for I32 {}
impl StaticEquatable for I32 {}
impl sealed::Equatable for I64 {}
impl StaticEquatable for I64 {}
impl sealed::Equatable for F64 {}
impl StaticEquatable for F64 {}
impl sealed::Equatable for Text {}
impl StaticEquatable for Text {}
impl sealed::Ordered for I32 {}
impl StaticOrdered for I32 {}
impl sealed::Ordered for I64 {}
impl StaticOrdered for I64 {}
impl sealed::Ordered for F64 {}
impl StaticOrdered for F64 {}
impl sealed::Ordered for Text {}
impl StaticOrdered for Text {}
impl sealed::Integer for I32 {}
impl StaticInteger for I32 {}
impl sealed::Integer for I64 {}
impl StaticInteger for I64 {}

/// A record value branded with the module and exact declaration that created it.
pub struct RecordValue<'module, 'record>(PhantomData<(Cell<&'module ()>, Cell<&'record ()>)>);

impl sealed::Equatable for RecordValue<'_, '_> {}
impl StaticEquatable for RecordValue<'_, '_> {}

/// A typed expression owned by one callable body.
pub struct StaticExpr<'module, 'body, T> {
    node: StaticNode,
    marker: PhantomData<fn(&'module (), &'body (), T) -> T>,
}

enum StaticNode {
    Literal(Value),
    Local(String),
    Record {
        record: RecordId,
        fields: Vec<(RecordFieldId, StaticNode)>,
    },
    Field {
        base: Box<StaticNode>,
        field: RecordFieldId,
    },
    Call {
        function: FunctionId,
        arguments: Vec<StaticNode>,
    },
    Intrinsic {
        operation: Operation,
        arguments: Vec<StaticNode>,
    },
}

/// A typed local issued by one callable body.
pub struct StaticLocal<'module, 'body, T> {
    name: String,
    marker: PhantomData<fn(&'module (), &'body (), T) -> T>,
}

impl<T> Clone for StaticLocal<'_, '_, T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            marker: PhantomData,
        }
    }
}

/// A typed function handle with an exact argument tuple and return type.
pub struct StaticFunction<'module, Arguments, Result> {
    raw: FunctionId,
    marker: PhantomData<fn(&'module (), Arguments) -> Result>,
}

impl<A, R> Copy for StaticFunction<'_, A, R> {}
impl<A, R> Clone for StaticFunction<'_, A, R> {
    fn clone(&self) -> Self {
        *self
    }
}

type InvariantRecordBrand<'module, 'record, T> = (Cell<&'module ()>, Cell<&'record ()>, fn(T) -> T);

/// A field handle tied to one exact record declaration.
pub struct StaticField<'module, 'record, T> {
    raw: RecordFieldId,
    marker: PhantomData<InvariantRecordBrand<'module, 'record, T>>,
}

impl<T> Copy for StaticField<'_, '_, T> {}
impl<T> Clone for StaticField<'_, '_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An exact one-field record constructor.
pub struct StaticRecord1<'module, 'record, A> {
    raw: RecordId,
    first: StaticField<'module, 'record, A>,
}

impl<A> Copy for StaticRecord1<'_, '_, A> {}
impl<A> Clone for StaticRecord1<'_, '_, A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'module, 'record, A> StaticRecord1<'module, 'record, A> {
    pub fn ty(self) -> StaticType<RecordValue<'module, 'record>> {
        StaticType {
            ir: Type::named(self.raw),
            marker: PhantomData,
        }
    }

    pub const fn field(self) -> StaticField<'module, 'record, A> {
        self.first
    }
}

/// An exact two-field record constructor.
pub struct StaticRecord2<'module, 'record, A, B> {
    raw: RecordId,
    first: StaticField<'module, 'record, A>,
    second: StaticField<'module, 'record, B>,
}

impl<A, B> Copy for StaticRecord2<'_, '_, A, B> {}
impl<A, B> Clone for StaticRecord2<'_, '_, A, B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'module, 'record, A, B> StaticRecord2<'module, 'record, A, B> {
    pub fn ty(self) -> StaticType<RecordValue<'module, 'record>> {
        StaticType {
            ir: Type::named(self.raw),
            marker: PhantomData,
        }
    }

    pub const fn first(self) -> StaticField<'module, 'record, A> {
        self.first
    }
    pub const fn second(self) -> StaticField<'module, 'record, B> {
        self.second
    }
}

/// An immutable portable program whose public constructors preserve `F`.
pub struct StaticProgram<F: StaticFeatureProfile> {
    checked: CheckedProgram,
    marker: PhantomData<fn() -> F>,
}

impl<F: StaticFeatureProfile> StaticProgram<F> {
    /// Private-data access for target adapters; callers cannot replace it.
    pub fn checked_program(&self) -> &CheckedProgram {
        &self.checked
    }
}

/// Builds a static program. Any panic from the private replay bridge is an
/// implementation invariant failure and should be reported as a PolyRust bug.
pub fn static_program<F: StaticFeatureProfile>(
    name: PortableName,
    build: impl for<'module> FnOnce(&mut StaticModule<'module>),
) -> StaticProgram<F> {
    fn branded<F: StaticFeatureProfile>(
        name: PortableName,
        build: impl for<'module> FnOnce(&mut StaticModule<'module>),
    ) -> StaticProgram<F> {
        let mut dynamic = ModuleBuilder::new(name.preferred());
        let mut module = StaticModule {
            dynamic: &mut dynamic,
            names: NameAllocator::default(),
            marker: PhantomData,
        };
        build(&mut module);
        let checked = dynamic.finish().unwrap_or_else(|diagnostics| {
            panic!("StaticProgram invariant failure: {diagnostics:#?}")
        });
        StaticProgram {
            checked,
            marker: PhantomData,
        }
    }
    branded(name, build)
}

#[derive(Default)]
struct NameAllocator {
    used: std::collections::BTreeSet<String>,
}

impl NameAllocator {
    fn allocate(&mut self, preferred: PortableName) -> String {
        let preferred = preferred.preferred();
        if self.used.insert(preferred.to_owned()) {
            return preferred.to_owned();
        }
        let mut suffix = 2_u64;
        loop {
            let candidate = format!("{preferred}_{suffix}");
            if self.used.insert(candidate.clone()) {
                return candidate;
            }
            suffix += 1;
        }
    }
}

/// Static module construction. Its lifetime brands every declaration handle.
pub struct StaticModule<'module> {
    dynamic: &'module mut ModuleBuilder,
    names: NameAllocator,
    marker: PhantomData<&'module mut &'module ()>,
}

impl<'module> StaticModule<'module> {
    pub fn record1<A, R>(
        &mut self,
        name: PortableName,
        first: (PortableName, StaticType<A>),
        then: impl for<'record> FnOnce(&mut Self, StaticRecord1<'module, 'record, A>) -> R,
    ) -> R {
        let name = self.names.allocate(name);
        let first_name = first.0.preferred();
        let (raw, field) = self
            .dynamic
            .record(name, Visibility::Public, vec![], |record| {
                record.field(first_name, first.1.ir, vec![])
            });
        then(
            self,
            StaticRecord1 {
                raw,
                first: StaticField {
                    raw: field,
                    marker: PhantomData,
                },
            },
        )
    }

    pub fn record2<A, B, R>(
        &mut self,
        name: PortableName,
        first: (PortableName, StaticType<A>),
        second: (PortableName, StaticType<B>),
        then: impl for<'record> FnOnce(&mut Self, StaticRecord2<'module, 'record, A, B>) -> R,
    ) -> R {
        let name = self.names.allocate(name);
        let mut fields = NameAllocator::default();
        let first_name = fields.allocate(first.0);
        let second_name = fields.allocate(second.0);
        let (raw, (first_field, second_field)) =
            self.dynamic
                .record(name, Visibility::Public, vec![], |record| {
                    (
                        record.field(first_name, first.1.ir, vec![]),
                        record.field(second_name, second.1.ir, vec![]),
                    )
                });
        then(
            self,
            StaticRecord2 {
                raw,
                first: StaticField {
                    raw: first_field,
                    marker: PhantomData,
                },
                second: StaticField {
                    raw: second_field,
                    marker: PhantomData,
                },
            },
        )
    }

    pub fn function0<R>(
        &mut self,
        name: PortableName,
        result: StaticType<R>,
        build: impl for<'body> FnOnce(&mut StaticBody<'module, 'body>) -> StaticExpr<'module, 'body, R>,
    ) -> StaticFunction<'module, (), R> {
        let name = self.names.allocate(name);
        let raw = self
            .dynamic
            .function(name, Visibility::Public, vec![], |function| {
                function.returns(result.ir).body(|body| {
                    let mut typed = StaticBody {
                        marker: PhantomData,
                    };
                    let result = build(&mut typed);
                    let result = lower_expression(body, result.node);
                    body.block([], Some(result))
                });
            });
        StaticFunction {
            raw,
            marker: PhantomData,
        }
    }

    pub fn function1<A, R>(
        &mut self,
        name: PortableName,
        parameter: (PortableName, StaticType<A>),
        result: StaticType<R>,
        build: impl for<'body> FnOnce(
            &mut StaticBody<'module, 'body>,
            StaticLocal<'module, 'body, A>,
        ) -> StaticExpr<'module, 'body, R>,
    ) -> StaticFunction<'module, (A,), R> {
        let name = self.names.allocate(name);
        let parameter_name = parameter.0.preferred().to_owned();
        let raw = self
            .dynamic
            .function(name, Visibility::Public, vec![], |function| {
                function.parameter(Parameter::new(&parameter_name, parameter.1.ir));
                function.returns(result.ir).body(|body| {
                    let local = StaticLocal {
                        name: parameter_name,
                        marker: PhantomData,
                    };
                    let mut typed = StaticBody {
                        marker: PhantomData,
                    };
                    let result = build(&mut typed, local);
                    let result = lower_expression(body, result.node);
                    body.block([], Some(result))
                });
            });
        StaticFunction {
            raw,
            marker: PhantomData,
        }
    }

    pub fn function2<A, B, R>(
        &mut self,
        name: PortableName,
        first: (PortableName, StaticType<A>),
        second: (PortableName, StaticType<B>),
        result: StaticType<R>,
        build: impl for<'body> FnOnce(
            &mut StaticBody<'module, 'body>,
            StaticLocal<'module, 'body, A>,
            StaticLocal<'module, 'body, B>,
        ) -> StaticExpr<'module, 'body, R>,
    ) -> StaticFunction<'module, (A, B), R> {
        let name = self.names.allocate(name);
        let mut parameters = NameAllocator::default();
        let first_name = parameters.allocate(first.0);
        let second_name = parameters.allocate(second.0);
        let raw = self
            .dynamic
            .function(name, Visibility::Public, vec![], |function| {
                function.parameter(Parameter::new(&first_name, first.1.ir));
                function.parameter(Parameter::new(&second_name, second.1.ir));
                function.returns(result.ir).body(|body| {
                    let first = StaticLocal {
                        name: first_name,
                        marker: PhantomData,
                    };
                    let second = StaticLocal {
                        name: second_name,
                        marker: PhantomData,
                    };
                    let mut typed = StaticBody {
                        marker: PhantomData,
                    };
                    let result = build(&mut typed, first, second);
                    let result = lower_expression(body, result.node);
                    body.block([], Some(result))
                });
            });
        StaticFunction {
            raw,
            marker: PhantomData,
        }
    }
}

/// The only expression factory for one statically branded function body.
pub struct StaticBody<'module, 'body> {
    marker: PhantomData<&'module mut &'body ()>,
}

impl<'module, 'body> StaticBody<'module, 'body> {
    fn expression<T>(&self, node: StaticNode) -> StaticExpr<'module, 'body, T> {
        StaticExpr {
            node,
            marker: PhantomData,
        }
    }

    pub fn read<T>(
        &mut self,
        local: StaticLocal<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.expression(StaticNode::Local(local.name))
    }

    pub fn bool(&mut self, value: bool) -> StaticExpr<'module, 'body, Bool> {
        self.expression(StaticNode::Literal(Value::bool(value)))
    }
    pub fn i32(&mut self, value: i32) -> StaticExpr<'module, 'body, I32> {
        self.expression(StaticNode::Literal(Value::i32(value)))
    }
    pub fn i64(&mut self, value: i64) -> StaticExpr<'module, 'body, I64> {
        self.expression(StaticNode::Literal(Value::i64(value)))
    }
    pub fn f64(&mut self, value: f64) -> StaticExpr<'module, 'body, F64> {
        self.expression(StaticNode::Literal(Value::f64(value)))
    }
    pub fn text(&mut self, value: impl Into<String>) -> StaticExpr<'module, 'body, Text> {
        self.expression(StaticNode::Literal(Value::string(value)))
    }

    pub fn construct1<'record, A>(
        &mut self,
        record: StaticRecord1<'module, 'record, A>,
        first: StaticExpr<'module, 'body, A>,
    ) -> StaticExpr<'module, 'body, RecordValue<'module, 'record>> {
        self.expression(StaticNode::Record {
            record: record.raw,
            fields: vec![(record.first.raw, first.node)],
        })
    }

    pub fn construct2<'record, A, B>(
        &mut self,
        record: StaticRecord2<'module, 'record, A, B>,
        first: StaticExpr<'module, 'body, A>,
        second: StaticExpr<'module, 'body, B>,
    ) -> StaticExpr<'module, 'body, RecordValue<'module, 'record>> {
        self.expression(StaticNode::Record {
            record: record.raw,
            fields: vec![
                (record.first.raw, first.node),
                (record.second.raw, second.node),
            ],
        })
    }

    pub fn field<'record, T>(
        &mut self,
        base: StaticExpr<'module, 'body, RecordValue<'module, 'record>>,
        field: StaticField<'module, 'record, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.expression(StaticNode::Field {
            base: Box::new(base.node),
            field: field.raw,
        })
    }

    pub fn call0<R>(
        &mut self,
        function: StaticFunction<'module, (), R>,
    ) -> StaticExpr<'module, 'body, R> {
        self.expression(StaticNode::Call {
            function: function.raw,
            arguments: vec![],
        })
    }
    pub fn call1<A, R>(
        &mut self,
        function: StaticFunction<'module, (A,), R>,
        argument: StaticExpr<'module, 'body, A>,
    ) -> StaticExpr<'module, 'body, R> {
        self.expression(StaticNode::Call {
            function: function.raw,
            arguments: vec![argument.node],
        })
    }
    pub fn call2<A, B, R>(
        &mut self,
        function: StaticFunction<'module, (A, B), R>,
        first: StaticExpr<'module, 'body, A>,
        second: StaticExpr<'module, 'body, B>,
    ) -> StaticExpr<'module, 'body, R> {
        self.expression(StaticNode::Call {
            function: function.raw,
            arguments: vec![first.node, second.node],
        })
    }

    fn unary<A, R>(
        &mut self,
        operation: Operation,
        value: StaticExpr<'module, 'body, A>,
    ) -> StaticExpr<'module, 'body, R> {
        self.expression(StaticNode::Intrinsic {
            operation,
            arguments: vec![value.node],
        })
    }
    fn binary<A, B, R>(
        &mut self,
        operation: Operation,
        left: StaticExpr<'module, 'body, A>,
        right: StaticExpr<'module, 'body, B>,
    ) -> StaticExpr<'module, 'body, R> {
        self.expression(StaticNode::Intrinsic {
            operation,
            arguments: vec![left.node, right.node],
        })
    }

    pub fn bool_not(
        &mut self,
        value: StaticExpr<'module, 'body, Bool>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.unary(Operation::BoolNot, value)
    }
    pub fn bool_and(
        &mut self,
        left: StaticExpr<'module, 'body, Bool>,
        right: StaticExpr<'module, 'body, Bool>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.binary(Operation::BoolAnd, left, right)
    }
    pub fn bool_or(
        &mut self,
        left: StaticExpr<'module, 'body, Bool>,
        right: StaticExpr<'module, 'body, Bool>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.binary(Operation::BoolOr, left, right)
    }
    pub fn equal<T: StaticEquatable>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.binary(Operation::Equal, left, right)
    }
    pub fn not_equal<T: StaticEquatable>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.binary(Operation::NotEqual, left, right)
    }
    pub fn less<T: StaticOrdered>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.binary(Operation::Less, left, right)
    }
    pub fn less_equal<T: StaticOrdered>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.binary(Operation::LessEqual, left, right)
    }
    pub fn greater<T: StaticOrdered>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.binary(Operation::Greater, left, right)
    }
    pub fn greater_equal<T: StaticOrdered>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, Bool> {
        self.binary(Operation::GreaterEqual, left, right)
    }

    pub fn int_neg_checked<T: StaticInteger>(
        &mut self,
        value: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.unary(Operation::IntNegChecked, value)
    }
    pub fn int_add_checked<T: StaticInteger>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.binary(Operation::IntAddChecked, left, right)
    }
    pub fn int_sub_checked<T: StaticInteger>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.binary(Operation::IntSubChecked, left, right)
    }
    pub fn int_mul_checked<T: StaticInteger>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.binary(Operation::IntMulChecked, left, right)
    }
    pub fn int_div_checked<T: StaticInteger>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.binary(Operation::IntDivChecked, left, right)
    }
    pub fn int_rem_checked<T: StaticInteger>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.binary(Operation::IntRemChecked, left, right)
    }
    pub fn int_neg_wrapping<T: StaticInteger>(
        &mut self,
        value: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.unary(Operation::IntNegWrapping, value)
    }
    pub fn int_add_wrapping<T: StaticInteger>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.binary(Operation::IntAddWrapping, left, right)
    }
    pub fn int_sub_wrapping<T: StaticInteger>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.binary(Operation::IntSubWrapping, left, right)
    }
    pub fn int_mul_wrapping<T: StaticInteger>(
        &mut self,
        left: StaticExpr<'module, 'body, T>,
        right: StaticExpr<'module, 'body, T>,
    ) -> StaticExpr<'module, 'body, T> {
        self.binary(Operation::IntMulWrapping, left, right)
    }

    pub fn float_neg(
        &mut self,
        value: StaticExpr<'module, 'body, F64>,
    ) -> StaticExpr<'module, 'body, F64> {
        self.unary(Operation::FloatNeg, value)
    }
    pub fn float_add(
        &mut self,
        left: StaticExpr<'module, 'body, F64>,
        right: StaticExpr<'module, 'body, F64>,
    ) -> StaticExpr<'module, 'body, F64> {
        self.binary(Operation::FloatAdd, left, right)
    }
    pub fn float_sub(
        &mut self,
        left: StaticExpr<'module, 'body, F64>,
        right: StaticExpr<'module, 'body, F64>,
    ) -> StaticExpr<'module, 'body, F64> {
        self.binary(Operation::FloatSub, left, right)
    }
    pub fn float_mul(
        &mut self,
        left: StaticExpr<'module, 'body, F64>,
        right: StaticExpr<'module, 'body, F64>,
    ) -> StaticExpr<'module, 'body, F64> {
        self.binary(Operation::FloatMul, left, right)
    }
    pub fn float_div(
        &mut self,
        left: StaticExpr<'module, 'body, F64>,
        right: StaticExpr<'module, 'body, F64>,
    ) -> StaticExpr<'module, 'body, F64> {
        self.binary(Operation::FloatDiv, left, right)
    }
    pub fn float_rem_trunc(
        &mut self,
        left: StaticExpr<'module, 'body, F64>,
        right: StaticExpr<'module, 'body, F64>,
    ) -> StaticExpr<'module, 'body, F64> {
        self.binary(Operation::FloatRemTrunc, left, right)
    }
    pub fn string_concat(
        &mut self,
        left: StaticExpr<'module, 'body, Text>,
        right: StaticExpr<'module, 'body, Text>,
    ) -> StaticExpr<'module, 'body, Text> {
        self.binary(Operation::StringConcat, left, right)
    }
}

fn lower_expression(body: &mut BodyBuilder<'_>, node: StaticNode) -> crate::Expr {
    match node {
        StaticNode::Literal(value) => body.literal(value),
        StaticNode::Local(name) => body.local(name),
        StaticNode::Record { record, fields } => {
            let fields = fields
                .into_iter()
                .map(|(field, value)| (field, lower_expression(body, value)))
                .collect::<Vec<_>>();
            body.record(record, fields)
        }
        StaticNode::Field { base, field } => {
            let base = lower_expression(body, *base);
            body.field(base, field)
        }
        StaticNode::Call {
            function,
            arguments,
        } => {
            let arguments = arguments
                .into_iter()
                .map(|argument| lower_expression(body, argument))
                .collect::<Vec<_>>();
            body.call(function, arguments)
        }
        StaticNode::Intrinsic {
            operation,
            arguments,
        } => {
            let arguments = arguments
                .into_iter()
                .map(|argument| lower_expression(body, argument))
                .collect::<Vec<_>>();
            body.intrinsic(operation, arguments)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_typed_records_calls_and_nested_arithmetic() {
        let program = static_program::<StaticV1>(portable_name!("static_fixture"), |module| {
            let compute = module.function2(
                portable_name!("compute"),
                (portable_name!("left"), I32::TYPE),
                (portable_name!("right"), I32::TYPE),
                I32::TYPE,
                |body, left, right| {
                    let sum_left = body.read(left.clone());
                    let sum_right = body.read(right.clone());
                    let sum = body.int_add_wrapping(sum_left, sum_right);
                    let difference_left = body.read(left);
                    let difference_right = body.read(right);
                    let difference = body.int_sub_wrapping(difference_left, difference_right);
                    body.int_mul_wrapping(sum, difference)
                },
            );
            module.record2(
                portable_name!("Point"),
                (portable_name!("x"), I32::TYPE),
                (portable_name!("y"), I32::TYPE),
                |module, point| {
                    module.function2(
                        portable_name!("make_point"),
                        (portable_name!("x"), I32::TYPE),
                        (portable_name!("y"), I32::TYPE),
                        point.ty(),
                        |body, x, y| {
                            let x = body.read(x);
                            let y = body.read(y);
                            body.construct2(point, x, y)
                        },
                    );
                    module.function0(portable_name!("computed"), I32::TYPE, |body| {
                        let left = body.i32(7);
                        let right = body.i32(2);
                        body.call2(compute, left, right)
                    });
                },
            );
        });
        assert_eq!(program.checked_program().module().declarations.len(), 4);
    }

    #[test]
    fn resolves_repeated_preferred_declaration_and_field_names() {
        let program = static_program::<StaticV1>(portable_name!("collisions"), |module| {
            module.function0(portable_name!("same"), I32::TYPE, |body| body.i32(1));
            module.function0(portable_name!("same"), I32::TYPE, |body| body.i32(2));
            module.record2(
                portable_name!("same"),
                (portable_name!("value"), I32::TYPE),
                (portable_name!("value"), I32::TYPE),
                |_, _| {},
            );
        });
        let names: Vec<_> = program
            .checked_program()
            .module()
            .declarations
            .iter()
            .map(|declaration| declaration.header().name.as_str())
            .collect();
        assert_eq!(names, ["same", "same_2", "same_3"]);
    }

    #[test]
    fn every_static_v1_expression_constructor_replays_through_the_checker() {
        let program = static_program::<StaticV1>(portable_name!("all_static_v1"), |module| {
            macro_rules! unary {
                ($name:literal, $ty:expr, $method:ident) => {
                    module.function1(
                        portable_name!($name),
                        (portable_name!("value"), $ty),
                        $ty,
                        |body, value| {
                            let value = body.read(value);
                            body.$method(value)
                        },
                    );
                };
            }
            macro_rules! binary {
                ($name:literal, $ty:expr, $method:ident) => {
                    module.function2(
                        portable_name!($name),
                        (portable_name!("left"), $ty),
                        (portable_name!("right"), $ty),
                        $ty,
                        |body, left, right| {
                            let left = body.read(left);
                            let right = body.read(right);
                            body.$method(left, right)
                        },
                    );
                };
            }
            macro_rules! predicate {
                ($name:literal, $ty:expr, $method:ident) => {
                    module.function2(
                        portable_name!($name),
                        (portable_name!("left"), $ty),
                        (portable_name!("right"), $ty),
                        Bool::TYPE,
                        |body, left, right| {
                            let left = body.read(left);
                            let right = body.read(right);
                            body.$method(left, right)
                        },
                    );
                };
            }

            module.function1(
                portable_name!("bool_not"),
                (portable_name!("value"), Bool::TYPE),
                Bool::TYPE,
                |body, value| {
                    let value = body.read(value);
                    body.bool_not(value)
                },
            );
            predicate!("bool_and", Bool::TYPE, bool_and);
            predicate!("bool_or", Bool::TYPE, bool_or);
            predicate!("equal_i32", I32::TYPE, equal);
            predicate!("not_equal_text", Text::TYPE, not_equal);
            predicate!("less_i64", I64::TYPE, less);
            predicate!("less_equal_f64", F64::TYPE, less_equal);
            predicate!("greater_i32", I32::TYPE, greater);
            predicate!("greater_equal_text", Text::TYPE, greater_equal);

            unary!("i32_neg_checked", I32::TYPE, int_neg_checked);
            binary!("i32_add_checked", I32::TYPE, int_add_checked);
            binary!("i64_sub_checked", I64::TYPE, int_sub_checked);
            binary!("i32_mul_checked", I32::TYPE, int_mul_checked);
            binary!("i64_div_checked", I64::TYPE, int_div_checked);
            binary!("i32_rem_checked", I32::TYPE, int_rem_checked);
            unary!("i64_neg_wrapping", I64::TYPE, int_neg_wrapping);
            binary!("i32_add_wrapping", I32::TYPE, int_add_wrapping);
            binary!("i64_sub_wrapping", I64::TYPE, int_sub_wrapping);
            binary!("i32_mul_wrapping", I32::TYPE, int_mul_wrapping);

            unary!("float_neg", F64::TYPE, float_neg);
            binary!("float_add", F64::TYPE, float_add);
            binary!("float_sub", F64::TYPE, float_sub);
            binary!("float_mul", F64::TYPE, float_mul);
            binary!("float_div", F64::TYPE, float_div);
            binary!("float_rem", F64::TYPE, float_rem_trunc);
            binary!("string_concat", Text::TYPE, string_concat);

            let zero = module.function0(portable_name!("zero"), I64::TYPE, |body| body.i64(0));
            module.function0(portable_name!("call_zero"), I64::TYPE, |body| {
                body.call0(zero)
            });
            let identity = module.function1(
                portable_name!("identity"),
                (portable_name!("value"), Bool::TYPE),
                Bool::TYPE,
                |body, value| body.read(value),
            );
            module.function0(portable_name!("call_identity"), Bool::TYPE, |body| {
                let value = body.bool(true);
                body.call1(identity, value)
            });
            module.function0(portable_name!("text_literal"), Text::TYPE, |body| {
                body.text("portable")
            });
            module.function0(portable_name!("float_literal"), F64::TYPE, |body| {
                body.f64(1.5)
            });

            module.record1(
                portable_name!("Boxed"),
                (portable_name!("value"), I32::TYPE),
                |module, boxed| {
                    let make = module.function1(
                        portable_name!("box_value"),
                        (portable_name!("value"), I32::TYPE),
                        boxed.ty(),
                        |body, value| {
                            let value = body.read(value);
                            body.construct1(boxed, value)
                        },
                    );
                    module.function1(
                        portable_name!("unbox_value"),
                        (portable_name!("value"), boxed.ty()),
                        I32::TYPE,
                        |body, value| {
                            let value = body.read(value);
                            body.field(value, boxed.field())
                        },
                    );
                    module.function0(portable_name!("make_box"), boxed.ty(), |body| {
                        let value = body.i32(4);
                        body.call1(make, value)
                    });
                },
            );
        });
        assert!(program.checked_program().module().declarations.len() >= 30);
        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("every StaticV1 constructor lowers to CoreIR");
        portable_core_ir::verify_core(&core).expect("lowered StaticV1 CoreIR verifies");
    }
}
