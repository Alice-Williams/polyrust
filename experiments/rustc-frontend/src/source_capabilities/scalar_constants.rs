//! Compiler-evaluated scalar reads; no initializer interpreter or target syntax.
use super::{Capability, LiteralValue};
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::DefId,
};
use rustc_middle::ty::{self, TyCtxt, TypeckResults};

pub(crate) struct ScalarConstants;

/// Construction retains compiler provenance and is confined to this module.
#[derive(Clone, Copy)]
pub(crate) struct ConstantInput<'tcx> {
    value: LiteralValue,
    _definition: DefId,
    _expression: &'tcx hir::Expr<'tcx>,
}

impl Capability for ScalarConstants {
    type Input<'tcx> = ConstantInput<'tcx>;
}

impl<'tcx> ConstantInput<'tcx> {
    pub(crate) fn is_constant(
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
        path: &hir::QPath<'tcx>,
    ) -> bool {
        matches!(
            checked.qpath_res(path, expression.hir_id),
            Res::Def(DefKind::Const { .. } | DefKind::AssocConst { .. }, _)
        )
    }

    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::ExprKind::Path(ref path) = expression.kind else {
            return Err("scalar constant input requires a resolved path".into());
        };
        let Res::Def(
            kind @ (DefKind::Const {
                is_type_const: false,
            }
            | DefKind::AssocConst {
                is_type_const: false,
            }),
            definition,
        ) = checked.qpath_res(path, expression.hir_id)
        else {
            return Err("scalar constant input requires a constant definition".into());
        };
        if !checked.expr_adjustments(expression).is_empty()
            || !checked.node_args(expression.hir_id).is_empty()
            || tcx.generics_of(definition).count() != 0
        {
            return Err(
                "scalar constants require nongeneric unadjusted module or inherent definitions"
                    .into(),
            );
        }
        if matches!(kind, DefKind::AssocConst { .. }) {
            check_inherent_owner(tcx, definition)?;
        }
        let ty = checked.expr_ty(expression);
        let definition_ty = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.type_of(definition).instantiate_identity(),
            )
            .map_err(|_| "scalar constant type normalization failed")?;
        if definition_ty != ty {
            return Err("scalar constant definition and expression types disagree".into());
        }
        if !matches!(
            ty.kind(),
            ty::Bool | ty::Int(ty::IntTy::I32 | ty::IntTy::I64)
        ) {
            return Err("scalar constants support only bool, i32 and i64".into());
        }
        let evaluated = tcx
            .const_eval_poly(definition)
            .map_err(|_| "compiler could not evaluate scalar constant")?;
        let scalar = evaluated
            .try_to_scalar_int()
            .ok_or("compiler constant is not a scalar integer")?;
        // rustc's signed decoders assert their input width. Check before calling.
        let value = match (ty.kind(), scalar.size().bytes()) {
            (ty::Bool, 1) => LiteralValue::Bool(
                scalar
                    .try_to_bool()
                    .map_err(|_| "invalid compiler Boolean scalar")?,
            ),
            (ty::Int(ty::IntTy::I32), 4) => LiteralValue::I32(scalar.to_i32()),
            (ty::Int(ty::IntTy::I64), 8) => LiteralValue::I64(scalar.to_i64()),
            _ => return Err("compiler constant scalar width disagrees with its type".into()),
        };
        Ok(Self {
            value,
            _definition: definition,
            _expression: expression,
        })
    }

    pub(crate) fn value(self) -> LiteralValue {
        self.value
    }

    #[cfg(constant_ast_probe)]
    pub(crate) fn origin(self) -> (DefId, &'tcx hir::Expr<'tcx>) {
        (self._definition, self._expression)
    }
}

/// A concrete impl can have zero parameters while its self type still has
/// instantiated generic arguments. Those include omitted/defaulted arguments;
/// neither expression node_args nor the constant's own generics exposes them.
fn check_inherent_owner(tcx: TyCtxt<'_>, definition: DefId) -> Result<(), String> {
    let implementation = tcx
        .inherent_impl_of_assoc(definition)
        .ok_or("scalar constants require nongeneric unadjusted module or inherent definitions")?;
    let owner = tcx
        .try_normalize_erasing_regions(
            ty::TypingEnv::fully_monomorphized(),
            tcx.type_of(implementation).instantiate_identity(),
        )
        .map_err(|_| "scalar constant inherent owner normalization failed")?;
    match owner.kind() {
        ty::Adt(_, arguments) if arguments.is_empty() => Ok(()),
        ty::Bool | ty::Char | ty::Int(_) | ty::Uint(_) | ty::Float(_) | ty::Str => Ok(()),
        _ => Err("scalar constants require nongeneric nominal or primitive inherent owners".into()),
    }
}
