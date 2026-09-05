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
//! Constant initializers return the exact declared type:
//!
//! ```compile_fail
//! use portable_build::{I32, portable_name, typed_program};
//! let _ = typed_program(portable_name!("constant_type"), |builder| {
//!     builder.constant(portable_name!("VALUE"), I32::TYPE, |body| body.bool(true)).builder
//! });
//! ```
//!
//! Transparent aliases retain distinct declaration brands in the typed AST:
//!
//! ```compile_fail
//! use portable_build::{I64, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("alias_brand"), |builder| {
//!     builder.alias(portable_name!("First"), I64::TYPE, |builder, first| {
//!         builder.alias(portable_name!("Second"), I64::TYPE, |builder, second| {
//!             builder.function(portable_name!("bad"), typed_list![], second.ty(), |body, _| {
//!                 let value = body.i64(7);
//!                 body.alias_wrap(&first, value)
//!             }).builder
//!         })
//!     })
//! });
//! ```
//!
//! Conditional branches must produce the same value type:
//!
//! ```compile_fail
//! use portable_build::{I32, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("conditional_type"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![], I32::TYPE, |body, _| {
//!         let condition = body.bool(true);
//!         let yes = body.i32(1);
//!         let no = body.text("no");
//!         body.if_else(condition, yes, no)
//!     }).builder
//! });
//! ```
//!
//! A lexical binding handle cannot escape its continuation:
//!
//! ```compile_fail
//! use portable_build::{I32, portable_name, typed_list, typed_program};
//! let mut escaped = None;
//! let _ = typed_program(portable_name!("binding_escape"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![], I32::TYPE, |body, _| {
//!         let value = body.i32(1);
//!         body.let_value(portable_name!("value"), value, |body, local| {
//!             escaped = Some(local);
//!             body.i32(0)
//!         })
//!     }).builder
//! });
//! ```
//!
//! A bounded-loop iteration handle cannot escape its body:
//!
//! ```compile_fail
//! use portable_build::{Unit, list_type, portable_name, typed_list, typed_program, I32};
//! let mut escaped = None;
//! let _ = typed_program(portable_name!("loop_escape"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![], Unit::TYPE, |body, _| {
//!         let item = body.i32(1);
//!         let items = body.list(I32::TYPE, typed_list![item]);
//!         body.for_each(portable_name!("item"), items, |body, item| {
//!             escaped = Some(item);
//!             body.unit()
//!         })
//!     }).builder
//! });
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
//!
//! Even an otherwise empty typed program is a named portable module, so a
//! plugin without `Modules` cannot admit it:
//!
//! ```compile_fail
//! use portable_build::{Requirements, SupportsAll, TypedProgram, portable_name, typed_program};
//! struct EmptyDialect;
//! fn require_support<R: Requirements>(program: &TypedProgram<R>)
//! where
//!     EmptyDialect: SupportsAll<R>,
//! {
//!     let _ = program;
//! }
//! let program = typed_program(portable_name!("module_required"), |builder| builder);
//! require_support(&program);
//! ```
//!
//! A feature mapping cannot be registered twice because the first consuming
//! call replaces its type-level `Missing` slot:
//!
//! ```compile_fail
//! use portable_build::{CapabilityMapping, I32Values, language_plugin};
//! struct Dialect;
//! #[derive(Clone, Copy)]
//! struct Mapping;
//! impl CapabilityMapping<Dialect> for Mapping {
//!     type Capability = I32Values;
//!     type Context = ();
//!     type Input = ();
//!     type Output = ();
//!     type Error = ();
//!     fn lower(&self, _: &mut (), _: ()) -> Result<(), ()> { Ok(()) }
//! }
//! let _ = language_plugin(Dialect).support(Mapping).support(Mapping);
//! ```
//!
//! A plugin which implements functions but omits call propagation cannot
//! admit a typed program containing a call:
//!
//! ```compile_fail
//! use std::marker::PhantomData;
//! use portable_build::{
//!     Capability, CapabilityMapping, Functions, I32, I32Values, Modules,
//!     Requirements, SupportsAll, TypedProgram, language_plugin, parameter,
//!     portable_name, typed_list, typed_program,
//! };
//! struct Dialect;
//! #[derive(Clone, Copy)]
//! struct Mapping<C: Capability>(PhantomData<C>);
//! impl<C: Capability + 'static> CapabilityMapping<Dialect> for Mapping<C> {
//!     type Capability = C;
//!     type Context = ();
//!     type Input = ();
//!     type Output = ();
//!     type Error = ();
//!     fn lower(&self, _: &mut (), _: ()) -> Result<(), ()> { Ok(()) }
//! }
//! const fn mapping<C: Capability>() -> Mapping<C> { Mapping(PhantomData) }
//! let program = typed_program(portable_name!("missing_propagation"), |builder| {
//!     let added = builder.function(
//!         portable_name!("identity"),
//!         typed_list![parameter(portable_name!("value"), I32::TYPE)],
//!         I32::TYPE,
//!         |body, values| body.read(values.head),
//!     );
//!     let identity = added.handle;
//!     added.builder.function(
//!         portable_name!("caller"), typed_list![], I32::TYPE,
//!         |body, _| {
//!             let value = body.i32(7);
//!             body.call(identity, typed_list![value])
//!         },
//!     ).builder
//! });
//! let plugin = language_plugin(Dialect)
//!     .support(mapping::<Modules>())
//!     .support(mapping::<Functions>())
//!     .support(mapping::<I32Values>())
//!     .build();
//! fn admit<P, R: Requirements>(plugin: &P, program: &TypedProgram<R>)
//! where P: SupportsAll<R> { let _ = (plugin, program); }
//! admit(&plugin, &program);
//! ```
//!
//! A mapping for one dialect cannot be registered in another dialect:
//!
//! ```compile_fail
//! use portable_build::{CapabilityMapping, I32Values, language_plugin};
//! struct First;
//! struct Second;
//! struct Mapping;
//! impl CapabilityMapping<First> for Mapping {
//!     type Capability = I32Values;
//!     type Context = ();
//!     type Input = ();
//!     type Output = ();
//!     type Error = ();
//!     fn lower(&self, _: &mut (), _: ()) -> Result<(), ()> { Ok(()) }
//! }
//! let _ = language_plugin(Second).support(Mapping);
//! ```
//!
//! Homogeneous lists reject a differently typed element:
//!
//! ```compile_fail
//! use portable_build::{I32, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("mixed_list"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![],
//!         portable_build::list_type(I32::TYPE), |body, _| {
//!             let integer = body.i32(1);
//!             let text = body.text("two");
//!             body.list(I32::TYPE, typed_list![integer, text])
//!         }).builder
//! });
//! ```
//!
//! List operations require an element of the declared element type:
//!
//! ```compile_fail
//! use portable_build::{I32, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("list_element"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![],
//!         portable_build::list_type(I32::TYPE), |body, _| {
//!             let integer = body.i32(1);
//!             let list = body.list(I32::TYPE, typed_list![integer]);
//!             let text = body.text("two");
//!             body.list_append(list, text)
//!         }).builder
//! });
//! ```
//!
//! Option fallbacks must have the option's inner type:
//!
//! ```compile_fail
//! use portable_build::{I32, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("option_fallback"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![], I32::TYPE, |body, _| {
//!         let value = body.none(I32::TYPE);
//!         let fallback = body.text("zero");
//!         body.option_unwrap_or(value, fallback)
//!     }).builder
//! });
//! ```
//!
//! Result branches retain both exact branch types:
//!
//! ```compile_fail
//! use portable_build::{I32, Text, portable_name, result_type, typed_list, typed_program};
//! let _ = typed_program(portable_name!("result_branch"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![],
//!         result_type(I32::TYPE, Text::TYPE), |body, _| {
//!             let error = body.bool(false);
//!             body.err(error, I32::TYPE)
//!         }).builder
//! });
//! ```
//!
//! Replace-many accepts one or more typed replacement pairs, not raw strings:
//!
//! ```compile_fail
//! use portable_build::{Text, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("replacement_shape"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![], Text::TYPE, |body, _| {
//!         let source = body.text("abc");
//!         let raw = body.text("a");
//!         body.string_replace_many(source, typed_list![raw])
//!     }).builder
//! });
//! ```
//!
//! String and bytes operations cannot be confused:
//!
//! ```compile_fail
//! use portable_build::{Text, portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("string_bytes"), |builder| {
//!     builder.function(portable_name!("bad"), typed_list![], Text::TYPE, |body, _| {
//!         let left = body.text("a");
//!         let right = body.bytes(vec![98]);
//!         body.string_concat(left, right)
//!     }).builder
//! });
//! ```
//!
//! Payload-free enums require at least one variant:
//!
//! ```compile_fail
//! use portable_build::{portable_name, typed_list, typed_program};
//! let _ = typed_program(portable_name!("empty_enum"), |builder| {
//!     builder.enumeration(
//!         portable_name!("Empty"),
//!         typed_list![],
//!         |builder, _| builder,
//!     )
//! });
//! ```
//!
//! Enum values cannot be constructed with payload fields because the typed
//! constructor accepts only a declaration and one of its branded variants:
//!
//! ```compile_fail
//! use portable_build::{portable_name, typed_list, typed_program, variant};
//! let _ = typed_program(portable_name!("enum_payload"), |builder| {
//!     builder.enumeration(
//!         portable_name!("Choice"),
//!         typed_list![variant(portable_name!("ONLY"))],
//!         |builder, choice| {
//!             builder.function(
//!                 portable_name!("bad"), typed_list![], choice.ty(), |body, _| {
//!                     let payload = body.i32(1);
//!                     body.enum_variant(&choice, choice.variants().head, payload)
//!                 },
//!             ).builder
//!         },
//!     )
//! });
//! ```
//!
//! A variant from one enum cannot construct a value of another enum:
//!
//! ```compile_fail
//! use portable_build::{portable_name, typed_list, typed_program, variant};
//! let _ = typed_program(portable_name!("enum_brands"), |builder| {
//!     builder.enumeration(
//!         portable_name!("First"),
//!         typed_list![variant(portable_name!("ONE"))],
//!         |builder, first| builder.enumeration(
//!             portable_name!("Second"),
//!             typed_list![variant(portable_name!("TWO"))],
//!             |builder, second| builder.function(
//!                 portable_name!("bad"), typed_list![], second.ty(), |body, _| {
//!                     body.enum_variant(&second, first.variants().head)
//!                 },
//!             ).builder,
//!         ),
//!     )
//! });
//! ```
//!
//! Exhaustive enum branches must contain one arm for every variant, in the
//! declaration order. Omitting an arm changes the arm-list type:
//!
//! ```compile_fail
//! use portable_build::{I32, enum_arm, parameter, portable_name, typed_list, typed_program, variant};
//! let _ = typed_program(portable_name!("enum_match_missing"), |builder| {
//!     builder.enumeration(
//!         portable_name!("Choice"),
//!         typed_list![variant(portable_name!("FIRST")), variant(portable_name!("SECOND"))],
//!         |builder, choice| builder.function(
//!             portable_name!("rank"),
//!             typed_list![parameter(portable_name!("value"), choice.ty())],
//!             I32::TYPE,
//!             |body, values| {
//!                 let value = body.read(values.head);
//!                 let first = body.i32(1);
//!                 body.enum_match(
//!                     &choice,
//!                     value,
//!                     typed_list![enum_arm(choice.variants().head, first)],
//!                 )
//!             },
//!         ).builder,
//!     )
//! });
//! ```
//!
//! Duplicating or reordering a variant also changes the positional handle
//! types and is rejected before the portable IR can be built:
//!
//! ```compile_fail
//! use portable_build::{I32, enum_arm, parameter, portable_name, typed_list, typed_program, variant};
//! let _ = typed_program(portable_name!("enum_match_duplicate"), |builder| {
//!     builder.enumeration(
//!         portable_name!("Choice"),
//!         typed_list![variant(portable_name!("FIRST")), variant(portable_name!("SECOND"))],
//!         |builder, choice| builder.function(
//!             portable_name!("rank"),
//!             typed_list![parameter(portable_name!("value"), choice.ty())],
//!             I32::TYPE,
//!             |body, values| {
//!                 let value = body.read(values.head);
//!                 let first = body.i32(1);
//!                 let duplicate = body.i32(2);
//!                 body.enum_match(
//!                     &choice,
//!                     value,
//!                     typed_list![
//!                         enum_arm(choice.variants().head, first),
//!                         enum_arm(choice.variants().head, duplicate),
//!                     ],
//!                 )
//!             },
//!         ).builder,
//!     )
//! });
//! ```

