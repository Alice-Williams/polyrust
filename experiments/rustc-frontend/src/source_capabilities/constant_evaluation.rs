//! Shared compiler definition admission/evaluation for reads and declarations.
use super::ScalarConstantValue;
use portable_binary64::FiniteBinary64;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self, Ty, TyCtxt};

pub(super) fn evaluate<'tcx>(
    tcx: TyCtxt<'tcx>,
    definition: DefId,
) -> Result<(Ty<'tcx>, ScalarConstantValue), String> {
    let kind = tcx.def_kind(definition);
    if !matches!(
        kind,
        DefKind::Const {
            is_type_const: false
        } | DefKind::AssocConst {
            is_type_const: false
        }
    ) || tcx.generics_of(definition).count() != 0
    {
        return Err("scalar constants require nongeneric constant definitions".into());
    }
    if matches!(kind, DefKind::AssocConst { .. }) {
        check_inherent_owner(tcx, definition)?;
    }
    let ty = tcx
        .try_normalize_erasing_regions(
            ty::TypingEnv::fully_monomorphized(),
            tcx.type_of(definition).instantiate_identity(),
        )
        .map_err(|_| "scalar constant type normalization failed")?;
    if !matches!(
        ty.kind(),
        ty::Bool | ty::Int(ty::IntTy::I32 | ty::IntTy::I64) | ty::Float(ty::FloatTy::F64)
    ) {
        return Err("scalar constants support only bool, i32, i64 and finite f64".into());
    }
    let evaluated = tcx
        .const_eval_poly(definition)
        .map_err(|_| "compiler could not evaluate scalar constant")?;
    let scalar = evaluated
        .try_to_scalar_int()
        .ok_or("compiler constant is not scalar bits")?;
    // Every decoder asserts its input width; check both type and width first.
    let value = match (ty.kind(), scalar.size().bytes()) {
        (ty::Bool, 1) => ScalarConstantValue::Bool(
            scalar
                .try_to_bool()
                .map_err(|_| "invalid compiler Boolean scalar")?,
        ),
        (ty::Int(ty::IntTy::I32), 4) => ScalarConstantValue::I32(scalar.to_i32()),
        (ty::Int(ty::IntTy::I64), 8) => ScalarConstantValue::I64(scalar.to_i64()),
        (ty::Float(ty::FloatTy::F64), 8) => ScalarConstantValue::F64(
            FiniteBinary64::from_bits(scalar.to_u64())
                .map_err(|_| "nonfinite f64 constants are not implemented")?,
        ),
        _ => return Err("compiler constant scalar width disagrees with its type".into()),
    };
    Ok((ty, value))
}

/// A concrete impl can have no parameters but still instantiate generic self
/// arguments, including omitted defaults. Definition/node_args checks miss it.
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
