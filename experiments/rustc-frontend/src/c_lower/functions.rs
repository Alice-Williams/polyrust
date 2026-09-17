//! Finite compiler-resolved function inventory, before any target body lowering.
use super::{ForeignLookup, Result};
use portable_backend_c::dialect::CDependencyFunction;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::{DefId, LocalDefId},
    intravisit::{self, Visitor},
};
use rustc_middle::ty::{self, TyCtxt, TypeckResults};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy)]
pub(super) enum CallTarget {
    Local(LocalDefId),
    Foreign(DefId),
}

pub(super) struct Inventory {
    pub owned: Vec<LocalDefId>,
    pub foreign: HashMap<DefId, CDependencyFunction>,
    pub constants: Vec<DefId>,
}

pub(super) fn resolve<'tcx>(
    tcx: TyCtxt<'tcx>,
    checked: &TypeckResults<'tcx>,
    expression: &'tcx hir::Expr<'tcx>,
) -> Result<CallTarget> {
    let hir::ExprKind::Call(callee, _) = expression.kind else {
        return Err("direct-call mapping requires an ordinary call".into());
    };
    if !checked.expr_adjustments(expression).is_empty()
        || !checked.expr_adjustments(callee).is_empty()
    {
        return Err("direct-call compiler adjustments are not implemented".into());
    }
    let hir::ExprKind::Path(path) = &callee.kind else {
        return Err("indirect calls are not implemented".into());
    };
    let Res::Def(DefKind::Fn, resolved) = checked.qpath_res(path, callee.hir_id) else {
        return Err("direct calls require resolved ordinary functions".into());
    };
    let ty::FnDef(typed, arguments) = checked.expr_ty(callee).kind() else {
        return Err("indirect calls are not implemented".into());
    };
    if *typed != resolved || !arguments.is_empty() || tcx.generics_of(resolved).count() != 0 {
        return Err("generic or mismatched direct callee identity".into());
    }
    Ok(match resolved.as_local() {
        Some(local) => CallTarget::Local(local),
        None => CallTarget::Foreign(resolved),
    })
}

pub(super) fn inventory(
    tcx: TyCtxt<'_>,
    roots: &[LocalDefId],
    lookup: Option<&ForeignLookup<'_>>,
) -> Result<Inventory> {
    let mut pending = VecDeque::from(roots.to_vec());
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    let mut foreign = HashMap::new();
    let mut constants = std::collections::BTreeMap::new();
    let mut remaining = 100_000;
    while let Some(function) = pending.pop_front() {
        if !seen.insert(function) {
            continue;
        }
        if seen.len() + foreign.len() > 4096 {
            return Err("source function inventory budget exceeded".into());
        }
        let mut visitor = Calls {
            tcx,
            checked: tcx.typeck(function),
            calls: vec![],
            constants: vec![],
            error: None,
            depth: 0,
            remaining,
        };
        visitor.visit_body(tcx.hir_body_owned_by(function));
        if let Some(error) = visitor.error {
            return Err(error);
        }
        remaining = visitor.remaining;
        for definition in visitor.constants {
            constants.insert(crate::source_origin::identity(tcx, definition), definition);
            if constants.len() > 4096 {
                return Err("source constant import inventory budget exceeded".into());
            }
        }
        for target in visitor.calls {
            match target {
                CallTarget::Local(local) => pending.push_back(local),
                CallTarget::Foreign(id) => {
                    if let std::collections::hash_map::Entry::Vacant(entry) = foreign.entry(id) {
                        let lookup = lookup.ok_or("foreign direct calls are not implemented")?;
                        entry.insert((lookup.function)(id)?);
                        if seen.len() + foreign.len() > 4096 {
                            return Err("source function inventory budget exceeded".into());
                        }
                    }
                }
            }
        }
        result.push(function);
    }
    Ok(Inventory {
        owned: result,
        foreign,
        constants: constants.into_values().collect(),
    })
}

struct Calls<'tcx> {
    tcx: TyCtxt<'tcx>,
    checked: &'tcx TypeckResults<'tcx>,
    calls: Vec<CallTarget>,
    constants: Vec<DefId>,
    error: Option<String>,
    depth: usize,
    remaining: usize,
}
impl<'tcx> Visitor<'tcx> for Calls<'tcx> {
    fn visit_expr(&mut self, expression: &'tcx hir::Expr<'tcx>) {
        if self.error.is_some() {
            return;
        }
        if self.remaining == 0 || self.depth >= 128 {
            self.error = Some("source call inventory traversal budget exceeded".into());
            return;
        }
        self.remaining -= 1;
        if let Some(id) = crate::source_capabilities::ConstantImportInput::discover(
            self.tcx,
            self.checked,
            expression,
        ) {
            self.constants.push(id);
        }
        let builtin = match crate::source_capabilities::WrappingInput::discover(
            self.tcx,
            self.checked,
            expression,
        )
        .and_then(|input| {
            if input.is_some() {
                Ok(true)
            } else {
                crate::source_capabilities::NaNInput::discover(self.tcx, self.checked, expression)
                    .and_then(|input| {
                        if input.is_some() {
                            Ok(true)
                        } else {
                            crate::source_capabilities::AbsoluteInput::discover(
                                self.tcx,
                                self.checked,
                                expression,
                            )
                            .map(|input| input.is_some())
                        }
                    })
            }
        })
        .and_then(|builtin| {
            if builtin {
                Ok(true)
            } else {
                crate::source_capabilities::TruncationInput::discover(
                    self.tcx,
                    self.checked,
                    expression,
                )
                .map(|input| input.is_some())
            }
        }) {
            Ok(admitted) => admitted,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        // Keep walking the receiver: its ordinary calls still need inventory
        // and original producer authority even though the primitive itself does not.
        if !builtin && matches!(expression.kind, hir::ExprKind::Call(..)) {
            match resolve(self.tcx, self.checked, expression) {
                Ok(target) => self.calls.push(target),
                Err(error) => {
                    self.error = Some(error);
                    return;
                }
            }
        }
        self.depth += 1;
        intravisit::walk_expr(self, expression);
        self.depth -= 1;
    }
}