use std::{cell::Cell, marker::PhantomData};

use portable_check::v0::CheckedProgram;

use crate::capabilities::*;
use crate::{
    AliasId, BodyBuilder, ConstantId, EnumId, EnumVariantId, FunctionId, ModuleBuilder, Operation,
    Parameter, RecordFieldId, RecordId, Type, Value, Visibility,
};

mod sealed {
    pub trait Parameters {}
    pub trait Arguments {}
    pub trait Fields {}
    pub trait Variants {}
    pub trait NonEmptyVariants {}
    pub trait EnumArms {}
    pub trait HomogeneousArguments {}
    pub trait ReplacementArguments {}
    pub trait Equatable {}
    pub trait Ordered {}
    pub trait Integer {}
}

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
/// Unit value marker.
pub enum Unit {}
/// 32-bit signed integer marker.
pub enum I32 {}
/// 64-bit signed integer marker.
pub enum I64 {}
/// IEEE-754 binary64 marker.
pub enum F64 {}
/// Unicode string marker.
pub enum Text {}
/// Unicode scalar marker.
pub enum Char {}
/// Immutable byte-string marker.
pub enum Bytes {}
/// Homogeneous immutable-list marker.
pub struct List<T>(PhantomData<fn() -> T>);
/// Optional-value marker.
pub struct Optional<T>(PhantomData<fn() -> T>);
/// Success/error tagged-value marker.
pub struct ResultValue<Ok, Error>(PhantomData<fn() -> (Ok, Error)>);

type InvariantAliasBrand<'module, 'alias, T, R> =
    (Cell<&'module ()>, Cell<&'alias ()>, fn(T, R) -> (T, R));

/// A transparent alias value branded with its exact declaration.
pub struct AliasValue<'module, 'alias, T>(PhantomData<InvariantAliasBrand<'module, 'alias, T, ()>>);

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
primitive_type!(Unit, unit, UnitValues);
primitive_type!(I32, i32, I32Values);
primitive_type!(I64, i64, I64Values);
primitive_type!(F64, f64, F64Values);
primitive_type!(Text, string, TextValues);
primitive_type!(Char, char, CharValues);
primitive_type!(Bytes, bytes, BytesValues);

pub fn list_type<T, R: Requirements>(
    element: TypedType<T, R>,
) -> TypedType<List<T>, All<Requires<ListValues>, R>> {
    TypedType {
        ir: Type::list(element.ir),
        marker: PhantomData,
    }
}

pub fn option_type<T, R: Requirements>(
    inner: TypedType<T, R>,
) -> TypedType<Optional<T>, All<Requires<OptionValues>, R>> {
    TypedType {
        ir: Type::option(inner.ir),
        marker: PhantomData,
    }
}

pub fn result_type<Ok, OkR, Error, ErrorR>(
    ok: TypedType<Ok, OkR>,
    error: TypedType<Error, ErrorR>,
) -> TypedType<ResultValue<Ok, Error>, ResultTypeRequirements<OkR, ErrorR>>
where
    OkR: Requirements,
    ErrorR: Requirements,
{
    TypedType {
        ir: Type::result(ok.ir, error.ir),
        marker: PhantomData,
    }
}

/// Values admitted by equality operations, with their owning capability.
pub trait TypedEquatable: sealed::Equatable {
    type EqualityCapability: Capability;
}
/// Values admitted by ordered comparisons.
pub trait TypedOrdered: sealed::Ordered {}
/// Values admitted by integer operations.
pub trait TypedInteger: sealed::Integer {}

macro_rules! equatable {
    ($($type:ty),+ $(,)?) => {$(
        impl sealed::Equatable for $type {}
        impl TypedEquatable for $type {
            type EqualityCapability = Equality;
        }
    )+};
}

equatable!(Bool, I32, I64, F64, Text, Char, Bytes);
impl<T: TypedEquatable> sealed::Equatable for List<T> {}
impl<T: TypedEquatable> TypedEquatable for List<T> {
    type EqualityCapability = Equality;
}
impl<T: TypedEquatable> sealed::Equatable for Optional<T> {}
impl<T: TypedEquatable> TypedEquatable for Optional<T> {
    type EqualityCapability = Equality;
}
impl<Ok: TypedEquatable, Error: TypedEquatable> sealed::Equatable for ResultValue<Ok, Error> {}
impl<Ok: TypedEquatable, Error: TypedEquatable> TypedEquatable for ResultValue<Ok, Error> {
    type EqualityCapability = Equality;
}
impl sealed::Ordered for I32 {}
impl TypedOrdered for I32 {}
impl sealed::Ordered for I64 {}
impl TypedOrdered for I64 {}
impl sealed::Ordered for F64 {}
impl TypedOrdered for F64 {}
impl sealed::Ordered for Text {}
impl TypedOrdered for Text {}
impl sealed::Ordered for Char {}
impl TypedOrdered for Char {}
impl sealed::Integer for I32 {}
impl TypedInteger for I32 {}
impl sealed::Integer for I64 {}
impl TypedInteger for I64 {}

/// A record value branded with its module and exact declaration.
pub struct RecordValue<'module, 'record>(PhantomData<(Cell<&'module ()>, Cell<&'record ()>)>);

impl sealed::Equatable for RecordValue<'_, '_> {}
impl TypedEquatable for RecordValue<'_, '_> {
    type EqualityCapability = Equality;
}

/// A payload-free enum value branded with its module and exact declaration.
pub struct EnumValue<'module, 'enumeration>(
    PhantomData<(Cell<&'module ()>, Cell<&'enumeration ()>)>,
);

impl sealed::Equatable for EnumValue<'_, '_> {}
impl TypedEquatable for EnumValue<'_, '_> {
    type EqualityCapability = Enums;
}

/// A typed expression owned by one callable body with inferred requirements.
type InvariantExpressionBrand<'module, 'body, T, R> = fn(&'module (), &'body (), T, R) -> (T, R);

pub struct TypedExpr<'module, 'body, T, R: Requirements> {
    node: TypedNode,
    marker: PhantomData<InvariantExpressionBrand<'module, 'body, T, R>>,
}

enum TypedNode {
    Literal(Value),
    Local(String),
    Constant(ConstantId),
    LetValue {
        name: String,
        value: Box<TypedNode>,
        result: Box<TypedNode>,
    },
    IfValue {
        condition: Box<TypedNode>,
        then_value: Box<TypedNode>,
        else_value: Box<TypedNode>,
    },
    ForEach {
        binding: String,
        iterable: Box<TypedNode>,
        body: Box<TypedNode>,
    },
    Record {
        record: RecordId,
        fields: Vec<(RecordFieldId, TypedNode)>,
    },
    Enum {
        enumeration: EnumId,
        variant: EnumVariantId,
    },
    EnumMatch {
        enumeration: EnumId,
        value: Box<TypedNode>,
        arms: Vec<(EnumVariantId, TypedNode)>,
    },
    Field {
        base: Box<TypedNode>,
        field: RecordFieldId,
    },
    Call {
        function: FunctionId,
        arguments: Vec<TypedNode>,
    },
    List {
        element: Type,
        values: Vec<TypedNode>,
    },
    Some(Box<TypedNode>),
    None(Type),
    Ok {
        value: Box<TypedNode>,
        error: Type,
    },
    Err {
        value: Box<TypedNode>,
        ok: Type,
    },
    Intrinsic {
        operation: Operation,
        arguments: Vec<TypedNode>,
    },
}

/// A typed constant expression owned by one constant declaration.
type InvariantConstantBrand<'module, 'constant, T, R> =
    fn(&'module (), &'constant (), T, R) -> (T, R);

pub struct TypedConstantExpr<'module, 'constant, T, R: Requirements> {
    node: TypedConstantNode,
    marker: PhantomData<InvariantConstantBrand<'module, 'constant, T, R>>,
}

enum TypedConstantNode {
    Literal(Value),
    Reference(ConstantId),
}

