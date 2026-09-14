//! Invocation-scoped access to exact source-authenticated owning results.
use rustc_hir::def_id::CrateNum;
use rustc_middle::ty::TyCtxt;
use std::{collections::BTreeMap, marker::PhantomData, ops::Deref};

/// Only the source checker can construct this view, after exact artifact joins.
/// The compiler lifetime is invariant: the view cannot outlive its invocation.
/// Callers must retain owned stable identities, not copied transient CrateNums.
pub(crate) struct CheckedDependencies<'a, 'tcx, T> {
    results: BTreeMap<CrateNum, &'a T>,
    context: PhantomData<fn(TyCtxt<'tcx>) -> TyCtxt<'tcx>>,
}

impl<'a, 'tcx, T> CheckedDependencies<'a, 'tcx, T> {
    pub(super) fn new(
        _: TyCtxt<'tcx>,
        loaded: BTreeMap<CrateNum, String>,
        results: &'a BTreeMap<String, &'a T>,
    ) -> Result<Self, String> {
        let bound = loaded
            .into_iter()
            .map(|(krate, key)| {
                let result = results
                    .get(&key)
                    .ok_or("authenticated dependency result missing")?;
                Ok((krate, *result))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        if bound.len() != results.len() {
            return Err("authenticated dependency result inventory differs".into());
        }
        Ok(Self {
            results: bound,
            context: PhantomData,
        })
    }
}

impl<'a, T> Deref for CheckedDependencies<'a, '_, T> {
    type Target = BTreeMap<CrateNum, &'a T>;

    fn deref(&self) -> &Self::Target {
        &self.results
    }
}
