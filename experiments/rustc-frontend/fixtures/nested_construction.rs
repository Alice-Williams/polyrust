//! Closed nested operation grammar; this is not whole-body admission.
#![allow(dead_code, private_interfaces)]

mod nested_budgets;
pub use nested_budgets::*;

fn allocator_type(_: std::alloc::System) {}

struct Leaf {
    value: Box<i32>,
}
struct Branch {
    left: Leaf,
    right: Leaf,
}
struct Mixed {
    nested: Leaf,
    extra: Box<i32>,
}
struct Deep {
    branch: Branch,
}
pub fn repeated(left: Leaf, right: Leaf) -> Branch {
    Branch { left, right }
}
pub fn reversed(left: Leaf, right: Leaf) -> Branch {
    Branch { right, left }
}
pub fn mixed(nested: Leaf, extra: Box<i32>) -> Mixed {
    Mixed { nested, extra }
}
pub fn deeper(branch: Branch) -> Deep {
    Deep { branch }
}
type Alias = Leaf;
pub fn alias(left: Alias, right: Alias) -> Branch {
    Branch { left, right }
}
mod other {
    pub struct Leaf {
        pub value: Box<i32>,
    }
    pub struct Branch {
        pub nested: Leaf,
    }
}
pub fn distinct(nested: other::Leaf) -> other::Branch {
    other::Branch { nested }
}
pub fn flat(value: Box<i32>) -> Leaf {
    Leaf { value }
}
pub fn update(left: Leaf, original: Branch) -> Branch {
    Branch { left, ..original }
}

struct Generic<T> {
    value: T,
}
struct GenericChild {
    nested: Generic<Box<i32>>,
}
pub fn generic(nested: Generic<Box<i32>>) -> GenericChild {
    GenericChild { nested }
}
pub fn generic_root(value: Leaf) -> Generic<Leaf> {
    Generic { value }
}
struct ScalarChild {
    nested: Leaf,
    value: i32,
}
pub fn scalar(nested: Leaf, value: i32) -> ScalarChild {
    ScalarChild { nested, value }
}
struct WrongBox {
    nested: Leaf,
    value: Box<bool>,
}
pub fn payload(nested: Leaf, value: Box<bool>) -> WrongBox {
    WrongBox { nested, value }
}
struct DoubleBox {
    nested: Leaf,
    value: Box<Box<i32>>,
}
pub fn boxed_box(nested: Leaf, value: Box<Box<i32>>) -> DoubleBox {
    DoubleBox { nested, value }
}
struct Reference<'a> {
    nested: &'a Leaf,
}
pub fn reference(nested: &Leaf) -> Reference<'_> {
    Reference { nested }
}
struct ArrayChild {
    nested: [Leaf; 1],
}
pub fn array(nested: [Leaf; 1]) -> ArrayChild {
    ArrayChild { nested }
}
struct Tuple(Box<i32>);
struct TupleChild {
    nested: Tuple,
}
pub fn tuple(nested: Tuple) -> TupleChild {
    TupleChild { nested }
}
struct Unit;
struct UnitChild {
    nested: Unit,
}
pub fn unit(nested: Unit) -> UnitChild {
    UnitChild { nested }
}
struct Empty {}
struct EmptyChild {
    nested: Empty,
}
pub fn empty(nested: Empty) -> EmptyChild {
    EmptyChild { nested }
}
enum Choice {
    Owned(Box<i32>),
}
struct EnumChild {
    nested: Choice,
}
pub fn enumeration(nested: Choice) -> EnumChild {
    EnumChild { nested }
}
struct Custom {
    value: Box<i32>,
}
impl Drop for Custom {
    fn drop(&mut self) {}
}
struct CustomChild {
    nested: Custom,
}
pub fn custom(nested: Custom) -> CustomChild {
    CustomChild { nested }
}
#[repr(C)]
struct Represented {
    value: Box<i32>,
}
struct ReprChild {
    nested: Represented,
}
pub fn representation(nested: Represented) -> ReprChild {
    ReprChild { nested }
}
struct ExternalChild {
    nested: std::ops::Range<i32>,
}
pub fn external(nested: std::ops::Range<i32>) -> ExternalChild {
    ExternalChild { nested }
}

union Union {
    nested: std::mem::ManuallyDrop<Leaf>,
}
pub fn union_record(nested: std::mem::ManuallyDrop<Leaf>) -> Union {
    Union { nested }
}
mod counterfeit {
    pub struct Box {
        pub value: i32,
    }
}
struct Counterfeit {
    nested: Leaf,
    fake: counterfeit::Box,
}
pub fn counterfeit(nested: Leaf, fake: counterfeit::Box) -> Counterfeit {
    Counterfeit { nested, fake }
}
pub fn diverging(left: Leaf) -> Branch {
    Branch {
        left,
        right: panic!("no value"),
    }
}

#[repr(Rust)]
struct RustLeaf {
    value: Box<i32>,
}
#[repr(Rust)]
struct RustRoot {
    nested: RustLeaf,
}
pub fn explicit_rust(nested: RustLeaf) -> RustRoot {
    RustRoot { nested }
}
struct StaticReference {
    nested: &'static Leaf,
}
pub fn static_reference(nested: &'static Leaf) -> StaticReference {
    StaticReference { nested }
}
struct Foreign {
    nested: std::time::Duration,
}
pub fn external_nongeneric(nested: std::time::Duration) -> Foreign {
    Foreign { nested }
}
#[repr(packed)]
struct PackedLeaf {
    value: Box<i32>,
}
struct Packed {
    nested: PackedLeaf,
}
pub fn packed(nested: PackedLeaf) -> Packed {
    Packed { nested }
}
#[repr(align(16))]
struct AlignedLeaf {
    value: Box<i32>,
}
struct Aligned {
    nested: AlignedLeaf,
}
pub fn aligned(nested: AlignedLeaf) -> Aligned {
    Aligned { nested }
}
struct Pointer {
    nested: *const Leaf,
}
pub fn pointer(nested: *const Leaf) -> Pointer {
    Pointer { nested }
}