/// The constant-expression factory for one declaration.
pub struct TypedConstantBody<'module, 'constant> {
    marker: PhantomData<(Cell<&'module ()>, Cell<&'constant ()>)>,
}

impl<'module, 'constant> TypedConstantBody<'module, 'constant> {
    fn expression<T, R: Requirements>(
        &self,
        node: TypedConstantNode,
    ) -> TypedConstantExpr<'module, 'constant, T, R> {
        TypedConstantExpr {
            node,
            marker: PhantomData,
        }
    }

    pub fn unit(&mut self) -> TypedConstantExpr<'module, 'constant, Unit, Requires<UnitValues>> {
        self.expression(TypedConstantNode::Literal(Value::unit()))
    }

    pub fn bool(
        &mut self,
        value: bool,
    ) -> TypedConstantExpr<'module, 'constant, Bool, Requires<BoolValues>> {
        self.expression(TypedConstantNode::Literal(Value::bool(value)))
    }

    pub fn i32(
        &mut self,
        value: i32,
    ) -> TypedConstantExpr<'module, 'constant, I32, Requires<I32Values>> {
        self.expression(TypedConstantNode::Literal(Value::i32(value)))
    }

    pub fn i64(
        &mut self,
        value: i64,
    ) -> TypedConstantExpr<'module, 'constant, I64, Requires<I64Values>> {
        self.expression(TypedConstantNode::Literal(Value::i64(value)))
    }

    pub fn f64(
        &mut self,
        value: f64,
    ) -> TypedConstantExpr<'module, 'constant, F64, Requires<F64Values>> {
        self.expression(TypedConstantNode::Literal(Value::f64(value)))
    }

    pub fn text(
        &mut self,
        value: impl Into<String>,
    ) -> TypedConstantExpr<'module, 'constant, Text, Requires<TextValues>> {
        self.expression(TypedConstantNode::Literal(Value::string(value)))
    }

    pub fn char(
        &mut self,
        value: char,
    ) -> TypedConstantExpr<'module, 'constant, Char, Requires<CharValues>> {
        self.expression(TypedConstantNode::Literal(Value::char(value)))
    }

    pub fn bytes(
        &mut self,
        value: impl Into<Vec<u8>>,
    ) -> TypedConstantExpr<'module, 'constant, Bytes, Requires<BytesValues>> {
        self.expression(TypedConstantNode::Literal(Value::bytes(value)))
    }

    pub fn read<T>(
        &mut self,
        constant: TypedConstant<'module, T>,
    ) -> TypedConstantExpr<'module, 'constant, T, Requires<Constants>> {
        self.expression(TypedConstantNode::Reference(constant.raw))
    }
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

type InvariantBindingBrand<'module, 'body, 'binding, T> = (
    Cell<&'module ()>,
    Cell<&'body ()>,
    Cell<&'binding ()>,
    fn(T) -> T,
);

/// An immutable local whose brand exists only inside its binding continuation.
pub struct TypedBinding<'module, 'body, 'binding, T> {
    name: String,
    marker: PhantomData<InvariantBindingBrand<'module, 'body, 'binding, T>>,
}

type InvariantLoopItemBrand<'module, 'body, 'iteration, T> = (
    Cell<&'module ()>,
    Cell<&'body ()>,
    Cell<&'iteration ()>,
    fn(T) -> T,
);

/// One immutable iteration value scoped to a single `for_each` body.
pub struct TypedLoopItem<'module, 'body, 'iteration, T> {
    name: String,
    marker: PhantomData<InvariantLoopItemBrand<'module, 'body, 'iteration, T>>,
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

/// A recursively typed homogeneous expression list.
pub trait HomogeneousArgumentList<'module, 'body, T>: sealed::HomogeneousArguments {
    type Requirements: Requirements;

    #[doc(hidden)]
    fn into_homogeneous_nodes(self) -> ArgumentNodes;
}

impl sealed::HomogeneousArguments for Nil {}
impl<'module, 'body, T> HomogeneousArgumentList<'module, 'body, T> for Nil {
    type Requirements = NoneRequired;

    fn into_homogeneous_nodes(self) -> ArgumentNodes {
        ArgumentNodes(Vec::new())
    }
}

impl<'module, 'body, T, R, Tail> sealed::HomogeneousArguments
    for Cons<TypedExpr<'module, 'body, T, R>, Tail>
where
    R: Requirements,
    Tail: HomogeneousArgumentList<'module, 'body, T>,
{
}

impl<'module, 'body, T, R, Tail> HomogeneousArgumentList<'module, 'body, T>
    for Cons<TypedExpr<'module, 'body, T, R>, Tail>
where
    R: Requirements,
    Tail: HomogeneousArgumentList<'module, 'body, T>,
{
    type Requirements = All<R, Tail::Requirements>;

    fn into_homogeneous_nodes(self) -> ArgumentNodes {
        let mut nodes = vec![self.head.node];
        nodes.extend(self.tail.into_homogeneous_nodes().0);
        ArgumentNodes(nodes)
    }
}

/// One strongly typed needle/replacement pair for `string_replace_many`.
pub struct TypedReplacement<'module, 'body, NeedleR, ReplacementR>
where
    NeedleR: Requirements,
    ReplacementR: Requirements,
{
    needle: TypedExpr<'module, 'body, Text, NeedleR>,
    replacement: TypedExpr<'module, 'body, Text, ReplacementR>,
}

pub fn replacement<'module, 'body, NeedleR, ReplacementR>(
    needle: TypedExpr<'module, 'body, Text, NeedleR>,
    replacement: TypedExpr<'module, 'body, Text, ReplacementR>,
) -> TypedReplacement<'module, 'body, NeedleR, ReplacementR>
where
    NeedleR: Requirements,
    ReplacementR: Requirements,
{
    TypedReplacement {
        needle,
        replacement,
    }
}

/// Recursive list of string replacement pairs.
pub trait ReplacementList<'module, 'body>: sealed::ReplacementArguments {
    type Requirements: Requirements;

    #[doc(hidden)]
    fn into_replacement_nodes(self) -> ArgumentNodes;
}

/// Computes the inferred requirement tree for a non-empty replace-many list.
#[doc(hidden)]
pub trait ReplaceManyRequirements<'module, 'body, SourceR: Requirements>:
    ReplacementList<'module, 'body>
{
    type Combined: Requirements;
}

impl<'module, 'body, SourceR, NeedleR, ReplacementR, Tail>
    ReplaceManyRequirements<'module, 'body, SourceR>
    for Cons<TypedReplacement<'module, 'body, NeedleR, ReplacementR>, Tail>
where
    SourceR: Requirements,
    NeedleR: Requirements,
    ReplacementR: Requirements,
    Cons<TypedReplacement<'module, 'body, NeedleR, ReplacementR>, Tail>:
        ReplacementList<'module, 'body>,
{
    type Combined = All<
        Requires<StringTransformation>,
        All<SourceR, <Self as ReplacementList<'module, 'body>>::Requirements>,
    >;
}

type ReplaceManyExpr<'module, 'body, SourceR, NeedleR, ReplacementR, Tail> = TypedExpr<
    'module,
    'body,
    Text,
    <Cons<TypedReplacement<'module, 'body, NeedleR, ReplacementR>, Tail> as ReplaceManyRequirements<
        'module,
        'body,
        SourceR,
    >>::Combined,
>;

impl sealed::ReplacementArguments for Nil {}
impl<'module, 'body> ReplacementList<'module, 'body> for Nil {
    type Requirements = NoneRequired;

    fn into_replacement_nodes(self) -> ArgumentNodes {
        ArgumentNodes(Vec::new())
    }
}

impl<'module, 'body, NeedleR, ReplacementR, Tail> sealed::ReplacementArguments
    for Cons<TypedReplacement<'module, 'body, NeedleR, ReplacementR>, Tail>
where
    NeedleR: Requirements,
    ReplacementR: Requirements,
    Tail: ReplacementList<'module, 'body>,
{
}

impl<'module, 'body, NeedleR, ReplacementR, Tail> ReplacementList<'module, 'body>
    for Cons<TypedReplacement<'module, 'body, NeedleR, ReplacementR>, Tail>
where
    NeedleR: Requirements,
    ReplacementR: Requirements,
    Tail: ReplacementList<'module, 'body>,
{
    type Requirements = All<All<NeedleR, ReplacementR>, Tail::Requirements>;

    fn into_replacement_nodes(self) -> ArgumentNodes {
        let mut nodes = vec![self.head.needle.node, self.head.replacement.node];
        nodes.extend(self.tail.into_replacement_nodes().0);
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

/// A payload-free enum variant specification.
pub struct TypedVariantSpec {
    name: PortableName,
}

pub const fn variant(name: PortableName) -> TypedVariantSpec {
    TypedVariantSpec { name }
}

type InvariantEnumBrand<'module, 'enumeration, Position = ()> = (
    Cell<&'module ()>,
    Cell<&'enumeration ()>,
    fn(Position) -> Position,
);

/// A variant handle tied to one exact enum declaration and list position.
pub struct TypedVariant<'module, 'enumeration, Position> {
    raw: EnumVariantId,
    marker: PhantomData<InvariantEnumBrand<'module, 'enumeration, Position>>,
}

impl<Position> Copy for TypedVariant<'_, '_, Position> {}
impl<Position> Clone for TypedVariant<'_, '_, Position> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A recursive list of payload-free enum variants.
pub trait VariantList: sealed::Variants {
    type Handles<'module, 'enumeration>;

    #[doc(hidden)]
    fn append_raw(self, output: &mut Vec<PortableName>);
    #[doc(hidden)]
    fn make_handles<'module, 'enumeration>(
        variants: &mut std::vec::IntoIter<EnumVariantId>,
    ) -> Self::Handles<'module, 'enumeration>;
}

impl sealed::Variants for Nil {}
impl VariantList for Nil {
    type Handles<'module, 'enumeration> = Nil;

    fn append_raw(self, _output: &mut Vec<PortableName>) {}

    fn make_handles<'module, 'enumeration>(
        _variants: &mut std::vec::IntoIter<EnumVariantId>,
    ) -> Self::Handles<'module, 'enumeration> {
        Nil
    }
}

impl<Tail> sealed::Variants for Cons<TypedVariantSpec, Tail> where Tail: VariantList {}
impl<Tail> sealed::NonEmptyVariants for Cons<TypedVariantSpec, Tail> where Tail: VariantList {}

impl<Tail> VariantList for Cons<TypedVariantSpec, Tail>
where
    Tail: VariantList,
{
    type Handles<'module, 'enumeration> =
        Cons<TypedVariant<'module, 'enumeration, Tail>, Tail::Handles<'module, 'enumeration>>;

    fn append_raw(self, output: &mut Vec<PortableName>) {
        output.push(self.head.name);
        self.tail.append_raw(output);
    }

    fn make_handles<'module, 'enumeration>(
        variants: &mut std::vec::IntoIter<EnumVariantId>,
    ) -> Self::Handles<'module, 'enumeration> {
        Cons::new(
            TypedVariant {
                raw: variants.next().expect("typed enum variant"),
                marker: PhantomData,
            },
            Tail::make_handles(variants),
        )
    }
}

/// One value-producing arm for one exact payload-free enum variant.
pub struct TypedEnumArm<'module, 'body, 'enumeration, Position, Output, R: Requirements> {
    variant: EnumVariantId,
    value: TypedNode,
    marker: PhantomData<(
        InvariantEnumBrand<'module, 'enumeration, Position>,
        InvariantExpressionBrand<'module, 'body, Output, R>,
    )>,
}

/// Binds an enum variant to its typed result expression.
pub fn enum_arm<'module, 'body, 'enumeration, Position, Output, R: Requirements>(
    variant: TypedVariant<'module, 'enumeration, Position>,
    value: TypedExpr<'module, 'body, Output, R>,
) -> TypedEnumArm<'module, 'body, 'enumeration, Position, Output, R> {
    TypedEnumArm {
        variant: variant.raw,
        value: value.node,
        marker: PhantomData,
    }
}

