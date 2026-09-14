//! A complete local-number bijection preserves semantics, unlike substitution.
use super::{relations, source};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{
        self,
        visit::{MutVisitor, PlaceContext},
    },
    ty::TyCtxt,
};

struct Rename<'tcx> {
    tcx: TyCtxt<'tcx>,
    first: mir::Local,
    second: mir::Local,
}
impl<'tcx> MutVisitor<'tcx> for Rename<'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn visit_local(&mut self, local: &mut mir::Local, _: PlaceContext, _: mir::Location) {
        if *local == self.first {
            *local = self.second;
        } else if *local == self.second {
            *local = self.first;
        }
    }
}

pub(crate) fn check(tcx: TyCtxt<'_>, owner: LocalDefId) {
    let plan = source::read(tcx, owner).unwrap();
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let before = relations::validate(tcx, owner, &plan, &original).unwrap();
    assert_eq!(before.chains.len(), 2);
    assert_eq!(before.chains[0].owners.len(), 1);
    assert_eq!(before.chains[1].owners.len(), 1);
    let first = before.chains[0].owners[0];
    let second = before.chains[1].owners[0];
    assert_ne!(first, second);
    assert_eq!(
        original.local_decls[first].ty,
        original.local_decls[second].ty
    );
    let mut changed = original.clone();
    Rename { tcx, first, second }.visit_body(&mut changed);
    // Rename the declaration table as well as all code/storage/debug uses.
    changed.local_decls[first] = original.local_decls[second].clone();
    changed.local_decls[second] = original.local_decls[first].clone();
    let after = relations::validate(tcx, owner, &plan, &changed).unwrap();
    assert_eq!(after.chains[0].owners, vec![second]);
    assert_eq!(after.chains[1].owners, vec![first]);
    assert_eq!(after.chains[0].parameter, before.chains[0].parameter);
    assert_eq!(after.chains[1].parameter, before.chains[1].parameter);
    assert_eq!(after.drops, before.drops);
    assert_eq!(after.read, before.read);
    assert_eq!(after.chains[0].drop, before.chains[0].drop);
    assert_eq!(after.chains[1].drop, before.chains[1].drop);
    println!("bijective owner-local renaming preserves the authenticated relation");
}
