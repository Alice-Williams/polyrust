//! Compile-only input boundary probe with no concrete target backend dependency.
#![feature(rustc_private)]
#![forbid(unsafe_code)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

#[path = "../src/source_origin/mod.rs"]
mod source_origin;

#[path = "../src/source_capabilities/mod.rs"]
mod source_capabilities;
use source_capabilities::*;
mod constant_domains;

#[derive(Clone, Copy)]
struct NumericMapping;
impl Mapping for NumericMapping {
    type Capability = LiteralValues;
    type Context<'tcx> = u32;
    type Output = u32;

    fn lower<'tcx>(&self, context: &mut u32, input: LiteralInput<'tcx>) -> Result<u32, String> {
        Ok(*context
            + match input.value() {
                LiteralValue::I32(_) => 1,
                LiteralValue::I64(_) => 2,
                LiteralValue::Bool(_) => 3,
                LiteralValue::F64(_) => 4,
            })
    }
}

#[derive(Clone, Copy)]
struct BooleanMapping;
impl Mapping for BooleanMapping {
    type Capability = LiteralValues;
    type Context<'tcx> = bool;
    type Output = bool;

    fn lower<'tcx>(&self, context: &mut bool, input: LiteralInput<'tcx>) -> Result<bool, String> {
        Ok(*context && matches!(input.value(), LiteralValue::Bool(true)))
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
    let _: Result<u32, String> = NumericBindings.mapping().lower(&mut 0, input);
    let _: Result<bool, String> = BooleanBindings.mapping().lower(&mut false, input);
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
    let _ = (input.expression, input.parent, input.completion);
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

fn negation<'tcx>(
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    let _ = NegationInput::read(checked, expression).map(|input| input.operand());
}

fn capability<C: Capability>() {}

fn literal<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    let _ = LiteralInput::read(tcx, checked, expression).map(|input| {
        input.require_context(tcx, checked).unwrap();
        input.value()
    });
}

fn lazy_boolean<'tcx>(
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    let _ = LazyBooleanInput::read(checked, expression)
        .map(|input| (input.operator(), input.left(), input.right()));
}

fn bitwise<'tcx>(
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    let _ = BitwiseInput::read(checked, expression).map(|input| match input.operands() {
        BitwiseOperands::Complement(operand) => operand,
        BitwiseOperands::Binary(operator, left, right) => {
            let _ = (operator, right);
            left
        }
    });
}

fn eager_boolean<'tcx>(
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    let _ = EagerBooleanInput::read(checked, expression)
        .map(|input| (input.operator(), input.left(), input.right()));
}

fn constant<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    if let rustc_hir::ExprKind::Path(ref path) = expression.kind
        && ConstantInput::is_constant(checked, expression, path)
    {
        let _ = ConstantInput::read(tcx, checked, expression).map(|input| input.value());
    }
}

fn local_constant<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    statement: &'tcx rustc_hir::Stmt<'tcx>,
) {
    let _ = LocalConstantInput::read(tcx, statement).map(|input| input.value());
}

fn public_constant<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
    definition: rustc_hir::def_id::DefId,
) {
    let _ = ConstantImportInput::discover(tcx, checked, expression);
    let _ = ConstantImportInput::read(tcx, definition)
        .map(|input| (input.tcx(), input.definition(), input.value()));
    let mut cache = source_origin::Cache::default();
    let _ = source_origin::read(
        tcx,
        &mut cache,
        definition,
        portable_codegen::RustSourceNode::Declaration,
        tcx.def_span(definition),
    );
    if let Ok(inventory) = source_origin::public_api::Inventory::read(tcx, &mut cache) {
        let _ = ConstantDeclarationInput::read(tcx, &inventory, definition)
            .map(|input| (input.tcx(), input.definition(), input.value()));
    }
    if PublicConstantReadInput::requires_reference(tcx, definition) {
        let _ = PublicConstantReadInput::read(tcx, checked, expression)
            .map(|input| (input.definition(), input.value()));
    }
}

fn unit<'tcx>(
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    let _ = UnitInput::read(checked, expression, expression.hir_id).map(|input| {
        let _ = input.scope();
        match input.operation() {
            UnitOperation::Empty => {}
            UnitOperation::Call(expression) | UnitOperation::Block(expression) => {
                let _ = expression;
            }
            UnitOperation::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                let _ = (condition, then_value, else_value);
            }
        }
    });
}
fn wrapping<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    if let Ok(Some(input)) = WrappingInput::discover(tcx, checked, expression) {
        let _ = input.require_context(tcx, checked);
        let _ = (input.receiver(), input.width());
    }
}
fn floating<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    if let Ok(input) = FloatingInput::read(tcx, checked, expression) {
        let _ = input.require_context(tcx, checked);
        let _ = input.operand();
    }
}
fn nan<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    checked: &rustc_middle::ty::TypeckResults<'tcx>,
    expression: &'tcx rustc_hir::Expr<'tcx>,
) {
    if let Ok(Some(input)) = NaNInput::discover(tcx, checked, expression) {
        let _ = input.require_context(tcx, checked);
        let _ = input.receiver();
    }
}
fn main() {
    capability::<FloatingNaN>();
    let _ = nan;
    capability::<FloatingNegation>();
    let _ = floating;
    let _ = (constant_domains::distinct, constant_domains::witnesses);
    capability::<WrappingNegation>();
    let _ = wrapping;
    let _ = [WrappingWidth::I32, WrappingWidth::I64];
    capability::<UnitEffects>();
    let _ = unit;
    let _ = [ControlCompletion::Return, ControlCompletion::Effect];
    capability::<PublicConstantImports>();
    capability::<PublicConstants>();
    capability::<PublicConstantReads>();
    let _ = public_constant;
    capability::<LocalConstants>();
    let _ = local_constant;
    capability::<ScalarConstants>();
    let _ = constant;
    capability::<EagerBooleans>();
    let _ = eager_boolean;
    let _ = [
        EagerBooleanOperator::And,
        EagerBooleanOperator::Or,
        EagerBooleanOperator::Xor,
    ];
    let _ = [
        BitwiseOperator::And,
        BitwiseOperator::Or,
        BitwiseOperator::Xor,
    ];
    capability::<IntegerBitwise>();
    let _ = bitwise;
    let _ = [LazyBooleanOperator::And, LazyBooleanOperator::Or];
    capability::<BooleanNegation>();
    capability::<ShortCircuitBooleans>();
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
        map_both,
        literal,
        call,
        entry,
        function,
        control,
        object,
        record,
        place,
        comparison,
        borrow,
        negation,
        lazy_boolean,
    );
}