/// An exhaustive, ordered arm list whose type preserves every variant identity.
pub trait EnumArmList<'module, 'body, 'enumeration, Output>: sealed::EnumArms {
    type VariantHandles;
    type Requirements: Requirements;

    #[doc(hidden)]
    fn into_nodes(self) -> EnumArmNodes;
}

/// Opaque enum-branch lowering payload used only by the checked-IR bridge.
#[doc(hidden)]
pub struct EnumArmNodes(Vec<(EnumVariantId, TypedNode)>);

impl sealed::EnumArms for Nil {}
impl<'module, 'body, 'enumeration, Output> EnumArmList<'module, 'body, 'enumeration, Output>
    for Nil
{
    type VariantHandles = Nil;
    type Requirements = NoneRequired;

    fn into_nodes(self) -> EnumArmNodes {
        EnumArmNodes(vec![])
    }
}

impl<'module, 'body, 'enumeration, Position, Output, R, Tail> sealed::EnumArms
    for Cons<TypedEnumArm<'module, 'body, 'enumeration, Position, Output, R>, Tail>
where
    R: Requirements,
    Tail: EnumArmList<'module, 'body, 'enumeration, Output>,
{
}

impl<'module, 'body, 'enumeration, Position, Output, R, Tail>
    EnumArmList<'module, 'body, 'enumeration, Output>
    for Cons<TypedEnumArm<'module, 'body, 'enumeration, Position, Output, R>, Tail>
where
    R: Requirements,
    Tail: EnumArmList<'module, 'body, 'enumeration, Output>,
{
    type VariantHandles = Cons<TypedVariant<'module, 'enumeration, Position>, Tail::VariantHandles>;
    type Requirements = All<R, Tail::Requirements>;

    fn into_nodes(self) -> EnumArmNodes {
        let mut nodes = vec![(self.head.variant, self.head.value)];
        nodes.extend(self.tail.into_nodes().0);
        EnumArmNodes(nodes)
    }
}

/// A payload-free enum and its exact branded variant-handle list.
pub struct TypedEnum<'module, 'enumeration, Handles> {
    raw: EnumId,
    variants: Handles,
    marker: PhantomData<InvariantEnumBrand<'module, 'enumeration>>,
}

impl<'module, 'enumeration, Handles> TypedEnum<'module, 'enumeration, Handles> {
    pub fn ty(&self) -> TypedType<EnumValue<'module, 'enumeration>, Requires<Enums>> {
        TypedType {
            ir: Type::named(self.raw),
            marker: PhantomData,
        }
    }

    pub const fn variants(&self) -> &Handles {
        &self.variants
    }
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

/// A typed immutable constant handle tied to its module.
pub struct TypedConstant<'module, T> {
    raw: ConstantId,
    marker: PhantomData<fn(&'module ()) -> T>,
}

impl<T> Copy for TypedConstant<'_, T> {}
impl<T> Clone for TypedConstant<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A transparent type alias and its exact declaration brand.
pub struct TypedAlias<'module, 'alias, T, R: Requirements> {
    raw: AliasId,
    target: Type,
    marker: PhantomData<InvariantAliasBrand<'module, 'alias, T, R>>,
}

impl<'module, 'alias, T, R: Requirements> TypedAlias<'module, 'alias, T, R> {
    pub fn ty(&self) -> TypedType<AliasValue<'module, 'alias, T>, All<Requires<TypeAliases>, R>> {
        TypedType {
            ir: Type::named(self.raw),
            marker: PhantomData,
        }
    }
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
type EnumRequirement<Existing> = All<Existing, Requires<Enums>>;
type ModuleRequirement<R> = All<Requires<Modules>, R>;
type ConstantRequirement<Existing, TypeR, ValueR> =
    All<Existing, All<Requires<Constants>, All<TypeR, ValueR>>>;
type AliasRequirement<Existing, TargetR> = All<Existing, All<Requires<TypeAliases>, TargetR>>;
type AliasConversionRequirements<AliasR, ValueR> = All<Requires<TypeAliases>, All<AliasR, ValueR>>;
type AliasWrappedExpr<'module, 'body, 'alias, T, AliasR, ValueR> = TypedExpr<
    'module,
    'body,
    AliasValue<'module, 'alias, T>,
    AliasConversionRequirements<AliasR, ValueR>,
>;
type AliasUnwrappedExpr<'module, 'body, T, AliasR, ValueR> =
    TypedExpr<'module, 'body, T, AliasConversionRequirements<AliasR, ValueR>>;
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
) -> TypedProgram<ModuleRequirement<R>> {
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
    fn with_used(used: impl IntoIterator<Item = String>) -> Self {
        Self {
            used: used.into_iter().collect(),
        }
    }

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
    pub fn constant<T, TypeR, ValueR>(
        mut self,
        name: PortableName,
        ty: TypedType<T, TypeR>,
        build: impl for<'constant> FnOnce(
            &mut TypedConstantBody<'module, 'constant>,
        ) -> TypedConstantExpr<'module, 'constant, T, ValueR>,
    ) -> Added<
        ProgramBuilder<'module, ConstantRequirement<Existing, TypeR, ValueR>>,
        TypedConstant<'module, T>,
    >
    where
        TypeR: Requirements,
        ValueR: Requirements,
    {
        let name = self.names.allocate(name);
        let raw = self
            .dynamic
            .constant(name, Visibility::Public, vec![], ty.ir, |body| {
                let mut typed = TypedConstantBody {
                    marker: PhantomData,
                };
                let value = build(&mut typed);
                lower_constant_expression(body, value.node)
            });
        Added {
            builder: ProgramBuilder {
                dynamic: self.dynamic,
                names: self.names,
                marker: PhantomData,
            },
            handle: TypedConstant {
                raw,
                marker: PhantomData,
            },
        }
    }

    pub fn alias<T, TargetR, OutputRequirements>(
        mut self,
        name: PortableName,
        target: TypedType<T, TargetR>,
        then: impl for<'alias> FnOnce(
            ProgramBuilder<'module, AliasRequirement<Existing, TargetR>>,
            TypedAlias<'module, 'alias, T, TargetR>,
        ) -> ProgramBuilder<'module, OutputRequirements>,
    ) -> ProgramBuilder<'module, OutputRequirements>
    where
        TargetR: Requirements,
        OutputRequirements: Requirements,
    {
        let name = self.names.allocate(name);
        let target_ir = target.ir;
        let raw = self
            .dynamic
            .alias(name, Visibility::Public, vec![], target_ir.clone());
        then(
            ProgramBuilder {
                dynamic: self.dynamic,
                names: self.names,
                marker: PhantomData,
            },
            TypedAlias {
                raw,
                target: target_ir,
                marker: PhantomData,
            },
        )
    }

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
                    let reserved_names = local_names.clone();
                    let mut names = local_names.into_iter();
                    let locals = P::make_locals(&mut names);
                    let mut typed = TypedBody {
                        names: NameAllocator::with_used(reserved_names),
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

    pub fn enumeration<Variants, OutputRequirements>(
        mut self,
        name: PortableName,
        variants: Variants,
        then: impl for<'enumeration> FnOnce(
            ProgramBuilder<'module, EnumRequirement<Existing>>,
            TypedEnum<'module, 'enumeration, Variants::Handles<'module, 'enumeration>>,
        ) -> ProgramBuilder<'module, OutputRequirements>,
    ) -> ProgramBuilder<'module, OutputRequirements>
    where
        Variants: VariantList + sealed::NonEmptyVariants,
        OutputRequirements: Requirements,
    {
        let name = self.names.allocate(name);
        let mut raw_variants = Vec::new();
        variants.append_raw(&mut raw_variants);
        let mut variant_names = NameAllocator::default();
        let raw_variants = raw_variants
            .into_iter()
            .map(|name| variant_names.allocate(name))
            .collect::<Vec<_>>();
        let (raw, variant_ids) =
            self.dynamic
                .enumeration(name, Visibility::Public, vec![], |enumeration| {
                    raw_variants
                        .into_iter()
                        .map(|name| enumeration.variant(name, vec![], |_| {}).0)
                        .collect::<Vec<_>>()
                });
        let handles = Variants::make_handles(&mut variant_ids.into_iter());
        then(
            ProgramBuilder {
                dynamic: self.dynamic,
                names: self.names,
                marker: PhantomData,
            },
            TypedEnum {
                raw,
                variants: handles,
                marker: PhantomData,
            },
        )
    }
}

/// The only expression factory for one branded function body.
pub struct TypedBody<'module, 'body> {
    names: NameAllocator,
    marker: PhantomData<(Cell<&'module ()>, Cell<&'body ()>)>,
}

