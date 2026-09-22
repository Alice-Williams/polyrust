//! Traversal dispatches to typed mappings and bounds recursive source work.
use super::capabilities::*;
use super::{Place, Reader, Result, TypePlan, Value, name};
use portable_backend_java::ast::{JavaExpr, JavaLocalFinality, JavaStmt};
use rustc_hir as hir;
use rustc_middle::ty::Ty;

impl<'tcx> Reader<'tcx> {
    fn bounded<T>(&mut self, work: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        if self.depth >= 128 || self.remaining == 0 {
            return Err("Java source lowering traversal budget exceeded".into());
        }
        self.depth += 1;
        self.remaining -= 1;
        let result = work(self);
        self.depth -= 1;
        result
    }

    pub(super) fn ty(&mut self, ty: Ty<'tcx>) -> Result<TypePlan> {
        self.bounded(|reader| {
            Supports::<ObjectTypes>::mapping(&reader.mappings).lower(reader, TypeInput(ty))
        })
    }

    pub(super) fn expr(&mut self, value: &'tcx hir::Expr<'tcx>) -> Result<Value> {
        self.bounded(|reader| {
            reader.ty(reader.checked.expr_ty(value))?;
            let result = if let Some(input) =
                TruncationInput::discover(reader.tcx, reader.checked, value)?
            {
                Supports::<FloatingTruncation>::mapping(&reader.mappings).lower(reader, input)?
            } else if let Some(input) = AbsoluteInput::discover(reader.tcx, reader.checked, value)?
            {
                Supports::<FloatingAbsolute>::mapping(&reader.mappings).lower(reader, input)?
            } else if let Some(input) = NaNInput::discover(reader.tcx, reader.checked, value)? {
                Supports::<FloatingNaN>::mapping(&reader.mappings).lower(reader, input)?
            } else if let Some(input) = WrappingInput::discover(reader.tcx, reader.checked, value)?
            {
                Supports::<WrappingNegation>::mapping(&reader.mappings).lower(reader, input)?
            } else if let Some(input) = AdditionInput::discover(reader.tcx, reader.checked, value)?
            {
                Supports::<WrappingAddition>::mapping(&reader.mappings).lower(reader, input)?
            } else if let Some(input) =
                SubtractionInput::discover(reader.tcx, reader.checked, value)?
            {
                Supports::<WrappingSubtraction>::mapping(&reader.mappings).lower(reader, input)?
            } else if !reader.checked.expr_adjustments(value).is_empty() {
                reader.place(value)?.value()
            } else {
                match value.kind {
                    hir::ExprKind::Unary(hir::UnOp::Neg, operand)
                        if !matches!(operand.kind, hir::ExprKind::Lit(_))
                            && matches!(
                                reader.checked.expr_ty(value).kind(),
                                rustc_middle::ty::Float(rustc_middle::ty::FloatTy::F64)
                            ) =>
                    {
                        let input = FloatingInput::read(reader.tcx, reader.checked, value)?;
                        Supports::<FloatingNegation>::mapping(&reader.mappings).lower(reader, input)
                    }
                    hir::ExprKind::Unary(hir::UnOp::Not, _)
                        if !matches!(
                            reader.checked.expr_ty(value).kind(),
                            rustc_middle::ty::Bool
                        ) =>
                    {
                        let input = BitwiseInput::read(reader.checked, value)?;
                        Supports::<IntegerBitwise>::mapping(&reader.mappings).lower(reader, input)
                    }
                    hir::ExprKind::Unary(hir::UnOp::Not, _) => {
                        let input = NegationInput::read(reader.checked, value)?;
                        Supports::<BooleanNegation>::mapping(&reader.mappings).lower(reader, input)
                    }
                    hir::ExprKind::Path(ref path)
                        if ConstantInput::is_constant(reader.checked, value, path) =>
                    {
                        let input = ConstantInput::read(reader.tcx, reader.checked, value)?;
                        if reader.public_api
                            && PublicConstantReadInput::requires_reference(
                                reader.tcx,
                                input.definition(),
                            )
                        {
                            let input =
                                PublicConstantReadInput::read(reader.tcx, reader.checked, value)?;
                            Supports::<PublicConstantReads>::mapping(&reader.mappings)
                                .lower(reader, input)
                        } else {
                            Supports::<ScalarConstants>::mapping(&reader.mappings)
                                .lower(reader, input)
                        }
                    }
                    hir::ExprKind::Path(_)
                    | hir::ExprKind::Field(..)
                    | hir::ExprKind::Unary(hir::UnOp::Deref, _) => Ok(reader.place(value)?.value()),
                    hir::ExprKind::Call(..) => Supports::<DirectCalls>::mapping(&reader.mappings)
                        .lower(reader, CallInput(value)),
                    hir::ExprKind::Lit(_) | hir::ExprKind::Unary(hir::UnOp::Neg, _) => {
                        let input = LiteralInput::read(reader.tcx, reader.checked, value)?;
                        Supports::<LiteralValues>::mapping(&reader.mappings).lower(reader, input)
                    }
                    hir::ExprKind::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Not, _) => {
                        Supports::<SharedBorrows>::mapping(&reader.mappings)
                            .lower(reader, BorrowInput(value))
                    }
                    hir::ExprKind::Binary(operator, ..)
                        if operator.node == hir::BinOpKind::Rem
                            && matches!(
                                reader.checked.expr_ty(value).kind(),
                                rustc_middle::ty::Float(rustc_middle::ty::FloatTy::F64)
                            ) =>
                    {
                        let input = RemainderInput::read(reader.tcx, reader.checked, value)?;
                        Supports::<FloatingRemainder>::mapping(&reader.mappings)
                            .lower(reader, input)
                    }
                    hir::ExprKind::Binary(operator, ..)
                        if matches!(
                            operator.node,
                            hir::BinOpKind::Add
                                | hir::BinOpKind::Sub
                                | hir::BinOpKind::Mul
                                | hir::BinOpKind::Div
                        ) && matches!(
                            reader.checked.expr_ty(value).kind(),
                            rustc_middle::ty::Float(rustc_middle::ty::FloatTy::F64)
                        ) =>
                    {
                        let input = ArithmeticInput::read(reader.tcx, reader.checked, value)?;
                        Supports::<FloatingArithmetic>::mapping(&reader.mappings)
                            .lower(reader, input)
                    }
                    hir::ExprKind::Binary(operator, ..)
                        if matches!(operator.node, hir::BinOpKind::And | hir::BinOpKind::Or) =>
                    {
                        let input = LazyBooleanInput::read(reader.checked, value)?;
                        Supports::<ShortCircuitBooleans>::mapping(&reader.mappings)
                            .lower(reader, input)
                    }
                    hir::ExprKind::Binary(operator, ..)
                        if matches!(
                            operator.node,
                            hir::BinOpKind::BitAnd | hir::BinOpKind::BitOr | hir::BinOpKind::BitXor
                        ) =>
                    {
                        if matches!(reader.checked.expr_ty(value).kind(), rustc_middle::ty::Bool) {
                            let input = EagerBooleanInput::read(reader.checked, value)?;
                            Supports::<EagerBooleans>::mapping(&reader.mappings)
                                .lower(reader, input)
                        } else {
                            let input = BitwiseInput::read(reader.checked, value)?;
                            Supports::<IntegerBitwise>::mapping(&reader.mappings)
                                .lower(reader, input)
                        }
                    }
                    hir::ExprKind::Binary(..) => {
                        Supports::<ScalarComparisons>::mapping(&reader.mappings)
                            .lower(reader, ComparisonInput(value))
                    }
                    _ => Err("Java expression mapping is not implemented".into()),
                }?
            };
            #[cfg(java_ast_probe)]
            reader.expression_observations.push((
                value.hir_id,
                reader.prelude.len(),
                result.clone(),
            ));
            Ok(result)
        })
    }

    pub(super) fn place(&mut self, value: &'tcx hir::Expr<'tcx>) -> Result<Place> {
        self.bounded(|reader| {
            Supports::<ResolvedPlaces>::mapping(&reader.mappings).lower(reader, PlaceInput(value))
        })
    }

    pub(super) fn initializer(&mut self, value: &'tcx hir::Expr<'tcx>) -> Result<Value> {
        if matches!(value.kind, hir::ExprKind::Struct(..)) {
            self.bounded(|reader| {
                Supports::<RecordInitializers>::mapping(&reader.mappings)
                    .lower(reader, RecordInput(value))
            })
        } else {
            self.expr(value)
        }
    }

    pub(super) fn branch(
        &mut self,
        expression: &'tcx hir::Expr<'tcx>,
        parent: Option<hir::HirId>,
    ) -> Result<portable_backend_java::ast::JavaBlock> {
        self.control(expression, parent, ControlCompletion::Return)
    }

    pub(super) fn effect_branch(
        &mut self,
        expression: &'tcx hir::Expr<'tcx>,
        parent: hir::HirId,
    ) -> Result<portable_backend_java::ast::JavaBlock> {
        self.control(expression, Some(parent), ControlCompletion::Effect)
    }

    pub(super) fn unit(
        &mut self,
        expression: &'tcx hir::Expr<'tcx>,
        scope: hir::HirId,
    ) -> Result<Vec<JavaStmt>> {
        self.bounded(|reader| {
            let input = UnitInput::read(reader.checked, expression, scope)?;
            Supports::<UnitEffects>::mapping(&reader.mappings).lower(reader, input)
        })
    }

    fn control(
        &mut self,
        expression: &'tcx hir::Expr<'tcx>,
        parent: Option<hir::HirId>,
        completion: ControlCompletion,
    ) -> Result<portable_backend_java::ast::JavaBlock> {
        self.bounded(|reader| {
            Supports::<LexicalControl>::mapping(&reader.mappings).lower(
                reader,
                ControlInput {
                    expression,
                    parent,
                    completion,
                },
            )
        })
    }

    pub(super) fn fresh(&mut self) -> Result<portable_backend_java::ast::JavaIdentifier> {
        if self.next_local >= 100_000 {
            return Err("Java source local budget exceeded".into());
        }
        let result = name(&format!("v{}", self.next_local))?;
        self.next_local += 1;
        Ok(result)
    }

    pub(super) fn materialize(&mut self, value: Value) -> Result<Value> {
        if self.active_scope.is_none() {
            return Err("evaluation requires an active source scope".into());
        }
        let plan = value.plan().clone();
        let name = self.fresh()?;
        self.prelude.push(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: plan.java_type(),
            name: name.clone(),
            value: Some(value.into_expression()),
        });
        Value::new(plan.clone(), JavaExpr::local(plan.java_type(), name))
    }
}
