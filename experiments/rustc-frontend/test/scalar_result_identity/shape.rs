//! Observation-only compiler identities. No target certificate can be issued here.
use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::symbol::sym;

pub(super) struct ResultShape<'tcx> {
    value: Ty<'tcx>,
    ok: DefId,
    err: DefId,
}

impl<'tcx> ResultShape<'tcx> {
    pub(super) fn observe(tcx: TyCtxt<'tcx>, value: Ty<'tcx>) -> Result<Self, String> {
        let core = tcx
            .lang_items()
            .copy_trait()
            .filter(|item| !item.is_local())
            .ok_or("missing external core anchor")?
            .krate;
        let standard = tcx
            .get_diagnostic_item(sym::Result)
            .filter(|id| !id.is_local() && id.krate == core)
            .ok_or("missing standard Result identity")?;
        let ty::Adt(definition, arguments) = value.kind() else {
            return Err("expected standard scalar Result".into());
        };
        if definition.did() != standard || !definition.is_enum() {
            return Err("expected standard scalar Result".into());
        }
        if arguments.len() != 2 || arguments.type_at(0) != tcx.types.i32 {
            return Err("result success payload must be exact i32".into());
        }
        let conversion = tcx
            .get_diagnostic_item(sym::TryFrom)
            .filter(|id| !id.is_local() && id.krate == core)
            .ok_or("missing standard TryFrom identity")?;
        let error_items: Vec<_> = tcx
            .associated_items(conversion)
            .in_definition_order()
            .filter(|item| tcx.def_kind(item.def_id) == DefKind::AssocTy)
            .collect();
        if error_items.len() != 1 {
            return Err("standard TryFrom associated type inventory changed".into());
        }
        let args = tcx.mk_args(&[tcx.types.i32.into(), tcx.types.i64.into()]);
        // A trait associated declaration has no type_of value. Its applied
        // projection is explicitly unnormalized and must be resolved by rustc.
        let projection = Ty::new_projection(tcx, ty::IsRigid::No, error_items[0].def_id, args);
        let error = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                ty::Unnormalized::new_wip(projection),
            )
            .map_err(|_| "standard narrowing error projection failed")?;
        let ty::Adt(error_definition, error_arguments) = error.kind() else {
            return Err("standard narrowing error is not nominal".into());
        };
        if error_definition.did().is_local()
            || error_definition.did().krate != core
            || !error_arguments.is_empty()
            || arguments.type_at(1) != error
        {
            return Err("result error payload is not the standard narrowing error".into());
        }
        let ok = tcx
            .lang_items()
            .result_ok_variant()
            .ok_or("missing standard Ok variant")?;
        let err = tcx
            .lang_items()
            .result_err_variant()
            .ok_or("missing standard Err variant")?;
        if definition.variants().len() != 2 || ok == err {
            return Err("standard Result variant inventory changed".into());
        }
        for (variant_id, payload) in [(ok, tcx.types.i32), (err, error)] {
            if tcx.parent(variant_id) != standard {
                return Err("result variant owner differs".into());
            }
            let variant = definition
                .variants()
                .iter()
                .find(|variant| variant.def_id == variant_id)
                .ok_or("standard Result variant missing")?;
            if variant.fields.len() != 1 {
                return Err("result variant payload differs".into());
            }
            let field_type = tcx
                .try_normalize_erasing_regions(
                    ty::TypingEnv::fully_monomorphized(),
                    variant.fields.iter().next().unwrap().ty(tcx, arguments),
                )
                .map_err(|_| "result payload normalization failed")?;
            if field_type != payload {
                return Err("result variant payload differs".into());
            }
        }
        if value.needs_drop(tcx, ty::TypingEnv::fully_monomorphized()) {
            return Err("scalar result cannot require drop".into());
        }
        Ok(Self { value, ok, err })
    }

    pub(super) fn same_instance(&self, other: &Self) -> bool {
        self.value == other.value && self.ok == other.ok && self.err == other.err
    }

    pub(super) fn describe(&self, tcx: TyCtxt<'tcx>) -> String {
        let ty::Adt(definition, arguments) = self.value.kind() else {
            unreachable!("observed Result")
        };
        let ty::Adt(error, _) = arguments.type_at(1).kind() else {
            unreachable!("observed standard error")
        };
        format!(
            "RESULT {:?} I32 {:?} {:?} {:?} NO_DROP",
            tcx.def_path_hash(definition.did()),
            tcx.def_path_hash(error.did()),
            tcx.def_path_hash(self.ok),
            tcx.def_path_hash(self.err)
        )
    }
}