type With<F, R> = All<Requires<F>, R>;
type WithTwo<F, Left, Right> = All<Requires<F>, All<Left, Right>>;
type WithThree<F, First, Second, Third> = All<Requires<F>, All<First, All<Second, Third>>>;
type ResultTypeRequirements<OkR, ErrorR> = All<Requires<ResultValues>, All<OkR, ErrorR>>;
type ListConstructionRequirements<TypeR, ValuesR> = All<Requires<ListValues>, All<TypeR, ValuesR>>;
type OptionConstructionRequirements<R> = All<Requires<OptionValues>, R>;
type ResultConstructionRequirements<OkR, ErrorR> = All<Requires<ResultValues>, All<OkR, ErrorR>>;
type EnumBranchRequirements<ValueR, ArmsR> = All<Requires<Enums>, All<ValueR, ArmsR>>;
type LocalBindingRequirements<ValueR, BodyR> = All<Requires<LocalBindings>, All<ValueR, BodyR>>;
type ConditionalRequirements<ConditionR, ThenR, ElseR> =
    All<Requires<Conditionals>, All<ConditionR, All<ThenR, ElseR>>>;
type LoopRequirements<IterableR, BodyR> =
    All<Requires<Loops>, All<IterableR, All<BodyR, Requires<UnitValues>>>>;
type FunctionCallRequirements<ArgumentR> =
    All<Requires<Functions>, All<Requires<ResultPropagation>, ArgumentR>>;
type LocalBindingExpr<'module, 'body, Output, ValueR, BodyR> =
    TypedExpr<'module, 'body, Output, LocalBindingRequirements<ValueR, BodyR>>;
type ConditionalExpr<'module, 'body, Output, ConditionR, ThenR, ElseR> =
    TypedExpr<'module, 'body, Output, ConditionalRequirements<ConditionR, ThenR, ElseR>>;
