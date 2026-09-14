//! Shared, bounded source-surface admission before normalized type lowering.
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    intravisit::{self, Visitor},
};
use rustc_middle::ty::TyCtxt;
use std::ops::ControlFlow;

type Flow = ControlFlow<&'static str>;

/// Unused alias declarations are legal; uses need an explicit provenance mapping.
/// ControlFlow stops the compiler's outer item loop as well as recursive walks.
pub(crate) fn aliases(tcx: TyCtxt<'_>) -> Result<(), String> {
    let mut visitor = AliasUses {
        tcx,
        remaining: 100_000,
        depth: 0,
    };
    match tcx.hir_visit_all_item_likes_in_crate(&mut visitor) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(message) => Err(message.into()),
    }
}

struct AliasUses<'tcx> {
    tcx: TyCtxt<'tcx>,
    remaining: usize,
    depth: usize,
}

impl AliasUses<'_> {
    fn nested_reference(&mut self) -> Flow {
        self.charge()
            .map_break(|_| "Rust source admission nested-reference budget exceeded")
    }

    fn charge(&mut self) -> Flow {
        if self.remaining == 0 {
            return ControlFlow::Break("Rust source admission visit budget exceeded");
        }
        self.remaining -= 1;
        ControlFlow::Continue(())
    }

    fn bounded(&mut self, walk: impl FnOnce(&mut Self) -> Flow) -> Flow {
        self.charge()?;
        if self.depth >= 128 {
            return ControlFlow::Break("Rust source admission depth budget exceeded");
        }
        self.depth += 1;
        let result = walk(self);
        self.depth -= 1;
        result
    }
}

// Guard recursive categories and repeated list elements. Compiler walk helpers
// propagate ControlFlow, so rejected input cannot continue scanning sibling nodes.
macro_rules! bounded_visit {
    ($visit:ident, $walk:ident, $ty:ty) => {
        fn $visit(&mut self, value: $ty) -> Flow {
            self.bounded(|visitor| intravisit::$walk(visitor, value))
        }
    };
}

impl<'tcx> Visitor<'tcx> for AliasUses<'tcx> {
    type NestedFilter = rustc_middle::hir::nested_filter::OnlyBodies;
    type Result = Flow;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn visit_id(&mut self, _: hir::HirId) -> Flow {
        self.charge()
    }
    fn visit_name(&mut self, _: rustc_span::Symbol) -> Flow {
        self.charge()
    }

    // OnlyBodies intentionally does not descend through these references: the
    // compiler's outer item-like iterator visits their definitions separately.
    // Still charge each list element, and propagate exhaustion to its caller.
    fn visit_nested_item(&mut self, _: hir::ItemId) -> Flow {
        self.nested_reference()
    }
    fn visit_nested_trait_item(&mut self, _: hir::TraitItemId) -> Flow {
        self.nested_reference()
    }
    fn visit_nested_impl_item(&mut self, _: hir::ImplItemId) -> Flow {
        self.nested_reference()
    }
    fn visit_nested_foreign_item(&mut self, _: hir::ForeignItemId) -> Flow {
        self.nested_reference()
    }

    bounded_visit!(visit_item, walk_item, &'tcx hir::Item<'tcx>);
    bounded_visit!(
        visit_trait_item,
        walk_trait_item,
        &'tcx hir::TraitItem<'tcx>
    );
    bounded_visit!(visit_impl_item, walk_impl_item, &'tcx hir::ImplItem<'tcx>);
    bounded_visit!(
        visit_foreign_item,
        walk_foreign_item,
        &'tcx hir::ForeignItem<'tcx>
    );
    bounded_visit!(visit_expr, walk_expr, &'tcx hir::Expr<'tcx>);
    bounded_visit!(visit_pat, walk_pat, &'tcx hir::Pat<'tcx>);
    bounded_visit!(visit_block, walk_block, &'tcx hir::Block<'tcx>);
    bounded_visit!(visit_stmt, walk_stmt, &'tcx hir::Stmt<'tcx>);
    bounded_visit!(
        visit_generic_args,
        walk_generic_args,
        &'tcx hir::GenericArgs<'tcx>
    );
    bounded_visit!(
        visit_generic_arg,
        walk_generic_arg,
        &'tcx hir::GenericArg<'tcx>
    );
    bounded_visit!(
        visit_generic_param,
        walk_generic_param,
        &'tcx hir::GenericParam<'tcx>
    );
    bounded_visit!(
        visit_param_bound,
        walk_param_bound,
        &'tcx hir::GenericBound<'tcx>
    );
    bounded_visit!(
        visit_poly_trait_ref,
        walk_poly_trait_ref,
        &'tcx hir::PolyTraitRef<'tcx>
    );
    bounded_visit!(
        visit_assoc_item_constraint,
        walk_assoc_item_constraint,
        &'tcx hir::AssocItemConstraint<'tcx>
    );
    bounded_visit!(visit_variant, walk_variant, &'tcx hir::Variant<'tcx>);
    bounded_visit!(visit_field_def, walk_field_def, &'tcx hir::FieldDef<'tcx>);
    bounded_visit!(visit_arm, walk_arm, &'tcx hir::Arm<'tcx>);
    bounded_visit!(
        visit_const_arg,
        walk_const_arg,
        &'tcx hir::ConstArg<'tcx, hir::AmbigArg>
    );
    bounded_visit!(
        visit_pattern_type_pattern,
        walk_ty_pat,
        &'tcx hir::TyPat<'tcx>
    );

    fn visit_ty(&mut self, value: &'tcx hir::Ty<'tcx, hir::AmbigArg>) -> Flow {
        self.bounded(|visitor| {
            if let hir::TyKind::Path(path) = &value.kind {
                let alias = match path {
                    hir::QPath::Resolved(_, path) => {
                        matches!(path.res, Res::Def(DefKind::TyAlias | DefKind::AssocTy, _))
                    }
                    hir::QPath::TypeRelative(..) => true,
                };
                if alias {
                    return ControlFlow::Break(
                        "Rust type alias uses require an unimplemented provenance mapping",
                    );
                }
            }
            intravisit::walk_ty(visitor, value)
        })
    }

    fn visit_path(&mut self, path: &hir::Path<'tcx>, _: hir::HirId) -> Flow {
        self.bounded(|visitor| {
            if matches!(path.res, Res::Def(DefKind::TyAlias | DefKind::AssocTy, _)) {
                return ControlFlow::Break(
                    "Rust type alias uses require an unimplemented provenance mapping",
                );
            }
            intravisit::walk_path(visitor, path)
        })
    }
}
