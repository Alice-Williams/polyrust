//! Exercise historical entry points and executable capability consumption.
use crate::{
    owned_linear::{LinearOwnedBody, multiple::MultipleOwnedBody},
    owned_source::{BoxConstructionInput, Builder, OwnedBoxConstruction},
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_middle::ty::TyCtxt;

#[derive(Clone, Copy)]
struct Observe;
impl Mapping for Observe {
    type Capability = OwnedBoxConstruction;
    type Context<'tcx> = TyCtxt<'tcx>;
    type Output = DefId;
    fn lower<'tcx>(
        &self,
        tcx: &mut TyCtxt<'tcx>,
        input: BoxConstructionInput<'tcx>,
    ) -> Result<DefId, String> {
        assert_eq!(input.call().hir_id.owner.def_id, input.owner());
        assert_eq!(
            tcx.typeck(input.owner()).expr_ty(input.argument()),
            tcx.types.i32
        );
        assert_eq!(
            tcx.typeck(input.owner()).expr_ty(input.call()),
            input.result()
        );
        Ok(input.constructor())
    }
}
pub(super) fn mapping<'tcx>(tcx: TyCtxt<'tcx>, body: MultipleOwnedBody<'tcx>) {
    let owner = body.chains()[0].parameter().0.owner.def_id;
    crate::exit_consumer::check(
        tcx,
        owner,
        body.scopes().read_scope(),
        body.exit(),
        body.returning(),
    );
    chains(tcx, body.into_chains());
}
pub(super) fn chains<'tcx>(
    mut tcx: TyCtxt<'tcx>,
    chains: Vec<crate::owned_linear::multiple::ChainEvidence<'tcx>>,
) {
    let binding = Builder::new().construction(Observe).build();
    for chain in chains {
        assert_eq!(
            binding
                .mapping()
                .lower(&mut tcx, chain.into_construction())
                .unwrap(),
            tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap()
        );
    }
}
pub(super) fn tail(tcx: TyCtxt<'_>, owner: LocalDefId) {
    let root = LinearOwnedBody::read(tcx, owner).unwrap();
    let nested = LinearOwnedBody::read_tail_scopes(tcx, owner).unwrap();
    let multiple = MultipleOwnedBody::read(tcx, owner).unwrap();
    let chain = &multiple.chains()[0];
    crate::exit_consumer::check(
        tcx,
        owner,
        root.scopes().read_scope(),
        root.exit(),
        root.returning(),
    );
    crate::exit_consumer::check(
        tcx,
        owner,
        nested.scopes().read_scope(),
        nested.exit(),
        nested.returning(),
    );
    assert_eq!(root.parameter(), nested.parameter());
    assert_eq!(root.parameter(), chain.parameter());
    assert_eq!(root.bindings(), chain.bindings());
    assert_eq!(root.moves(), chain.moves());
    assert_eq!(root.scalar_read(), multiple.scalar_read().1);
    assert_eq!(root.drop_location(), chain.drop_location());
    assert_eq!(root.scope(), nested.scope());
    assert_eq!(
        root.scopes().blocks().collect::<Vec<_>>(),
        multiple.scopes().blocks().collect::<Vec<_>>()
    );
    assert_eq!(root.scopes().bindings(), multiple.scopes().bindings());
    assert_eq!(root.scopes().read_scope(), multiple.scopes().read_scope());
    assert_eq!(root.scopes().drop_scope(), chain.drop_scope());
    assert_eq!(
        root.into_construction().constructor(),
        tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap()
    );
    mapping(tcx, multiple);
}