type LoopExpr<'module, 'body, IterableR, BodyR> =
    TypedExpr<'module, 'body, Unit, LoopRequirements<IterableR, BodyR>>;

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
    ) -> TypedExpr<'module, 'body, T, With<Functions, R>> {
        self.expression(TypedNode::Local(local.name))
    }

    pub fn read_binding<'binding, T>(
        &mut self,
        local: TypedBinding<'module, 'body, 'binding, T>,
    ) -> TypedExpr<'module, 'body, T, Requires<LocalBindings>> {
        self.expression(TypedNode::Local(local.name))
    }

    pub fn read_loop_item<'iteration, T>(
        &mut self,
        item: TypedLoopItem<'module, 'body, 'iteration, T>,
    ) -> TypedExpr<'module, 'body, T, Requires<Loops>> {
        self.expression(TypedNode::Local(item.name))
    }

    pub fn let_value<T, ValueR, Output, BodyR>(
        &mut self,
        name: PortableName,
        value: TypedExpr<'module, 'body, T, ValueR>,
        then: impl for<'binding> FnOnce(
            &mut TypedBody<'module, 'body>,
            TypedBinding<'module, 'body, 'binding, T>,
        ) -> TypedExpr<'module, 'body, Output, BodyR>,
    ) -> LocalBindingExpr<'module, 'body, Output, ValueR, BodyR>
    where
        ValueR: Requirements,
        BodyR: Requirements,
    {
        let name = self.names.allocate(name);
        let result = then(
            self,
            TypedBinding {
                name: name.clone(),
                marker: PhantomData,
            },
        );
        self.expression(TypedNode::LetValue {
            name,
            value: Box::new(value.node),
            result: Box::new(result.node),
        })
    }

    pub fn if_else<T, ConditionR, ThenR, ElseR>(
        &mut self,
        condition: TypedExpr<'module, 'body, Bool, ConditionR>,
        then_value: TypedExpr<'module, 'body, T, ThenR>,
        else_value: TypedExpr<'module, 'body, T, ElseR>,
    ) -> ConditionalExpr<'module, 'body, T, ConditionR, ThenR, ElseR>
    where
        ConditionR: Requirements,
        ThenR: Requirements,
        ElseR: Requirements,
    {
        self.expression(TypedNode::IfValue {
            condition: Box::new(condition.node),
            then_value: Box::new(then_value.node),
            else_value: Box::new(else_value.node),
        })
    }

    pub fn for_each<T, IterableR, BodyR>(
        &mut self,
        name: PortableName,
        iterable: TypedExpr<'module, 'body, List<T>, IterableR>,
        build: impl for<'iteration> FnOnce(
            &mut TypedBody<'module, 'body>,
            TypedLoopItem<'module, 'body, 'iteration, T>,
        ) -> TypedExpr<'module, 'body, Unit, BodyR>,
    ) -> LoopExpr<'module, 'body, IterableR, BodyR>
    where
        IterableR: Requirements,
        BodyR: Requirements,
    {
        let name = self.names.allocate(name);
        let loop_body = build(
            self,
            TypedLoopItem {
                name: name.clone(),
                marker: PhantomData,
            },
        );
        self.expression(TypedNode::ForEach {
            binding: name,
            iterable: Box::new(iterable.node),
            body: Box::new(loop_body.node),
        })
    }

    pub fn constant<T>(
        &mut self,
        constant: TypedConstant<'module, T>,
    ) -> TypedExpr<'module, 'body, T, Requires<Constants>> {
        self.expression(TypedNode::Constant(constant.raw))
    }

    pub fn alias_wrap<'alias, T, AliasR, ValueR>(
        &mut self,
        alias: &TypedAlias<'module, 'alias, T, AliasR>,
        value: TypedExpr<'module, 'body, T, ValueR>,
    ) -> AliasWrappedExpr<'module, 'body, 'alias, T, AliasR, ValueR>
    where
        AliasR: Requirements,
        ValueR: Requirements,
    {
        let _ = &alias.target;
        self.expression(value.node)
    }

    pub fn alias_unwrap<'alias, T, AliasR, ValueR>(
        &mut self,
        alias: &TypedAlias<'module, 'alias, T, AliasR>,
        value: TypedExpr<'module, 'body, AliasValue<'module, 'alias, T>, ValueR>,
    ) -> AliasUnwrappedExpr<'module, 'body, T, AliasR, ValueR>
    where
        AliasR: Requirements,
        ValueR: Requirements,
    {
        let _ = &alias.target;
        self.expression(value.node)
    }

    pub fn unit(&mut self) -> TypedExpr<'module, 'body, Unit, Requires<UnitValues>> {
        self.expression(TypedNode::Literal(Value::unit()))
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

    pub fn char(&mut self, value: char) -> TypedExpr<'module, 'body, Char, Requires<CharValues>> {
        self.expression(TypedNode::Literal(Value::char(value)))
    }

    pub fn bytes(
        &mut self,
        value: impl Into<Vec<u8>>,
    ) -> TypedExpr<'module, 'body, Bytes, Requires<BytesValues>> {
        self.expression(TypedNode::Literal(Value::bytes(value)))
    }

    pub fn list<T, TypeR, Values>(
        &mut self,
        element: TypedType<T, TypeR>,
        values: Values,
    ) -> TypedExpr<'module, 'body, List<T>, ListConstructionRequirements<TypeR, Values::Requirements>>
    where
        TypeR: Requirements,
        Values: HomogeneousArgumentList<'module, 'body, T>,
    {
        self.expression(TypedNode::List {
            element: element.ir,
            values: values.into_homogeneous_nodes().0,
        })
    }

    pub fn some<T, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, Optional<T>, OptionConstructionRequirements<R>> {
        self.expression(TypedNode::Some(Box::new(value.node)))
    }

    pub fn none<T, R: Requirements>(
        &mut self,
        inner: TypedType<T, R>,
    ) -> TypedExpr<'module, 'body, Optional<T>, OptionConstructionRequirements<R>> {
        self.expression(TypedNode::None(inner.ir))
    }

    pub fn ok<Ok, OkR, Error, ErrorR>(
        &mut self,
        value: TypedExpr<'module, 'body, Ok, OkR>,
        error: TypedType<Error, ErrorR>,
    ) -> TypedExpr<
        'module,
        'body,
        ResultValue<Ok, Error>,
        ResultConstructionRequirements<OkR, ErrorR>,
    >
    where
        OkR: Requirements,
        ErrorR: Requirements,
    {
        self.expression(TypedNode::Ok {
            value: Box::new(value.node),
            error: error.ir,
        })
    }

    pub fn err<Ok, OkR, Error, ErrorR>(
        &mut self,
        value: TypedExpr<'module, 'body, Error, ErrorR>,
        ok: TypedType<Ok, OkR>,
    ) -> TypedExpr<
        'module,
        'body,
        ResultValue<Ok, Error>,
        ResultConstructionRequirements<OkR, ErrorR>,
    >
    where
        OkR: Requirements,
        ErrorR: Requirements,
    {
        self.expression(TypedNode::Err {
            value: Box::new(value.node),
            ok: ok.ir,
        })
    }

    pub fn construct<'record, Types, Handles, Arguments>(
        &mut self,
        record: &TypedRecord<'module, 'record, Types, Handles>,
        arguments: Arguments,
    ) -> TypedExpr<
        'module,
        'body,
        RecordValue<'module, 'record>,
        With<Records, Arguments::Requirements>,
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

    pub fn enum_variant<'enumeration, Handles, Position>(
        &mut self,
        enumeration: &TypedEnum<'module, 'enumeration, Handles>,
        variant: TypedVariant<'module, 'enumeration, Position>,
    ) -> TypedExpr<'module, 'body, EnumValue<'module, 'enumeration>, Requires<Enums>> {
        self.expression(TypedNode::Enum {
            enumeration: enumeration.raw,
            variant: variant.raw,
        })
    }

    pub fn enum_match<'enumeration, Handles, Output, ValueR, Arms>(
        &mut self,
        enumeration: &TypedEnum<'module, 'enumeration, Handles>,
        value: TypedExpr<'module, 'body, EnumValue<'module, 'enumeration>, ValueR>,
        arms: Arms,
    ) -> TypedExpr<'module, 'body, Output, EnumBranchRequirements<ValueR, Arms::Requirements>>
    where
        ValueR: Requirements,
        Arms: EnumArmList<'module, 'body, 'enumeration, Output, VariantHandles = Handles>,
    {
        self.expression(TypedNode::EnumMatch {
            enumeration: enumeration.raw,
            value: Box::new(value.node),
            arms: arms.into_nodes().0,
        })
    }

    pub fn field<'record, T, BaseRequirements>(
        &mut self,
        base: TypedExpr<'module, 'body, RecordValue<'module, 'record>, BaseRequirements>,
        field: TypedField<'module, 'record, T>,
    ) -> TypedExpr<'module, 'body, T, With<Records, BaseRequirements>>
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
    ) -> TypedExpr<'module, 'body, Output, FunctionCallRequirements<Arguments::Requirements>>
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
        FeatureMarker: Capability,
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
        FeatureMarker: Capability,
        LeftRequirements: Requirements,
        RightRequirements: Requirements,
    {
        self.expression(TypedNode::Intrinsic {
            operation,
            arguments: vec![left.node, right.node],
        })
    }

    fn ternary<A, B, C, Output, FeatureMarker, FirstR, SecondR, ThirdR>(
        &mut self,
        operation: Operation,
        first: TypedExpr<'module, 'body, A, FirstR>,
        second: TypedExpr<'module, 'body, B, SecondR>,
        third: TypedExpr<'module, 'body, C, ThirdR>,
    ) -> TypedExpr<'module, 'body, Output, WithThree<FeatureMarker, FirstR, SecondR, ThirdR>>
    where
        FeatureMarker: Capability,
        FirstR: Requirements,
        SecondR: Requirements,
        ThirdR: Requirements,
    {
        self.expression(TypedNode::Intrinsic {
            operation,
            arguments: vec![first.node, second.node, third.node],
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
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<T::EqualityCapability, L, R>> {
        self.binary(Operation::Equal, left, right)
    }

    pub fn not_equal<T: TypedEquatable, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<T::EqualityCapability, L, R>> {
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

    pub fn int_bit_not<T: TypedInteger, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, With<IntegerBitwise, R>> {
        self.unary(Operation::IntBitNot, value)
    }

    pub fn int_bit_and<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<IntegerBitwise, L, R>> {
        self.binary(Operation::IntBitAnd, left, right)
    }

    pub fn int_bit_or<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<IntegerBitwise, L, R>> {
        self.binary(Operation::IntBitOr, left, right)
    }

    pub fn int_bit_xor<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, T, L>,
        right: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<IntegerBitwise, L, R>> {
        self.binary(Operation::IntBitXor, left, right)
    }

    pub fn int_shift_left_checked<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, T, L>,
        distance: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<CheckedIntegerShifts, L, R>> {
        self.binary(Operation::IntShiftLeftChecked, value, distance)
    }

    pub fn int_shift_right_checked<T: TypedInteger, L: Requirements, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, T, L>,
        distance: TypedExpr<'module, 'body, T, R>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<CheckedIntegerShifts, L, R>> {
        self.binary(Operation::IntShiftRightChecked, value, distance)
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

    pub fn float_trunc<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, F64, With<FloatingPointInspection, R>> {
        self.unary(Operation::FloatTrunc, value)
    }

    pub fn float_is_nan<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<FloatingPointInspection, R>> {
        self.unary(Operation::FloatIsNaN, value)
    }

    pub fn float_is_negative_zero<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<FloatingPointInspection, R>> {
        self.unary(Operation::FloatIsNegativeZero, value)
    }

    pub fn float_abs<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, F64, With<FloatingPointInspection, R>> {
        self.unary(Operation::FloatAbs, value)
    }

    pub fn widen_i32_to_i64<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, I32, R>,
    ) -> TypedExpr<'module, 'body, I64, With<IntegerConversions, R>> {
        self.unary(Operation::WidenI32ToI64, value)
    }

    pub fn narrow_i64_to_i32_checked<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, I64, R>,
    ) -> TypedExpr<'module, 'body, I32, With<IntegerConversions, R>> {
        self.unary(Operation::NarrowI64ToI32Checked, value)
    }

    pub fn string_concat<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, Text, L>,
        right: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Text, WithTwo<StringConcatenation, L, R>> {
        self.binary(Operation::StringConcat, left, right)
    }

    pub fn string_scalar_length<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, I64, With<StringInspection, R>> {
        self.unary(Operation::StringScalarLength, value)
    }

    pub fn string_utf16_length<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, I64, With<StringInspection, R>> {
        self.unary(Operation::StringUtf16Length, value)
    }

    pub fn string_index_of_literal<L: Requirements, R: Requirements>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, L>,
        needle: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Optional<I64>, WithTwo<StringInspection, L, R>> {
        self.binary(Operation::StringIndexOfLiteral, source, needle)
    }

    pub fn string_slice_scalars<SourceR, StartR, EndR>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, SourceR>,
        start: TypedExpr<'module, 'body, I64, StartR>,
        end: TypedExpr<'module, 'body, I64, EndR>,
    ) -> TypedExpr<'module, 'body, Text, WithThree<StringTransformation, SourceR, StartR, EndR>>
    where
        SourceR: Requirements,
        StartR: Requirements,
        EndR: Requirements,
    {
        self.ternary(Operation::StringSliceScalars, source, start, end)
    }

    pub fn string_is_empty<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<StringInspection, R>> {
        self.unary(Operation::StringIsEmpty, value)
    }

    pub fn string_contains<L: Requirements, R: Requirements>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, L>,
        needle: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<StringInspection, L, R>> {
        self.binary(Operation::StringContains, source, needle)
    }

    pub fn string_starts_with<L: Requirements, R: Requirements>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, L>,
        prefix: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<StringInspection, L, R>> {
        self.binary(Operation::StringStartsWith, source, prefix)
    }

    pub fn string_ends_with<L: Requirements, R: Requirements>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, L>,
        suffix: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<StringInspection, L, R>> {
        self.binary(Operation::StringEndsWith, source, suffix)
    }

    pub fn string_strip_prefix<L: Requirements, R: Requirements>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, L>,
        prefix: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Text, WithTwo<StringTransformation, L, R>> {
        self.binary(Operation::StringStripPrefix, source, prefix)
    }

    pub fn string_replace_all<SourceR, NeedleR, ReplacementR>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, SourceR>,
        needle: TypedExpr<'module, 'body, Text, NeedleR>,
        replacement: TypedExpr<'module, 'body, Text, ReplacementR>,
    ) -> TypedExpr<
        'module,
        'body,
        Text,
        WithThree<StringTransformation, SourceR, NeedleR, ReplacementR>,
    >
    where
        SourceR: Requirements,
        NeedleR: Requirements,
        ReplacementR: Requirements,
    {
        self.ternary(Operation::StringReplaceAll, source, needle, replacement)
    }

    pub fn string_replace_many<SourceR, NeedleR, ReplacementR, Tail>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, SourceR>,
        replacements: Cons<TypedReplacement<'module, 'body, NeedleR, ReplacementR>, Tail>,
    ) -> ReplaceManyExpr<'module, 'body, SourceR, NeedleR, ReplacementR, Tail>
    where
        SourceR: Requirements,
        NeedleR: Requirements,
        ReplacementR: Requirements,
        Cons<TypedReplacement<'module, 'body, NeedleR, ReplacementR>, Tail>:
            ReplaceManyRequirements<'module, 'body, SourceR>,
    {
        let mut arguments = vec![source.node];
        arguments.extend(replacements.into_replacement_nodes().0);
        self.expression(TypedNode::Intrinsic {
            operation: Operation::StringReplaceMany,
            arguments,
        })
    }

    pub fn string_truncate_utf8_bytes<L: Requirements, R: Requirements>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, L>,
        budget: TypedExpr<'module, 'body, F64, R>,
    ) -> TypedExpr<'module, 'body, Text, WithTwo<StringTransformation, L, R>> {
        self.binary(Operation::StringTruncateUtf8Bytes, source, budget)
    }

    pub fn string_trim_start<L: Requirements, R: Requirements>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, L>,
        characters: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Text, WithTwo<StringTransformation, L, R>> {
        self.binary(Operation::StringTrimStart, source, characters)
    }

    pub fn string_trim_end<L: Requirements, R: Requirements>(
        &mut self,
        source: TypedExpr<'module, 'body, Text, L>,
        characters: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Text, WithTwo<StringTransformation, L, R>> {
        self.binary(Operation::StringTrimEnd, source, characters)
    }

    pub fn bytes_concat<L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, Bytes, L>,
        right: TypedExpr<'module, 'body, Bytes, R>,
    ) -> TypedExpr<'module, 'body, Bytes, WithTwo<BytesOperations, L, R>> {
        self.binary(Operation::BytesConcat, left, right)
    }

    pub fn bytes_replace_all<SourceR, NeedleR, ReplacementR>(
        &mut self,
        source: TypedExpr<'module, 'body, Bytes, SourceR>,
        needle: TypedExpr<'module, 'body, Bytes, NeedleR>,
        replacement: TypedExpr<'module, 'body, Bytes, ReplacementR>,
    ) -> TypedExpr<'module, 'body, Bytes, WithThree<BytesOperations, SourceR, NeedleR, ReplacementR>>
    where
        SourceR: Requirements,
        NeedleR: Requirements,
        ReplacementR: Requirements,
    {
        self.ternary(Operation::BytesReplaceAll, source, needle, replacement)
    }

    pub fn bytes_length<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Bytes, R>,
    ) -> TypedExpr<'module, 'body, I64, With<BytesOperations, R>> {
        self.unary(Operation::BytesLength, value)
    }

    pub fn bytes_is_empty<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Bytes, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<BytesOperations, R>> {
        self.unary(Operation::BytesIsEmpty, value)
    }

    pub fn list_length<T, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, List<T>, R>,
    ) -> TypedExpr<'module, 'body, I64, With<ListOperations, R>> {
        self.unary(Operation::ListLength, value)
    }

    pub fn list_is_empty<T, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, List<T>, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<ListOperations, R>> {
        self.unary(Operation::ListIsEmpty, value)
    }

    pub fn list_get_checked<T, ListR: Requirements, IndexR: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, List<T>, ListR>,
        index: TypedExpr<'module, 'body, I64, IndexR>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<ListOperations, ListR, IndexR>> {
        self.binary(Operation::ListGetChecked, value, index)
    }

    pub fn list_append<T, ListR: Requirements, ValueR: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, List<T>, ListR>,
        element: TypedExpr<'module, 'body, T, ValueR>,
    ) -> TypedExpr<'module, 'body, List<T>, WithTwo<ListOperations, ListR, ValueR>> {
        self.binary(Operation::ListAppend, value, element)
    }

    pub fn list_concat<T, L: Requirements, R: Requirements>(
        &mut self,
        left: TypedExpr<'module, 'body, List<T>, L>,
        right: TypedExpr<'module, 'body, List<T>, R>,
    ) -> TypedExpr<'module, 'body, List<T>, WithTwo<ListOperations, L, R>> {
        self.binary(Operation::ListConcat, left, right)
    }

    pub fn list_contains<T: TypedEquatable, ListR: Requirements, ValueR: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, List<T>, ListR>,
        element: TypedExpr<'module, 'body, T, ValueR>,
    ) -> TypedExpr<'module, 'body, Bool, WithTwo<ListOperations, ListR, ValueR>> {
        self.binary(Operation::ListContains, value, element)
    }

    pub fn list_index_of<T: TypedEquatable, ListR: Requirements, ValueR: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, List<T>, ListR>,
        element: TypedExpr<'module, 'body, T, ValueR>,
    ) -> TypedExpr<'module, 'body, Optional<I64>, WithTwo<ListOperations, ListR, ValueR>> {
        self.binary(Operation::ListIndexOf, value, element)
    }

    pub fn option_is_some<T, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Optional<T>, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<OptionOperations, R>> {
        self.unary(Operation::OptionIsSome, value)
    }

    pub fn option_is_none<T, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Optional<T>, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<OptionOperations, R>> {
        self.unary(Operation::OptionIsNone, value)
    }

    pub fn option_unwrap_or<T, OptionR: Requirements, FallbackR: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Optional<T>, OptionR>,
        fallback: TypedExpr<'module, 'body, T, FallbackR>,
    ) -> TypedExpr<'module, 'body, T, WithTwo<OptionOperations, OptionR, FallbackR>> {
        self.binary(Operation::OptionUnwrapOr, value, fallback)
    }

    pub fn result_is_ok<Ok, Error, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, ResultValue<Ok, Error>, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<ResultOperations, R>> {
        self.unary(Operation::ResultIsOk, value)
    }

    pub fn result_is_err<Ok, Error, R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, ResultValue<Ok, Error>, R>,
    ) -> TypedExpr<'module, 'body, Bool, With<ResultOperations, R>> {
        self.unary(Operation::ResultIsErr, value)
    }

    pub fn string_to_utf8<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Text, R>,
    ) -> TypedExpr<'module, 'body, Bytes, With<Utf8Conversions, R>> {
        self.unary(Operation::StringToUtf8, value)
    }

    pub fn string_from_utf8_checked<R: Requirements>(
        &mut self,
        value: TypedExpr<'module, 'body, Bytes, R>,
    ) -> TypedExpr<'module, 'body, Text, With<Utf8Conversions, R>> {
        self.unary(Operation::StringFromUtf8Checked, value)
    }
}

