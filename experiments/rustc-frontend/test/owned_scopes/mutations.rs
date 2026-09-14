//! Deliberately incorrect scope claims go through the production certifier.
use super::{
    LinearError, relations,
    scopes::{self, Claims},
    source,
};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{Place, TerminatorKind},
    ty::TyCtxt,
};

pub(crate) fn check(tcx: TyCtxt<'_>, owner: LocalDefId) {
    let plan = source::read_tail_scopes(tcx, owner).unwrap();
    let fresh = || Claims {
        blocks: plan.scopes.blocks().collect(),
        bindings: plan.scopes.bindings().to_vec(),
        read: plan.scopes.read_scope(),
        drop: plan.scopes.drop_scope(),
    };
    assert_eq!(fresh().blocks.len(), 4);
    assert_eq!(fresh().bindings.len(), 3);
    assert_ne!(fresh().read, fresh().drop);
    assert!(scopes::certify(tcx, owner, fresh()).is_ok());
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut Claims)| {
        let mut changed = fresh();
        edit(&mut changed);
        assert!(
            matches!(
                scopes::certify(tcx, owner, changed),
                Err(LinearError::Scope)
            ),
            "scope mutation admitted: {name}"
        );
        cases += 1;
    };
    reject("wrong parent", &|c| c.blocks[2].1 = None);
    reject("missing scope", &|c| {
        c.blocks.remove(1);
    });
    reject("duplicate scope", &|c| c.blocks.push(c.blocks[3]));
    reject("substituted scope", &|c| c.blocks.swap(0, 1));
    reject("wrong binding scope", &|c| c.bindings[0].1 = c.blocks[2].0);
    reject("wrong binding identity", &|c| {
        c.bindings[0].0 = c.bindings[1].0;
    });
    reject("wrong read scope", &|c| c.read = c.blocks[0].0);
    reject("read scope is not drop scope", &|c| c.drop = c.read);
    reject("missing binding", &|c| {
        c.bindings.pop();
    });
    reject("duplicate binding", &|c| c.bindings.push(c.bindings[0]));
    let other = tcx
        .hir_body_owners()
        .find(|id| tcx.def_path_str(*id) == "root")
        .unwrap();
    assert!(matches!(
        scopes::certify(tcx, other, fresh()),
        Err(LinearError::Scope)
    ));
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let matched = relations::validate(tcx, &plan, &original).unwrap();
    let mut changed = original.clone();
    let extra = changed.basic_blocks[matched.drop.block]
        .terminator()
        .kind
        .clone();
    let mut inserted = false;
    for block in changed.basic_blocks.as_mut().iter_mut() {
        if matches!(block.terminator().kind, TerminatorKind::Return) {
            block.terminator_mut().kind = extra.clone();
            let TerminatorKind::Drop { place, .. } = &mut block.terminator_mut().kind else {
                unreachable!()
            };
            *place = Place::from(matched.owners[0]);
            inserted = true;
            break;
        }
    }
    assert!(inserted);
    assert!(
        relations::validate(tcx, &plan, &changed).is_err(),
        "extra moved outer-owner cleanup admitted"
    );
    assert_eq!(cases, 10);
    println!("12 scope and outer-cleanup mutations rejected");
}
