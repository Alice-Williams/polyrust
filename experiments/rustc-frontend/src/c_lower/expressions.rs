//! Traversal selects executable mappings; their owners define operation rules.
use super::{
    Reader, Result, c,
    capabilities::{
        AbsoluteInput, AdditionInput, ArithmeticInput, BitwiseInput, BooleanNegation, BorrowInput,
        CallInput, ComparisonInput, ConstantInput, DirectCalls, EagerBooleanInput, EagerBooleans,
        FloatingAbsolute, FloatingArithmetic, FloatingInput, FloatingNaN, FloatingNegation,
        FloatingRemainder, FloatingTruncation, IntegerBitwise, LazyBooleanInput, LiteralInput,
        LiteralValues, Mapping, NaNInput, NegationInput, PlaceInput, PublicConstantReadInput,
        PublicConstantReads, RemainderInput, ResolvedPlaces, ScalarComparisons, ScalarConstants,
        SharedBorrows, ShortCircuitBooleans, SubtractionInput, Supports, TruncationInput,
        WrappingAddition, WrappingInput, WrappingNegation, WrappingSubtraction,
    },
};
use portable_backend_c::ast::{CPlace, CValue};
use rustc_hir as hir;

impl<'tcx> Reader<'tcx> {
    pub(super) fn expr(&mut self, value: &'tcx hir::Expr<'tcx>) -> Result<CValue> {
        self.ty(self.checked.expr_ty(value))?;
        if let Some(input) = TruncationInput::discover(self.tcx, self.checked, value)? {
            return Supports::<FloatingTruncation>::mapping(&self.mappings).lower(self, input);
        }
        if let Some(input) = AbsoluteInput::discover(self.tcx, self.checked, value)? {
            return Supports::<FloatingAbsolute>::mapping(&self.mappings).lower(self, input);
        }
        if let Some(input) = NaNInput::discover(self.tcx, self.checked, value)? {
            return Supports::<FloatingNaN>::mapping(&self.mappings).lower(self, input);
        }
        if let Some(input) = WrappingInput::discover(self.tcx, self.checked, value)? {
            return Supports::<WrappingNegation>::mapping(&self.mappings).lower(self, input);
        }
        if let Some(input) = AdditionInput::discover(self.tcx, self.checked, value)? {
            return Supports::<WrappingAddition>::mapping(&self.mappings).lower(self, input);
        }
        if let Some(input) = SubtractionInput::discover(self.tcx, self.checked, value)? {
            return Supports::<WrappingSubtraction>::mapping(&self.mappings).lower(self, input);
        }
        if let Some(input) = crate::source_capabilities::MultiplicationInput::discover(
            self.tcx,
            self.checked,
            value,
        )? {
            return Supports::<crate::source_capabilities::WrappingMultiplication>::mapping(
                &self.mappings,
            )
            .lower(self, input);
        }
        if !self.checked.expr_adjustments(value).is_empty() {
            let place = self.place(value)?;
            return c(self.expressions().read(place));
        }
        match value.kind {
            hir::ExprKind::Unary(hir::UnOp::Neg, operand)
                if !matches!(operand.kind, hir::ExprKind::Lit(_))
                    && matches!(
                        self.checked.expr_ty(value).kind(),
                        rustc_middle::ty::Float(rustc_middle::ty::FloatTy::F64)
                    ) =>
            {
                let input = FloatingInput::read(self.tcx, self.checked, value)?;
                Supports::<FloatingNegation>::mapping(&self.mappings).lower(self, input)
            }
            hir::ExprKind::Unary(hir::UnOp::Not, _)
                if !matches!(self.checked.expr_ty(value).kind(), rustc_middle::ty::Bool) =>
            {
                let input = BitwiseInput::read(self.checked, value)?;
                Supports::<IntegerBitwise>::mapping(&self.mappings).lower(self, input)
            }
            hir::ExprKind::Unary(hir::UnOp::Not, _) => {
                let input = NegationInput::read(self.checked, value)?;
                Supports::<BooleanNegation>::mapping(&self.mappings).lower(self, input)
            }
            hir::ExprKind::Call(..) => {
                let mapping = Supports::<DirectCalls>::mapping(&self.mappings);
                mapping.lower(self, CallInput(value))
            }
            hir::ExprKind::Path(ref path)
                if ConstantInput::is_constant(self.checked, value, path) =>
            {
                let input = ConstantInput::read(self.tcx, self.checked, value)?;
                if self.header.is_some()
                    && PublicConstantReadInput::requires_reference(self.tcx, input.definition())
                {
                    let input = PublicConstantReadInput::read(self.tcx, self.checked, value)?;
                    Supports::<PublicConstantReads>::mapping(&self.mappings).lower(self, input)
                } else {
                    Supports::<ScalarConstants>::mapping(&self.mappings).lower(self, input)
                }
            }
            hir::ExprKind::Path(_)
            | hir::ExprKind::Field(..)
            | hir::ExprKind::Unary(hir::UnOp::Deref, _) => {
                let place = self.place(value)?;
                c(self.expressions().read(place))
            }
            hir::ExprKind::Lit(_) | hir::ExprKind::Unary(hir::UnOp::Neg, _) => {
                let mapping = Supports::<LiteralValues>::mapping(&self.mappings);
                let input = LiteralInput::read(self.tcx, self.checked, value)?;
                mapping.lower(self, input)
            }
            hir::ExprKind::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Not, _) => {
                let mapping = Supports::<SharedBorrows>::mapping(&self.mappings);
                mapping.lower(self, BorrowInput(value))
            }
            hir::ExprKind::Binary(operator, ..)
                if operator.node == hir::BinOpKind::Rem
                    && matches!(
                        self.checked.expr_ty(value).kind(),
                        rustc_middle::ty::Float(rustc_middle::ty::FloatTy::F64)
                    ) =>
            {
                let input = RemainderInput::read(self.tcx, self.checked, value)?;
                Supports::<FloatingRemainder>::mapping(&self.mappings).lower(self, input)
            }
            hir::ExprKind::Binary(operator, ..)
                if matches!(
                    operator.node,
                    hir::BinOpKind::Add
                        | hir::BinOpKind::Sub
                        | hir::BinOpKind::Mul
                        | hir::BinOpKind::Div
                ) && matches!(
                    self.checked.expr_ty(value).kind(),
                    rustc_middle::ty::Float(rustc_middle::ty::FloatTy::F64)
                ) =>
            {
                let input = ArithmeticInput::read(self.tcx, self.checked, value)?;
                Supports::<FloatingArithmetic>::mapping(&self.mappings).lower(self, input)
            }
            hir::ExprKind::Binary(operator, ..)
                if matches!(operator.node, hir::BinOpKind::And | hir::BinOpKind::Or) =>
            {
                let input = LazyBooleanInput::read(self.checked, value)?;
                Supports::<ShortCircuitBooleans>::mapping(&self.mappings).lower(self, input)
            }
            hir::ExprKind::Binary(operator, ..)
                if matches!(
                    operator.node,
                    hir::BinOpKind::BitAnd | hir::BinOpKind::BitOr | hir::BinOpKind::BitXor
                ) =>
            {
                if matches!(self.checked.expr_ty(value).kind(), rustc_middle::ty::Bool) {
                    let input = EagerBooleanInput::read(self.checked, value)?;
                    Supports::<EagerBooleans>::mapping(&self.mappings).lower(self, input)
                } else {
                    let input = BitwiseInput::read(self.checked, value)?;
                    Supports::<IntegerBitwise>::mapping(&self.mappings).lower(self, input)
                }
            }
            hir::ExprKind::Binary(..) => {
                let mapping = Supports::<ScalarComparisons>::mapping(&self.mappings);
                mapping.lower(self, ComparisonInput(value))
            }
            _ => Err("C expression mapping is not implemented".into()),
        }
    }

    pub(super) fn place(&mut self, value: &'tcx hir::Expr<'tcx>) -> Result<CPlace> {
        let mapping = Supports::<ResolvedPlaces>::mapping(&self.mappings);
        mapping.lower(self, PlaceInput(value))
    }
}