fn lower_expression(body: &mut BodyBuilder<'_>, node: TypedNode) -> crate::Expr {
    match node {
        TypedNode::Literal(value) => body.literal(value),
        TypedNode::Local(name) => body.local(name),
        TypedNode::Constant(constant) => body.constant(constant),
        TypedNode::LetValue {
            name,
            value,
            result,
        } => {
            let value = lower_expression(body, *value);
            let binding = body.let_statement(name, None, value);
            let result = lower_expression(body, *result);
            let block = body.block([binding], Some(result));
            body.block_expression(block)
        }
        TypedNode::IfValue {
            condition,
            then_value,
            else_value,
        } => {
            let condition = lower_expression(body, *condition);
            let then_value = lower_expression(body, *then_value);
            let then_block = body.block([], Some(then_value));
            let else_value = lower_expression(body, *else_value);
            let else_block = body.block([], Some(else_value));
            body.if_else(condition, then_block, else_block)
        }
        TypedNode::ForEach {
            binding,
            iterable,
            body: loop_value,
        } => {
            let iterable = lower_expression(body, *iterable);
            let loop_value = lower_expression(body, *loop_value);
            let evaluate = body.expression_statement(loop_value);
            let loop_body = body.block([evaluate], None);
            let statement = body.for_each(binding, iterable, loop_body);
            let unit = body.literal(Value::unit());
            let block = body.block([statement], Some(unit));
            body.block_expression(block)
        }
        TypedNode::Record { record, fields } => {
            let fields = fields
                .into_iter()
                .map(|(field, value)| (field, lower_expression(body, value)))
                .collect::<Vec<_>>();
            body.record(record, fields)
        }
        TypedNode::Enum {
            enumeration,
            variant,
        } => body.enumeration(enumeration, variant, []),
        TypedNode::EnumMatch {
            enumeration,
            value,
            arms,
        } => {
            let value = lower_expression(body, *value);
            let arms = arms
                .into_iter()
                .map(|(variant, value)| {
                    let pattern = body.enum_pattern(enumeration, variant, []);
                    let value = lower_expression(body, value);
                    let block = body.block([], Some(value));
                    body.match_arm(pattern, block)
                })
                .collect::<Vec<_>>();
            body.match_value(value, arms)
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
        TypedNode::List { element, values } => {
            let values = values
                .into_iter()
                .map(|value| lower_expression(body, value))
                .collect::<Vec<_>>();
            body.list(element, values)
        }
        TypedNode::Some(value) => {
            let value = lower_expression(body, *value);
            body.some(value)
        }
        TypedNode::None(inner) => body.none(inner),
        TypedNode::Ok { value, error } => {
            let value = lower_expression(body, *value);
            body.ok(value, error)
        }
        TypedNode::Err { value, ok } => {
            let value = lower_expression(body, *value);
            body.err(value, ok)
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

fn lower_constant_expression(
    body: &mut BodyBuilder<'_>,
    node: TypedConstantNode,
) -> crate::ConstantExpr {
    match node {
        TypedConstantNode::Literal(value) => body.constant_literal(value),
        TypedConstantNode::Reference(constant) => body.constant_reference(constant),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDialect;

    #[derive(Clone, Copy)]
    struct TestMapping<C: Capability>(PhantomData<C>);

    impl<C: Capability + 'static> CapabilityMapping<TestDialect> for TestMapping<C> {
        type Capability = C;
        type Context = ();
        type Input = ();
        type Output = ();
        type Error = ();

        fn lower(&self, _context: &mut (), _input: ()) -> Result<(), ()> {
            Ok(())
        }
    }

    const fn test_mapping<C: Capability>() -> TestMapping<C> {
        TestMapping(PhantomData)
    }

    #[derive(Clone, Copy)]
    struct TestI32Mapping;

    impl CapabilityMapping<TestDialect> for TestI32Mapping {
        type Capability = I32Values;
        type Context = usize;
        type Input = i32;
        type Output = i64;
        type Error = ();

        fn lower(&self, context: &mut usize, input: i32) -> Result<i64, ()> {
            *context += 1;
            Ok(i64::from(input))
        }
    }

    #[test]
    fn registered_subset_exposes_its_executable_mapping() {
        let plugin = language_plugin(TestDialect).support(TestI32Mapping).build();
        let mut invocations = 0;
        let output = plugin
            .mapping_for::<I32Values>()
            .lower(&mut invocations, 42)
            .expect("test mapping succeeds");
        assert_eq!(output, 42);
        assert_eq!(invocations, 1);
    }

    #[test]
    fn modules_and_unit_values_are_inferred_from_construction() {
        let program = typed_program(portable_name!("module_and_unit"), |builder| {
            builder
                .function(
                    portable_name!("unit"),
                    typed_list![],
                    Unit::TYPE,
                    |body, _| body.unit(),
                )
                .builder
        });
        let plugin = language_plugin(TestDialect)
            .support(test_mapping::<Modules>())
            .support(test_mapping::<Functions>())
            .support(test_mapping::<UnitValues>())
            .build();

        fn admit<P, R: Requirements>(_plugin: &P, _program: &TypedProgram<R>)
        where
            P: SupportsAll<R>,
        {
        }

        admit(&plugin, &program);
        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("module/unit typed program lowers to CoreIR");
        portable_core_ir::verify_core(&core).expect("module/unit typed CoreIR verifies");
    }

    #[test]
    fn constants_and_aliases_are_typed_and_inferred() {
        let program = typed_program(portable_name!("constants_and_aliases"), |builder| {
            let added =
                builder.constant(portable_name!("TRUTH"), Bool::TYPE, |body| body.bool(true));
            let truth = added.handle;
            added
                .builder
                .alias(portable_name!("Count"), I64::TYPE, |builder, count| {
                    let builder = builder
                        .function(
                            portable_name!("truth"),
                            typed_list![],
                            Bool::TYPE,
                            |body, _| body.constant(truth),
                        )
                        .builder;
                    builder
                        .function(
                            portable_name!("count"),
                            typed_list![],
                            count.ty(),
                            |body, _| {
                                let value = body.i64(7);
                                body.alias_wrap(&count, value)
                            },
                        )
                        .builder
                        .function(
                            portable_name!("unwrapped_count"),
                            typed_list![],
                            I64::TYPE,
                            |body, _| {
                                let value = body.i64(7);
                                let value = body.alias_wrap(&count, value);
                                body.alias_unwrap(&count, value)
                            },
                        )
                        .builder
                })
        });
        let plugin = language_plugin(TestDialect)
            .support(test_mapping::<Modules>())
            .support(test_mapping::<Constants>())
            .support(test_mapping::<TypeAliases>())
            .support(test_mapping::<Functions>())
            .support(test_mapping::<BoolValues>())
            .support(test_mapping::<I64Values>())
            .build();

        fn admit<P, R: Requirements>(_plugin: &P, _program: &TypedProgram<R>)
        where
            P: SupportsAll<R>,
        {
        }

        admit(&plugin, &program);
        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("typed constants and aliases lower to CoreIR");
        portable_core_ir::verify_core(&core).expect("typed constants and aliases verify");
    }

    #[test]
    fn local_bindings_and_conditionals_are_typed_and_inferred() {
        let program = typed_program(portable_name!("bindings_and_conditionals"), |builder| {
            builder
                .function(
                    portable_name!("choose"),
                    typed_list![],
                    I32::TYPE,
                    |body, _| {
                        let bound = body.i32(7);
                        body.let_value(portable_name!("bound"), bound, |body, local| {
                            let condition = body.bool(true);
                            let yes = body.read_binding(local);
                            let no = body.i32(0);
                            body.if_else(condition, yes, no)
                        })
                    },
                )
                .builder
        });
        let plugin = language_plugin(TestDialect)
            .support(test_mapping::<Modules>())
            .support(test_mapping::<Functions>())
            .support(test_mapping::<I32Values>())
            .support(test_mapping::<BoolValues>())
            .support(test_mapping::<LocalBindings>())
            .support(test_mapping::<Conditionals>())
            .build();

        fn admit<P, R: Requirements>(_plugin: &P, _program: &TypedProgram<R>)
        where
            P: SupportsAll<R>,
        {
        }

        admit(&plugin, &program);
        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("typed bindings and conditionals lower to CoreIR");
        portable_core_ir::verify_core(&core).expect("typed bindings and conditionals verify");
    }

    #[test]
    fn bounded_loops_and_call_propagation_are_typed_and_inferred() {
        let program = typed_program(portable_name!("loops_and_propagation"), |builder| {
            let added = builder.function(
                portable_name!("identity"),
                typed_list![parameter(portable_name!("value"), I32::TYPE)],
                I32::TYPE,
                |body, values| body.read(values.head),
            );
            let identity = added.handle;
            added
                .builder
                .function(
                    portable_name!("visit"),
                    typed_list![],
                    Unit::TYPE,
                    |body, _| {
                        let item = body.i32(7);
                        let items = body.list(I32::TYPE, typed_list![item]);
                        body.for_each(portable_name!("item"), items, |body, item| {
                            let item = body.read_loop_item(item);
                            let called = body.call(identity, typed_list![item]);
                            body.let_value(portable_name!("called"), called, |body, _| body.unit())
                        })
                    },
                )
                .builder
        });
        let plugin = language_plugin(TestDialect)
            .support(test_mapping::<Modules>())
            .support(test_mapping::<Functions>())
            .support(test_mapping::<I32Values>())
            .support(test_mapping::<ListValues>())
            .support(test_mapping::<Loops>())
            .support(test_mapping::<LocalBindings>())
            .support(test_mapping::<ResultPropagation>())
            .support(test_mapping::<UnitValues>())
            .build();

        fn admit<P, R: Requirements>(_plugin: &P, _program: &TypedProgram<R>)
        where
            P: SupportsAll<R>,
        {
        }

        admit(&plugin, &program);
        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("typed loop and propagation lower to CoreIR");
        portable_core_ir::verify_core(&core).expect("typed loop and propagation verify");
    }

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
    fn extended_intrinsic_surface_replays_through_core_ir() {
        let program = typed_program(portable_name!("extended_features"), |builder| {
            let builder = builder
                .function(
                    portable_name!("bitwise"),
                    typed_list![],
                    I32::TYPE,
                    |body, _| {
                        let value = body.i32(12);
                        let value = body.int_bit_not(value);
                        let mask = body.i32(7);
                        let value = body.int_bit_and(value, mask);
                        let flag = body.i32(16);
                        let value = body.int_bit_or(value, flag);
                        let toggle = body.i32(3);
                        body.int_bit_xor(value, toggle)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("shifts"),
                    typed_list![],
                    I64::TYPE,
                    |body, _| {
                        let value = body.i64(8);
                        let one = body.i64(1);
                        let value = body.int_shift_left_checked(value, one);
                        let two = body.i64(2);
                        body.int_shift_right_checked(value, two)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("float_inspection"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let value = body.f64(-1.75);
                        let truncated = body.float_trunc(value);
                        let absolute = body.float_abs(truncated);
                        let nan = body.float_is_nan(absolute);
                        let negative_zero = body.f64(-0.0);
                        let negative_zero = body.float_is_negative_zero(negative_zero);
                        body.bool_or(nan, negative_zero)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("integer_conversions"),
                    typed_list![],
                    I32::TYPE,
                    |body, _| {
                        let value = body.i32(42);
                        let wide = body.widen_i32_to_i64(value);
                        body.narrow_i64_to_i32_checked(wide)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("char_value"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let left = body.char('λ');
                        let right = body.char('λ');
                        body.equal(left, right)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("string_inspection"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let source = body.text("polyrust");
                        let empty = body.string_is_empty(source);
                        let source = body.text("polyrust");
                        let needle = body.text("rust");
                        let contains = body.string_contains(source, needle);
                        let first = body.bool_or(empty, contains);
                        let source = body.text("polyrust");
                        let prefix = body.text("poly");
                        let starts = body.string_starts_with(source, prefix);
                        let source = body.text("polyrust");
                        let suffix = body.text("rust");
                        let ends = body.string_ends_with(source, suffix);
                        let second = body.bool_and(starts, ends);
                        let predicates = body.bool_and(first, second);
                        let source = body.text("λ");
                        let scalars = body.string_scalar_length(source);
                        let source = body.text("λ");
                        let utf16 = body.string_utf16_length(source);
                        let lengths = body.equal(scalars, utf16);
                        body.bool_and(predicates, lengths)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("string_index"),
                    typed_list![],
                    option_type(I64::TYPE),
                    |body, _| {
                        let source = body.text("polyrust");
                        let needle = body.text("rust");
                        body.string_index_of_literal(source, needle)
                    },
                )
                .builder;
            builder
                .function(
                    portable_name!("string_transform"),
                    typed_list![],
                    Text::TYPE,
                    |body, _| {
                        let source = body.text("  poly-rust  ");
                        let start = body.i64(0);
                        let end = body.i64(13);
                        let value = body.string_slice_scalars(source, start, end);
                        let prefix = body.text("  ");
                        let value = body.string_strip_prefix(value, prefix);
                        let needle = body.text("-");
                        let replacement_value = body.text(" ");
                        let value = body.string_replace_all(value, needle, replacement_value);
                        let needle = body.text("poly");
                        let replacement_value = body.text("many");
                        let pair = replacement(needle, replacement_value);
                        let value = body.string_replace_many(value, typed_list![pair]);
                        let budget = body.f64(64.0);
                        let value = body.string_truncate_utf8_bytes(value, budget);
                        let characters = body.text(" ");
                        let value = body.string_trim_start(value, characters);
                        let characters = body.text(" ");
                        body.string_trim_end(value, characters)
                    },
                )
                .builder
        });

        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("extended typed constructors lower to CoreIR");
        portable_core_ir::verify_core(&core).expect("extended typed constructors verify");
    }

    #[test]
    fn extended_collection_surface_replays_through_core_ir() {
        let program = typed_program(portable_name!("extended_collections"), |builder| {
            let builder = builder
                .function(
                    portable_name!("bytes_round_trip"),
                    typed_list![],
                    Text::TYPE,
                    |body, _| {
                        let text = body.text("poly");
                        let left = body.string_to_utf8(text);
                        let right = body.bytes(b"rust".to_vec());
                        let bytes = body.bytes_concat(left, right);
                        let needle = body.bytes(b"rust".to_vec());
                        let replacement_value = body.bytes(b"lang".to_vec());
                        let bytes = body.bytes_replace_all(bytes, needle, replacement_value);
                        body.string_from_utf8_checked(bytes)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("bytes_inspection"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let bytes = body.bytes(Vec::new());
                        let empty = body.bytes_is_empty(bytes);
                        let bytes = body.bytes(b"abc".to_vec());
                        let length = body.bytes_length(bytes);
                        let three = body.i64(3);
                        let expected = body.equal(length, three);
                        body.bool_and(empty, expected)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("list_value"),
                    typed_list![],
                    list_type(I32::TYPE),
                    |body, _| {
                        let one = body.i32(1);
                        let list = body.list(I32::TYPE, typed_list![one]);
                        let two = body.i32(2);
                        let list = body.list_append(list, two);
                        let three = body.i32(3);
                        let tail = body.list(I32::TYPE, typed_list![three]);
                        body.list_concat(list, tail)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("list_get"),
                    typed_list![],
                    I32::TYPE,
                    |body, _| {
                        let value = body.i32(7);
                        let list = body.list(I32::TYPE, typed_list![value]);
                        let index = body.i64(0);
                        body.list_get_checked(list, index)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("list_inspection"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let list = body.list(I32::TYPE, typed_list![]);
                        let empty = body.list_is_empty(list);
                        let one = body.i32(1);
                        let list = body.list(I32::TYPE, typed_list![one]);
                        let one = body.i32(1);
                        let contains = body.list_contains(list, one);
                        body.bool_and(empty, contains)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("list_length"),
                    typed_list![],
                    I64::TYPE,
                    |body, _| {
                        let one = body.i32(1);
                        let list = body.list(I32::TYPE, typed_list![one]);
                        body.list_length(list)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("list_index"),
                    typed_list![],
                    option_type(I64::TYPE),
                    |body, _| {
                        let one = body.i32(1);
                        let list = body.list(I32::TYPE, typed_list![one]);
                        let one = body.i32(1);
                        body.list_index_of(list, one)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("option_value"),
                    typed_list![],
                    I32::TYPE,
                    |body, _| {
                        let seven = body.i32(7);
                        let value = body.some(seven);
                        let fallback = body.i32(0);
                        body.option_unwrap_or(value, fallback)
                    },
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("option_inspection"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let seven = body.i32(7);
                        let some = body.some(seven);
                        let is_some = body.option_is_some(some);
                        let none = body.none(I32::TYPE);
                        let is_none = body.option_is_none(none);
                        body.bool_and(is_some, is_none)
                    },
                )
                .builder;
            builder
                .function(
                    portable_name!("result_inspection"),
                    typed_list![],
                    Bool::TYPE,
                    |body, _| {
                        let value = body.i32(7);
                        let ok = body.ok(value, Text::TYPE);
                        let is_ok = body.result_is_ok(ok);
                        let error = body.text("error");
                        let err = body.err(error, I32::TYPE);
                        let is_err = body.result_is_err(err);
                        body.bool_and(is_ok, is_err)
                    },
                )
                .builder
        });
        let core = portable_core_ir::lower_checked(program.checked_program())
            .expect("typed collection constructors lower to CoreIR");
        portable_core_ir::verify_core(&core).expect("typed collection constructors verify");
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
