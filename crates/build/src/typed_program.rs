//! Valid-by-construction portable authoring with inferred feature requirements.
//!
//! A consuming builder cannot add a declaration without changing its
//! requirement type. Expression constructors carry the requirements of their
//! complete subtrees, so a backend is callable only when its dialect supports
//! every inferred feature.
//!
//! ```compile_fail
//! use portable_build::{I32, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("mixed"), |builder| {
//!     builder.function(
//!         portable_name!("bad"), typed_list![], I32::TYPE,
//!         |body, _| {
//!             let left = body.i32(1);
//!             let right = body.text("not an integer");
//!             body.int_add_checked(left, right)
//!         },
//!     ).builder
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::{I32, parameter, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("calls"), |builder| {
//!     let added = builder.function(
//!         portable_name!("identity"),
//!         typed_list![parameter(portable_name!("value"), I32::TYPE)],
//!         I32::TYPE,
//!         |body, values| body.read(values.head),
//!     );
//!     added.builder.function(
//!         portable_name!("bad"), typed_list![], I32::TYPE,
//!         |body, _| {
//!             let wrong = body.bool(true);
//!             body.call(added.handle, typed_list![wrong])
//!         },
//!     ).builder
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::{I32, parameter, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("call_arity"), |builder| {
//!     let added = builder.function(
//!         portable_name!("identity"),
//!         typed_list![parameter(portable_name!("value"), I32::TYPE)],
//!         I32::TYPE,
//!         |body, values| body.read(values.head),
//!     );
//!     added.builder.function(
//!         portable_name!("bad"), typed_list![], I32::TYPE,
//!         |body, _| body.call(added.handle, typed_list![]),
//!     ).builder
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::{I32, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("returns"), |builder| {
//!     builder.function(
//!         portable_name!("bad"), typed_list![], I32::TYPE,
//!         |body, _| body.bool(true),
//!     ).builder
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::{I32, field, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("records"), |builder| {
//!     builder.record(
//!         portable_name!("Pair"),
//!         typed_list![
//!             field(portable_name!("left"), I32::TYPE),
//!             field(portable_name!("right"), I32::TYPE),
//!         ],
//!         |builder, pair| {
//!             builder.function(
//!                 portable_name!("bad"), typed_list![], pair.ty(),
//!                 |body, _| {
//!                     let only_one = body.i32(1);
//!                     body.construct(&pair, typed_list![only_one])
//!                 },
//!             ).builder
//!         },
//!     )
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::{I32, field, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("constructor_types"), |builder| {
//!     builder.record(
//!         portable_name!("Pair"),
//!         typed_list![
//!             field(portable_name!("left"), I32::TYPE),
//!             field(portable_name!("right"), I32::TYPE),
//!         ],
//!         |builder, pair| {
//!             builder.function(
//!                 portable_name!("bad"), typed_list![], pair.ty(),
//!                 |body, _| {
//!                     let left = body.i32(1);
//!                     let right = body.bool(false);
//!                     body.construct(&pair, typed_list![left, right])
//!                 },
//!             ).builder
//!         },
//!     )
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::{I32, field, parameter, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("field_brands"), |builder| {
//!     builder.record(
//!         portable_name!("Left"),
//!         typed_list![field(portable_name!("value"), I32::TYPE)],
//!         |builder, left| {
//!             builder.record(
//!                 portable_name!("Right"),
//!                 typed_list![field(portable_name!("value"), I32::TYPE)],
//!                 |builder, right| {
//!                     builder.function(
//!                         portable_name!("bad"),
//!                         typed_list![parameter(portable_name!("value"), right.ty())],
//!                         I32::TYPE,
//!                         |body, values| {
//!                             let value = body.read(values.head);
//!                             body.field(value, left.fields().head)
//!                         },
//!                     ).builder
//!                 },
//!             )
//!         },
//!     )
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::{I32, parameter, portable_name, typed_list, typed_program};
//! let mut escaped = None;
//! let _ = typed_program(portable_name!("body_brands"), |builder| {
//!     let builder = builder.function(
//!         portable_name!("capture"),
//!         typed_list![parameter(portable_name!("value"), I32::TYPE)],
//!         I32::TYPE,
//!         |body, values| {
//!             escaped = Some(values.head);
//!             body.i32(0)
//!         },
//!     ).builder;
//!     builder.function(
//!         portable_name!("bad"), typed_list![], I32::TYPE,
//!         |body, _| body.read(escaped.unwrap()),
//!     ).builder
//! });
//! ```
//!
//! ```compile_fail
//! use portable_build::portable_name;
//! const BAD: portable_build::PortableName = portable_name!("class");
//! ```
//!
//! ```compile_fail
//! use portable_build::{NoneRequired, TypedProgram};
//! let _ = TypedProgram::<NoneRequired> { checked: panic!(), marker: panic!() };
//! ```
//!
//! ```compile_fail
//! use portable_build::{I32, Requirements, SupportsAll, TypedProgram, portable_name, typed_list, typed_program};
//! struct EmptyDialect;
//! fn require_support<R: Requirements>(program: &TypedProgram<R>)
//! where
//!     EmptyDialect: SupportsAll<R>,
//! {
//!     let _ = program;
//! }
//! let program = typed_program(portable_name!("unsupported"), |builder| {
//!     builder.function(
//!         portable_name!("one"), typed_list![], I32::TYPE,
//!         |body, _| body.i32(1),
//!     ).builder
//! });
//! require_support(&program);
//! ```

use std::{cell::Cell, marker::PhantomData};

use portable_check::v0::CheckedProgram;

use crate::{
    BodyBuilder, FunctionId, ModuleBuilder, Operation, Parameter, RecordFieldId, RecordId, Type,
    Value, Visibility,
};

mod sealed {
    pub trait Feature {}
    pub trait Requirements {}
    pub trait Parameters {}
    pub trait Arguments {}
    pub trait Fields {}
    pub trait Equatable {}
    pub trait Ordered {}
    pub trait Integer {}
}

/// A feature which can be required by typed portable syntax.
pub trait Feature: sealed::Feature {}

/// A structural compile-time tree of inferred requirements.
pub trait Requirements: sealed::Requirements {}

/// The empty requirement tree.
#[derive(Clone, Copy, Debug)]
pub struct NoneRequired;

/// One required feature followed by another requirement tree.
#[derive(Clone, Copy, Debug)]
pub struct Requires<F: Feature, Tail: Requirements = NoneRequired>(PhantomData<(F, Tail)>);

/// The conjunction of two requirement trees.
#[derive(Clone, Copy, Debug)]
pub struct All<Left: Requirements, Right: Requirements>(PhantomData<(Left, Right)>);

impl sealed::Requirements for NoneRequired {}
impl Requirements for NoneRequired {}
impl<F: Feature, Tail: Requirements> sealed::Requirements for Requires<F, Tail> {}
impl<F: Feature, Tail: Requirements> Requirements for Requires<F, Tail> {}
impl<Left: Requirements, Right: Requirements> sealed::Requirements for All<Left, Right> {}
impl<Left: Requirements, Right: Requirements> Requirements for All<Left, Right> {}

/// Compile-time evidence that a dialect implements one portable feature.
pub trait Supports<F: Feature> {}

/// Compile-time evidence that a dialect implements a complete requirement tree.
pub trait SupportsAll<R: Requirements> {}

impl<D> SupportsAll<NoneRequired> for D {}

impl<D, F, Tail> SupportsAll<Requires<F, Tail>> for D
where
    F: Feature,
    Tail: Requirements,
    D: Supports<F> + SupportsAll<Tail>,
{
}

impl<D, Left, Right> SupportsAll<All<Left, Right>> for D
where
    Left: Requirements,
    Right: Requirements,
    D: SupportsAll<Left> + SupportsAll<Right>,
{
}

macro_rules! feature_markers {
    ($($(#[$meta:meta])* $name:ident),+ $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Clone, Copy, Debug)]
            pub enum $name {}
            impl sealed::Feature for $name {}
            impl Feature for $name {}
        )+
    };
}

feature_markers!(
    /// Function declarations.
    Functions,
    /// Reads through callable-branded locals.
    LocalReads,
    /// Calls through typed function handles.
    FunctionCalls,
    /// Record declarations.
    Records,
    /// Exact typed record construction.
    RecordConstruction,
    /// Projection through declaration-branded fields.
    FieldAccess,
    /// Boolean values.
    BoolValues,
    /// Signed 32-bit values.
    I32Values,
    /// Signed 64-bit values.
    I64Values,
    /// IEEE-754 binary64 values.
    F64Values,
    /// Unicode text values.
    TextValues,
    /// Boolean operations.
    BooleanLogic,
    /// Equality operations.
    Equality,
    /// Ordered comparisons.
    Ordering,
    /// Checked signed-integer arithmetic.
    CheckedIntegerArithmetic,
    /// Two's-complement wrapping signed-integer arithmetic.
    WrappingIntegerArithmetic,
    /// Binary64 arithmetic.
    FloatingPointArithmetic,
    /// Text concatenation.
    StringConcatenation,
);

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
        "invalid first identifier character"
    );
    let mut index = 1;
    while index < bytes.len() {
        assert!(
            is_ascii_continue(bytes[index]),
            "invalid identifier character"
        );
        index += 1;
    }
    assert!(!is_protected(value), "identifier is protected");
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

/// The empty structural typed list.
#[derive(Clone, Copy, Debug, Default)]
pub struct Nil;

/// One element followed by another structural typed list.
#[derive(Clone, Copy, Debug)]
pub struct Cons<Head, Tail> {
    pub head: Head,
    pub tail: Tail,
}

impl<Head, Tail> Cons<Head, Tail> {
    pub const fn new(head: Head, tail: Tail) -> Self {
        Self { head, tail }
    }
}

/// Builds a recursively typed list without imposing an arity cap.
#[macro_export]
macro_rules! typed_list {
    () => { $crate::Nil };
    ($head:expr $(, $tail:expr)* $(,)?) => {
        $crate::Cons::new($head, $crate::typed_list!($($tail),*))
    };
}

/// A typed witness for a portable value type and its representation feature.
pub struct TypedType<T, R: Requirements> {
    ir: Type,
    marker: PhantomData<fn() -> (T, R)>,
}

impl<T, R: Requirements> Clone for TypedType<T, R> {
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
    ($marker:ident, $type_fn:ident, $feature:ident) => {
        impl $marker {
            pub const TYPE: TypedType<Self, Requires<$feature>> = TypedType {
                ir: Type::$type_fn(),
                marker: PhantomData,
            };
        }
    };
}

primitive_type!(Bool, bool, BoolValues);
primitive_type!(I32, i32, I32Values);
primitive_type!(I64, i64, I64Values);
primitive_type!(F64, f64, F64Values);
primitive_type!(Text, string, TextValues);

/// Values admitted by equality operations.
pub trait TypedEquatable: sealed::Equatable {}
/// Values admitted by ordered comparisons.
pub trait TypedOrdered: sealed::Ordered {}
/// Values admitted by integer operations.
pub trait TypedInteger: sealed::Integer {}

macro_rules! equatable {
    ($($type:ty),+ $(,)?) => {$(
        impl sealed::Equatable for $type {}
        impl TypedEquatable for $type {}
    )+};
}

equatable!(Bool, I32, I64, F64, Text);
impl sealed::Ordered for I32 {}
impl TypedOrdered for I32 {}
impl sealed::Ordered for I64 {}
impl TypedOrdered for I64 {}
impl sealed::Ordered for F64 {}
impl TypedOrdered for F64 {}
impl sealed::Ordered for Text {}
impl TypedOrdered for Text {}
impl sealed::Integer for I32 {}
impl TypedInteger for I32 {}
impl sealed::Integer for I64 {}
impl TypedInteger for I64 {}

/// A record value branded with its module and exact declaration.
pub struct RecordValue<'module, 'record>(PhantomData<(Cell<&'module ()>, Cell<&'record ()>)>);

impl sealed::Equatable for RecordValue<'_, '_> {}
impl TypedEquatable for RecordValue<'_, '_> {}

/// A typed expression owned by one callable body with inferred requirements.
type InvariantExpressionBrand<'module, 'body, T, R> = fn(&'module (), &'body (), T, R) -> (T, R);

pub struct TypedExpr<'module, 'body, T, R: Requirements> {
    node: TypedNode,
    marker: PhantomData<InvariantExpressionBrand<'module, 'body, T, R>>,
}

enum TypedNode {
    Literal(Value),
    Local(String),
    Record {
        record: RecordId,
        fields: Vec<(RecordFieldId, TypedNode)>,
    },
    Field {
        base: Box<TypedNode>,
        field: RecordFieldId,
    },
    Call {
        function: FunctionId,
        arguments: Vec<TypedNode>,
    },
    Intrinsic {
        operation: Operation,
        arguments: Vec<TypedNode>,
    },
}

/// A typed parameter specification.
pub struct TypedParameter<T, R: Requirements> {
    name: PortableName,
    ty: TypedType<T, R>,
}

pub const fn parameter<T, R: Requirements>(
    name: PortableName,
    ty: TypedType<T, R>,
) -> TypedParameter<T, R> {
    TypedParameter { name, ty }
}

/// A typed local issued by one callable body.
pub struct TypedLocal<'module, 'body, T, R: Requirements> {
    name: String,
    marker: PhantomData<InvariantExpressionBrand<'module, 'body, T, R>>,
}

impl<T, R: Requirements> Clone for TypedLocal<'_, '_, T, R> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            marker: PhantomData,
        }
    }
}

/// A recursive list of typed function parameters.
pub trait ParameterList: sealed::Parameters {
    type Types;
    type Requirements: Requirements;
    type Locals<'module, 'body>;

    #[doc(hidden)]
    fn append_raw(self, output: &mut Vec<(PortableName, Type)>);
    #[doc(hidden)]
    fn make_locals<'module, 'body>(
        names: &mut std::vec::IntoIter<String>,
    ) -> Self::Locals<'module, 'body>;
}

impl sealed::Parameters for Nil {}
impl ParameterList for Nil {
    type Types = Nil;
    type Requirements = NoneRequired;
    type Locals<'module, 'body> = Nil;

    fn append_raw(self, _output: &mut Vec<(PortableName, Type)>) {}

    fn make_locals<'module, 'body>(
        _names: &mut std::vec::IntoIter<String>,
    ) -> Self::Locals<'module, 'body> {
        Nil
    }
}

impl<T, R, Tail> sealed::Parameters for Cons<TypedParameter<T, R>, Tail>
where
    R: Requirements,
    Tail: ParameterList,
{
}

impl<T, R, Tail> ParameterList for Cons<TypedParameter<T, R>, Tail>
where
    R: Requirements,
    Tail: ParameterList,
{
    type Types = Cons<T, Tail::Types>;
    type Requirements = All<R, Tail::Requirements>;
    type Locals<'module, 'body> =
        Cons<TypedLocal<'module, 'body, T, R>, Tail::Locals<'module, 'body>>;

    fn append_raw(self, output: &mut Vec<(PortableName, Type)>) {
        output.push((self.head.name, self.head.ty.ir));
        self.tail.append_raw(output);
    }

    fn make_locals<'module, 'body>(
        names: &mut std::vec::IntoIter<String>,
    ) -> Self::Locals<'module, 'body> {
        Cons::new(
            TypedLocal {
                name: names.next().expect("typed parameter name"),
                marker: PhantomData,
            },
            Tail::make_locals(names),
        )
    }
}

/// A recursive list of typed call or constructor expressions.
pub trait ArgumentList: sealed::Arguments {
    type Types;
    type Requirements: Requirements;

    #[doc(hidden)]
    fn into_nodes(self) -> ArgumentNodes;
}

/// Opaque lowering payload used only by the private checked-IR bridge.
#[doc(hidden)]
pub struct ArgumentNodes(Vec<TypedNode>);

impl sealed::Arguments for Nil {}
impl ArgumentList for Nil {
    type Types = Nil;
    type Requirements = NoneRequired;

    fn into_nodes(self) -> ArgumentNodes {
        ArgumentNodes(Vec::new())
    }
}

impl<'module, 'body, T, R, Tail> sealed::Arguments for Cons<TypedExpr<'module, 'body, T, R>, Tail>
where
    R: Requirements,
    Tail: ArgumentList,
{
}

impl<'module, 'body, T, R, Tail> ArgumentList for Cons<TypedExpr<'module, 'body, T, R>, Tail>
where
    R: Requirements,
    Tail: ArgumentList,
{
    type Types = Cons<T, Tail::Types>;
    type Requirements = All<R, Tail::Requirements>;

    fn into_nodes(self) -> ArgumentNodes {
        let mut nodes = vec![self.head.node];
        nodes.extend(self.tail.into_nodes().0);
        ArgumentNodes(nodes)
    }
}

/// A typed field specification.
pub struct TypedFieldSpec<T, R: Requirements> {
    name: PortableName,
    ty: TypedType<T, R>,
}

pub fn field<T, R: Requirements>(name: PortableName, ty: TypedType<T, R>) -> TypedFieldSpec<T, R> {
    TypedFieldSpec { name, ty }
}

type InvariantRecordBrand<'module, 'record, T> = (Cell<&'module ()>, Cell<&'record ()>, fn(T) -> T);

/// A field handle tied to one exact record declaration.
pub struct TypedField<'module, 'record, T> {
    raw: RecordFieldId,
    marker: PhantomData<InvariantRecordBrand<'module, 'record, T>>,
}

impl<T> Copy for TypedField<'_, '_, T> {}
impl<T> Clone for TypedField<'_, '_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A recursive list of typed record fields.
pub trait FieldList: sealed::Fields {
    type Types;
    type Requirements: Requirements;
    type Handles<'module, 'record>;

    #[doc(hidden)]
    fn append_raw(self, output: &mut Vec<(PortableName, Type)>);
    #[doc(hidden)]
    fn make_handles<'module, 'record>(
        fields: &mut std::vec::IntoIter<RecordFieldId>,
    ) -> Self::Handles<'module, 'record>;
}

impl sealed::Fields for Nil {}
impl FieldList for Nil {
    type Types = Nil;
    type Requirements = NoneRequired;
    type Handles<'module, 'record> = Nil;

    fn append_raw(self, _output: &mut Vec<(PortableName, Type)>) {}

    fn make_handles<'module, 'record>(
        _fields: &mut std::vec::IntoIter<RecordFieldId>,
    ) -> Self::Handles<'module, 'record> {
        Nil
    }
}

impl<T, R, Tail> sealed::Fields for Cons<TypedFieldSpec<T, R>, Tail>
where
    R: Requirements,
    Tail: FieldList,
{
}

impl<T, R, Tail> FieldList for Cons<TypedFieldSpec<T, R>, Tail>
where
    R: Requirements,
    Tail: FieldList,
{
    type Types = Cons<T, Tail::Types>;
    type Requirements = All<R, Tail::Requirements>;
    type Handles<'module, 'record> =
        Cons<TypedField<'module, 'record, T>, Tail::Handles<'module, 'record>>;

    fn append_raw(self, output: &mut Vec<(PortableName, Type)>) {
        output.push((self.head.name, self.head.ty.ir));
        self.tail.append_raw(output);
    }

    fn make_handles<'module, 'record>(
        fields: &mut std::vec::IntoIter<RecordFieldId>,
    ) -> Self::Handles<'module, 'record> {
        Cons::new(
            TypedField {
                raw: fields.next().expect("typed record field"),
                marker: PhantomData,
            },
            Tail::make_handles(fields),
        )
    }
}

/// A typed function handle with an exact recursive argument list.
pub struct TypedFunction<'module, Arguments, Result> {
    raw: FunctionId,
    marker: PhantomData<fn(&'module (), Arguments) -> Result>,
}

impl<A, R> Copy for TypedFunction<'_, A, R> {}
impl<A, R> Clone for TypedFunction<'_, A, R> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A record constructor and its exact branded field-handle list.
pub struct TypedRecord<'module, 'record, Types, Handles> {
    raw: RecordId,
    field_ids: Vec<RecordFieldId>,
    fields: Handles,
    marker: PhantomData<InvariantRecordBrand<'module, 'record, Types>>,
}

impl<'module, 'record, Types, Handles> TypedRecord<'module, 'record, Types, Handles> {
    pub fn ty(&self) -> TypedType<RecordValue<'module, 'record>, Requires<Records>> {
        TypedType {
            ir: Type::named(self.raw),
            marker: PhantomData,
        }
    }

    pub const fn fields(&self) -> &Handles {
        &self.fields
    }
}

/// A typed portable program with inferred requirement tree `R`.
pub struct TypedProgram<R: Requirements> {
    checked: CheckedProgram,
    marker: PhantomData<fn() -> R>,
}

impl<R: Requirements> TypedProgram<R> {
    /// Read-only bridge for target adapters. Callers cannot replace its data.
    pub fn checked_program(&self) -> &CheckedProgram {
        &self.checked
    }
}

/// The result of adding a declaration which also issues a typed handle.
pub struct Added<Builder, Handle> {
    pub builder: Builder,
    pub handle: Handle,
}

/// A consuming builder whose type contains all declaration requirements.
type InvariantBuilderBrand<'module, R> = (Cell<&'module ()>, fn() -> R);

pub struct ProgramBuilder<'module, R: Requirements> {
    dynamic: ModuleBuilder,
    names: NameAllocator,
    marker: PhantomData<InvariantBuilderBrand<'module, R>>,
}

type FunctionRequirement<Existing, Parameters, Result, Body> =
    All<Existing, All<Requires<Functions>, All<Parameters, All<Result, Body>>>>;
type RecordRequirement<Existing, Fields> = All<Existing, All<Requires<Records>, Fields>>;
type FunctionAdded<'module, Existing, Parameters, Output, OutputRequirements, BodyRequirements> =
    Added<
        ProgramBuilder<
            'module,
            FunctionRequirement<
                Existing,
                <Parameters as ParameterList>::Requirements,
                OutputRequirements,
                BodyRequirements,
            >,
        >,
        TypedFunction<'module, <Parameters as ParameterList>::Types, Output>,
    >;

/// Builds a typed program and infers every feature from construction.
pub fn typed_program<R: Requirements>(
    name: PortableName,
    build: impl for<'module> FnOnce(ProgramBuilder<'module, NoneRequired>) -> ProgramBuilder<'module, R>,
) -> TypedProgram<R> {
    let builder = ProgramBuilder {
        dynamic: ModuleBuilder::new(name.preferred()),
        names: NameAllocator::default(),
        marker: PhantomData,
    };
    let builder = build(builder);
    let checked = builder
        .dynamic
        .finish()
        .unwrap_or_else(|diagnostics| panic!("TypedProgram invariant failure: {diagnostics:#?}"));
    TypedProgram {
        checked,
        marker: PhantomData,
    }
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

impl<'module, Existing: Requirements> ProgramBuilder<'module, Existing> {
    pub fn function<P, Output, OutputRequirements, BodyRequirements>(
        mut self,
        name: PortableName,
        parameters: P,
        result: TypedType<Output, OutputRequirements>,
        build: impl for<'body> FnOnce(
            &mut TypedBody<'module, 'body>,
            P::Locals<'module, 'body>,
        ) -> TypedExpr<'module, 'body, Output, BodyRequirements>,
    ) -> FunctionAdded<'module, Existing, P, Output, OutputRequirements, BodyRequirements>
    where
        P: ParameterList,
        OutputRequirements: Requirements,
        BodyRequirements: Requirements,
    {
        let name = self.names.allocate(name);
        let mut raw_parameters = Vec::new();
        parameters.append_raw(&mut raw_parameters);
        let mut parameter_names = NameAllocator::default();
        let raw_parameters = raw_parameters
            .into_iter()
            .map(|(name, ty)| (parameter_names.allocate(name), ty))
            .collect::<Vec<_>>();
        let local_names = raw_parameters
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let raw = self
            .dynamic
            .function(name, Visibility::Public, vec![], |function| {
                for (name, ty) in raw_parameters {
                    function.parameter(Parameter::new(name, ty));
                }
                function.returns(result.ir).body(|body| {
                    let mut names = local_names.into_iter();
                    let locals = P::make_locals(&mut names);
                    let mut typed = TypedBody {
                        marker: PhantomData,
                    };
                    let result = build(&mut typed, locals);
                    let result = lower_expression(body, result.node);
                    body.block([], Some(result))
                });
            });
        Added {
            builder: ProgramBuilder {
                dynamic: self.dynamic,
                names: self.names,
                marker: PhantomData,
            },
            handle: TypedFunction {
                raw,
                marker: PhantomData,
            },
        }
    }

    pub fn record<Fields, OutputRequirements>(
        mut self,
        name: PortableName,
        fields: Fields,
        then: impl for<'record> FnOnce(
            ProgramBuilder<'module, RecordRequirement<Existing, Fields::Requirements>>,
            TypedRecord<'module, 'record, Fields::Types, Fields::Handles<'module, 'record>>,
        ) -> ProgramBuilder<'module, OutputRequirements>,
    ) -> ProgramBuilder<'module, OutputRequirements>
    where
        Fields: FieldList,
        OutputRequirements: Requirements,
    {
        let name = self.names.allocate(name);
        let mut raw_fields = Vec::new();
        fields.append_raw(&mut raw_fields);
        let mut field_names = NameAllocator::default();
        let raw_fields = raw_fields
            .into_iter()
            .map(|(name, ty)| (field_names.allocate(name), ty))
            .collect::<Vec<_>>();
        let (raw, field_ids) = self
            .dynamic
            .record(name, Visibility::Public, vec![], |record| {
                raw_fields
                    .into_iter()
                    .map(|(name, ty)| record.field(name, ty, vec![]))
                    .collect::<Vec<_>>()
            });
        let handles = Fields::make_handles(&mut field_ids.clone().into_iter());
        then(
            ProgramBuilder {
                dynamic: self.dynamic,
                names: self.names,
                marker: PhantomData,
            },
            TypedRecord {
                raw,
                field_ids,
                fields: handles,
                marker: PhantomData,
            },
        )
    }
}

/// The only expression factory for one branded function body.
pub struct TypedBody<'module, 'body> {
    marker: PhantomData<(Cell<&'module ()>, Cell<&'body ()>)>,
}

type With<F, R> = All<Requires<F>, R>;
type WithTwo<F, Left, Right> = All<Requires<F>, All<Left, Right>>;

impl<'module, 'body> TypedBody<'module, 'body> {
    fn expression<T, R: Requirements>(&self, node: TypedNode) -> TypedExpr<'module, 'body, T, R> {
        TypedExpr {
            node,
            marker: PhantomData,
        }
    }

    pub fn read<T, R: Requirements>(
        &mut self,
        local: TypedLocal<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, With<LocalReads, R>> {
        self.expression(TypedNode::Local(local.name))
    }

    pub fn bool(&mut self, value: bool) -> TypedExpr<'module, 'body, Bool, Requires<BoolValues>> {
        self.expression(TypedNode::Literal(Value::bool(value)))
    }

    pub fn i32(&mut self, value: i32) -> TypedExpr<'module, 'body, I32, Requires<I32Values>> {
        self.expression(TypedNode::Literal(Value::i32(value)))
    }

    pub fn i64(&mut self, value: i64) -> TypedExpr<'module, 'body, I64, Requires<I64Values>> {
        self.expression(TypedNode::Literal(Value::i64(value)))
    }

    pub fn f64(&mut self, value: f64) -> TypedExpr<'module, 'body, F64, Requires<F64Values>> {
        self.expression(TypedNode::Literal(Value::f64(value)))
    }

    pub fn text(
        &mut self,
        value: impl Into<String>,
    ) -> TypedExpr<'module, 'body, Text, Requires<TextValues>> {
        self.expression(TypedNode::Literal(Value::string(value)))
    }

    pub fn construct<'record, Types, Handles, Arguments>(
        &mut self,
        record: &TypedRecord<'module, 'record, Types, Handles>,
        arguments: Arguments,
    ) -> TypedExpr<
        'module,
        'body,
        RecordValue<'module, 'record>,
        With<RecordConstruction, Arguments::Requirements>,
    >
    where
        Arguments: ArgumentList<Types = Types>,
    {
        let nodes = arguments.into_nodes().0;
        assert_eq!(
            record.field_ids.len(),
            nodes.len(),
            "typed record arity invariant"
        );
        self.expression(TypedNode::Record {
            record: record.raw,
            fields: record.field_ids.iter().copied().zip(nodes).collect(),
        })
    }

    pub fn field<'record, T, BaseRequirements>(
        &mut self,
        base: TypedExpr<'module, 'body, RecordValue<'module, 'record>, BaseRequirements>,
        field: TypedField<'module, 'record, T>,
    ) -> TypedExpr<'module, 'body, T, With<FieldAccess, BaseRequirements>>
    where
        BaseRequirements: Requirements,
    {
        self.expression(TypedNode::Field {
            base: Box::new(base.node),
            field: field.raw,
        })
    }

    pub fn call<Arguments, Output>(
        &mut self,
        function: TypedFunction<'module, Arguments::Types, Output>,
        arguments: Arguments,
    ) -> TypedExpr<'module, 'body, Output, With<FunctionCalls, Arguments::Requirements>>
    where
        Arguments: ArgumentList,
    {
        self.expression(TypedNode::Call {
            function: function.raw,
            arguments: arguments.into_nodes().0,
        })
    }

    fn unary<A, Output, FeatureMarker, InputRequirements>(
        &mut self,
        operation: Operation,
        value: TypedExpr<'module, 'body, A, InputRequirements>,
    ) -> TypedExpr<'module, 'body, Output, With<FeatureMarker, InputRequirements>>
    where
        FeatureMarker: Feature,
        InputRequirements: Requirements,
    {
        self.expression(TypedNode::Intrinsic {
            operation,
            arguments: vec![value.node],
        })
    }

    fn binary<A, B, Output, FeatureMarker, LeftRequirements, RightRequirements>(
        &mut self,
        operation: Operation,
        left: TypedExpr<'module, 'body, A, LeftRequirements>,
        right: TypedExpr<'module, 'body, B, RightRequirements>,
    ) -> TypedExpr<
        'module,
        'body,
        Output,
        WithTwo<FeatureMarker, LeftRequirements, RightRequirements>,
    >
    where
        FeatureMarker: Feature,
        LeftRequirements: Requirements,
        RightRequirements: Requirements,
    {
        self.expression(TypedNode::Intrinsic {
            operation,
            arguments: vec![left.node, right.node],
        })
    }

    pub fn bool_not<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Bool, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<BooleanLogic, R>> {
        self.unary(Operation::BoolNot, value)
    }

    pub fn bool_and<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, Bool, L>,
        right: TypedExpr<'module, 'body, Bool, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<BooleanLogic, L, R>> {
        self.binary(Operation::BoolAnd, left, right)
    }

    pub fn bool_or<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, Bool, L>,
        right: TypedExpr<'module, 'body, Bool, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<BooleanLogic, L, R>> {
        self.binary(Operation::BoolOr, left, right)
    }

    pub fn equal<T: TypedEquatable, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<Equality, L, R>> {
        self.binary(Operation::Equal, left, right)
    }

    pub fn not_equal<T: TypedEquatable, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<Equality, L, R>> {
        self.binary(Operation::NotEqual, left, right)
    }

    pub fn less<T: TypedOrdered, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<Ordering, L, R>> {
        self.binary(Operation::Less, left, right)
    }

    pub fn less_equal<T: TypedOrdered, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<Ordering, L, R>> {
        self.binary(Operation::LessEqual, left, right)
    }

    pub fn greater<T: TypedOrdered, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<Ordering, L, R>> {
        self.binary(Operation::Greater, left, right)
    }

    pub fn greater_equal<T: TypedOrdered, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<Ordering, L, R>> {
        self.binary(Operation::GreaterEqual, left, right)
    }

    pub fn int_neg_checked<T: TypedInteger, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, With<CheckedIntegerArithmetic, R>> {
        self.unary(Operation::IntNegChecked, value)
    }

    pub fn int_add_checked<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<CheckedIntegerArithmetic, L, R>> {
        self.binary(Operation::IntAddChecked, left, right)
    }

    pub fn int_sub_checked<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<CheckedIntegerArithmetic, L, R>> {
        self.binary(Operation::IntSubChecked, left, right)
    }

    pub fn int_mul_checked<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<CheckedIntegerArithmetic, L, R>> {
        self.binary(Operation::IntMulChecked, left, right)
    }

    pub fn int_div_checked<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<CheckedIntegerArithmetic, L, R>> {
        self.binary(Operation::IntDivChecked, left, right)
    }

    pub fn int_rem_checked<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<CheckedIntegerArithmetic, L, R>> {
        self.binary(Operation::IntRemChecked, left, right)
    }

    pub fn int_neg_wrapping<T: TypedInteger, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, With<WrappingIntegerArithmetic, R>> {
        self.unary(Operation::IntNegWrapping, value)
    }

    pub fn int_add_wrapping<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<WrappingIntegerArithmetic, L, R>> {
        self.binary(Operation::IntAddWrapping, left, right)
    }

    pub fn int_sub_wrapping<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<WrappingIntegerArithmetic, L, R>> {
        self.binary(Operation::IntSubWrapping, left, right)
    }

    pub fn int_mul_wrapping<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<WrappingIntegerArithmetic, L, R>> {
        self.binary(Operation::IntMulWrapping, left, right)
    }

    pub fn float_neg<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, F64, With<FloatingPointArithmetic, R>> {
        self.unary(Operation::FloatNeg, value)
    }

    pub fn float_add<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, F64, L>,
        right: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, F64, WithTwo<FloatingPointArithmetic, L, R>> {
        self.binary(Operation::FloatAdd, left, right)
    }

    pub fn float_sub<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, F64, L>,
        right: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, F64, WithTwo<FloatingPointArithmetic, L, R>> {
        self.binary(Operation::FloatSub, left, right)
    }

    pub fn float_mul<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, F64, L>,
        right: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, F64, WithTwo<FloatingPointArithmetic, L, R>> {
        self.binary(Operation::FloatMul, left, right)
    }

    pub fn float_div<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, F64, L>,
        right: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, F64, WithTwo<FloatingPointArithmetic, L, R>> {
        self.binary(Operation::FloatDiv, left, right)
    }

    pub fn float_rem_trunc<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, F64, L>,
        right: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, F64, WithTwo<FloatingPointArithmetic, L, R>> {
        self.binary(Operation::FloatRemTrunc, left, right)
    }

    pub fn string_concat<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, Text, L>,
        right: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Text, WithTwo<StringConcatenation, L, R>> {
        self.binary(Operation::StringConcat, left, right)
    }
}

fn lower_expression(body: &mut BodyBuilder<'_>, node: TypedNode) -> crate::Expr {
    match node {
        TypedNode::Literal(value) => body.literal(value),
        TypedNode::Local(name) => body.local(name),
        TypedNode::Record { record, fields } => {
            let fields = fields
                .into_iter()
                .map(|(field, value)| (field, lower_expression(body, value)))
                .collect::<Vec<_>>();
            body.record(record, fields)
        }
        TypedNode::Field { base, field } => {
            let base = lower_expression(body, *base);
            body.field(base, field)
        }
        TypedNode::Call {
            function,
            arguments,
        } => {
            let arguments = arguments
                .into_iter()
                .map(|argument| lower_expression(body, argument))
                .collect::<Vec<_>>();
            body.call(function, arguments)
        }
        TypedNode::Intrinsic {
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
    fn infers_arbitrary_typed_function_and_record_shapes() {
        let program = typed_program(portable_name!("typed_fixture"), |builder| {
            let added = builder.function(
                portable_name!("sum_three"),
                typed_list![
                    parameter(portable_name!("first"), I32::TYPE),
                    parameter(portable_name!("second"), I32::TYPE),
                    parameter(portable_name!("third"), I32::TYPE),
                ],
                I32::TYPE,
                |body, values| {
                    let first = body.read(values.head);
                    let second = body.read(values.tail.head);
                    let third = body.read(values.tail.tail.head);
                    let partial = body.int_add_wrapping(first, second);
                    body.int_add_wrapping(partial, third)
                },
            );
            let sum_three = added.handle;
            added.builder.record(
                portable_name!("Point3"),
                typed_list![
                    field(portable_name!("x"), I32::TYPE),
                    field(portable_name!("y"), I32::TYPE),
                    field(portable_name!("z"), I32::TYPE),
                ],
                |builder, point| {
                    let added = builder.function(
                        portable_name!("make_point"),
                        typed_list![
                            parameter(portable_name!("x"), I32::TYPE),
                            parameter(portable_name!("y"), I32::TYPE),
                            parameter(portable_name!("z"), I32::TYPE),
                        ],
                        point.ty(),
                        |body, values| {
                            let x = body.read(values.head);
                            let y = body.read(values.tail.head);
                            let z = body.read(values.tail.tail.head);
                            body.construct(&point, typed_list![x, y, z])
                        },
                    );
                    let added = added.builder.function(
                        portable_name!("computed"),
                        typed_list![],
                        I32::TYPE,
                        |body, _| {
                            let one = body.i32(1);
                            let two = body.i32(2);
                            let three = body.i32(3);
                            body.call(sum_three, typed_list![one, two, three])
                        },
                    );
                    added
                        .builder
                        .function(
                            portable_name!("project_x"),
                            typed_list![parameter(portable_name!("point"), point.ty())],
                            I32::TYPE,
                            |body, values| {
                                let point_value = body.read(values.head);
                                body.field(point_value, point.fields().head)
                            },
                        )
                        .builder
                },
            )
        });
        assert_eq!(program.checked_program().module().declarations.len(), 5);
        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("typed program lowers to CoreIR");
        portable_core_ir::verify_core(&core).expect("typed CoreIR verifies");
    }

    #[test]
    fn every_exposed_expression_constructor_replays_through_core_ir() {
        let program = typed_program(portable_name!("all_features"), |builder| {
            let builder = builder
                .function(
                    portable_name!("boolean_logic"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let left = body.bool(true);
                        let left = body.bool_not(left);
                        let right = body.bool(false);
                        let both = body.bool_and(left, right);
                        let fallback = body.bool(true);
                        body.bool_or(both, fallback)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("equality"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let left = body.i64(1);
                        let right = body.i64(1);
                        let equal = body.equal(left, right);
                        let left = body.text("left");
                        let right = body.text("right");
                        let unequal = body.not_equal(left, right);
                        body.bool_and(equal, unequal)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("ordering"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let one = body.i32(1);
                        let two = body.i32(2);
                        let left = body.less(one, two);
                        let two_left = body.i64(2);
                        let two_right = body.i64(2);
                        let right = body.less_equal(two_left, two_right);
                        let first = body.bool_and(left, right);
                        let three = body.f64(3.0);
                        let two = body.f64(2.0);
                        let left = body.greater(three, two);
                        let z = body.text("z");
                        let a = body.text("a");
                        let right = body.greater_equal(z, a);
                        let second = body.bool_and(left, right);
                        body.bool_and(first, second)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("checked_integer"),
                    typed_list![],
                    I64::TYPE,
                    |body, _| {
                        let one = body.i64(1);
                        let value = body.int_neg_checked(one);
                        let ten = body.i64(10);
                        let value = body.int_add_checked(value, ten);
                        let two = body.i64(2);
                        let value = body.int_sub_checked(value, two);
                        let three = body.i64(3);
                        let value = body.int_mul_checked(value, three);
                        let two = body.i64(2);
                        let value = body.int_div_checked(value, two);
                        let five = body.i64(5);
                        body.int_rem_checked(value, five)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("wrapping_integer"),
                    typed_list![],
                    I32::TYPE,
                    |body, _| {
                        let one = body.i32(1);
                        let value = body.int_neg_wrapping(one);
                        let ten = body.i32(10);
                        let value = body.int_add_wrapping(value, ten);
                        let two = body.i32(2);
                        let value = body.int_sub_wrapping(value, two);
                        let three = body.i32(3);
                        body.int_mul_wrapping(value, three)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("floating_point"),
                    typed_list![],
                    F64::TYPE,
                    |body, _| {
                        let one = body.f64(1.0);
                        let value = body.float_neg(one);
                        let ten = body.f64(10.0);
                        let value = body.float_add(value, ten);
                        let two = body.f64(2.0);
                        let value = body.float_sub(value, two);
                        let three = body.f64(3.0);
                        let value = body.float_mul(value, three);
                        let two = body.f64(2.0);
                        let value = body.float_div(value, two);
                        let five = body.f64(5.0);
                        body.float_rem_trunc(value, five)
                    },
                )
                .builder;
            builder
                .function(
                    portable_name!("concatenate"),
                    typed_list![],
                    Text::TYPE,
                    |body, _| {
                        let left = body.text("poly");
                        let right = body.text("rust");
                        body.string_concat(left, right)
                    },
                )
                .builder
        });

        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("every typed constructor lowers to CoreIR");
        portable_core_ir::verify_core(&core).expect("every typed constructor verifies");
    }

    #[test]
    fn resolves_repeated_preferred_names_deterministically() {
        let program = typed_program(portable_name!("collisions"), |builder| {
            let builder = builder
                .function(
                    portable_name!("same"),
                    typed_list![],
                    I32::TYPE,
                    |body, _| body.i32(1),
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("same"),
                    typed_list![],
                    I32::TYPE,
                    |body, _| body.i32(2),
                )
                .builder;
            builder.record(
                portable_name!("same"),
                typed_list![
                    field(portable_name!("value"), I32::TYPE),
                    field(portable_name!("value"), I32::TYPE),
                    field(portable_name!("value"), I32::TYPE),
                ],
                |builder, _| builder,
            )
        });
        let names = program
            .checked_program()
            .module()
            .declarations
            .iter()
            .map(|declaration| declaration.header().name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["same", "same_2", "same_3"]);
    }
}
