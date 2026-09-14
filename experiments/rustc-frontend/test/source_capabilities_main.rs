//! Compile-only input boundary probe with no C, Java or codegen dependency.
#![feature(rustc_private)]
#![forbid(unsafe_code)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_middle;

#[path = "../src/source_capabilities/mod.rs"]
mod source_capabilities;
use source_capabilities::*;

#[derive(Clone, Copy)]
struct NumericMapping;
impl Mapping for NumericMapping {
    type Capability = LiteralValues;
    type Context<'tcx> = u32;
    type Output = u32;

    fn lower<'tcx>(&self, context: &mut u32, input: LiteralInput<'tcx>) -> Result<u32, String> {
        Ok(*context + input.0.hir_id.local_id.as_u32())
    }
}

#[derive(Clone, Copy)]
struct BooleanMapping;
impl Mapping for BooleanMapping {
    type Capability = LiteralValues;
    type Context<'tcx> = bool;
    type Output = bool;

    fn lower<'tcx>(&self, context: &mut bool, input: LiteralInput<'tcx>) -> Result<bool, String> {
        Ok(*context && input.0.hir_id.local_id.as_u32() == 0)
    }
}

struct NumericBindings;
impl Supports<LiteralValues> for NumericBindings {
    type Mapping = NumericMapping;
    fn mapping(&self) -> NumericMapping {
        NumericMapping
    }
}

struct BooleanBindings;
impl Supports<LiteralValues> for BooleanBindings {
    type Mapping = BooleanMapping;
    fn mapping(&self) -> BooleanMapping {
        BooleanMapping
    }
}

// Type-check both executable signatures against the very same input identity.
// No fabricated HIR value is constructed or evaluated by this probe.
fn map_both(input: LiteralInput<'_>) {
    let expression = input.0;
    let _: Result<u32, String> = NumericBindings.mapping().lower(&mut 0, input);
    let _: Result<bool, String> = BooleanBindings
        .mapping()
        .lower(&mut false, LiteralInput(expression));
}

fn call(input: CallInput<'_>) {
    let _ = input.0;
}
fn entry(input: EntryInput<'_>) {
    let _ = (input.tcx, input.root);
}
fn function(input: FunctionInput<'_>) {
    let _ = (input.tcx, input.function);
}
fn control(input: ControlInput<'_>) {
    let _ = (input.expression, input.parent);
}
fn object(input: TypeInput<'_>) {
    let _ = input.0;
}
fn record(input: RecordInput<'_>) {
    let _ = input.0;
}
fn place(input: PlaceInput<'_>) {
    let _ = input.0;
}
fn comparison(input: ComparisonInput<'_>) {
    let _ = input.0;
}
fn borrow(input: BorrowInput<'_>) {
    let _ = input.0;
}

fn capability<C: Capability>() {}

fn main() {
    capability::<DirectCalls>();
    capability::<EntrySignatures>();
    capability::<FunctionSignatures>();
    capability::<LexicalControl>();
    capability::<LiteralValues>();
    capability::<ObjectTypes>();
    capability::<RecordInitializers>();
    capability::<ResolvedPlaces>();
    capability::<ScalarComparisons>();
    capability::<SharedBorrows>();
    let _ = (
        map_both, call, entry, function, control, object, record, place, comparison, borrow,
    );
}
