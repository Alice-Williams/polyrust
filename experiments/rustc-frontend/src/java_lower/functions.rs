//! Bounded local-body discovery with separate resolved foreign declarations.
use super::Result;
use crate::source_origin::identity;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::{DefId, LocalDefId},
    intravisit::{self, Visitor},
};
use rustc_middle::ty::{self, TyCtxt, TypeckResults};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub(super) fn resolve<'tcx>(
    tcx: TyCtxt<'tcx>,
    checked: &TypeckResults<'tcx>,
    expression: &'tcx hir::Expr<'tcx>,
) -> Result<DefId> {
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
    Ok(resolved)
}

pub(super) struct Inventory {
    pub(super) local: Vec<LocalDefId>,
    pub(super) foreign: Vec<DefId>,
    pub(super) constants: Vec<DefId>,
}

pub(super) fn inventory(tcx: TyCtxt<'_>, roots: &[LocalDefId]) -> Result<Inventory> {
    let mut pending = VecDeque::from(roots.to_vec());
    let mut graph = HashMap::new();
    let mut foreign = BTreeMap::new();
    let mut constants = BTreeMap::new();
    let mut remaining = 100_000;
    while let Some(function) = pending.pop_front() {
        if graph.contains_key(&function) {
            continue;
        }
        if graph.len() >= 4096 {
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
            constants.insert(identity(tcx, definition), definition);
            if constants.len() > 4096 {
                return Err("source constant import inventory budget exceeded".into());
            }
        }
        let mut local = Vec::new();
        for callee in visitor.calls {
            match callee.as_local() {
                Some(id) => local.push(id),
                None => {
                    foreign.insert(identity(tcx, callee), callee);
                }
            }
        }
        pending.extend(local.iter().copied());
        graph.insert(function, local);
    }
    let mut depths = HashMap::new();
    for function in graph.keys() {
        call_depth(*function, &graph, &mut HashSet::new(), &mut depths, 0)?;
    }
    let mut result: Vec<_> = graph.into_keys().collect();
    result.sort_by_key(|id| identity(tcx, id.to_def_id()));
    Ok(Inventory {
        local: result,
        foreign: foreign.into_values().collect(),
        constants: constants.into_values().collect(),
    })
}

fn call_depth(
    id: LocalDefId,
    graph: &HashMap<LocalDefId, Vec<LocalDefId>>,
    visiting: &mut HashSet<LocalDefId>,
    depths: &mut HashMap<LocalDefId, usize>,
    depth: usize,
) -> Result<usize> {
    if depth >= 128 || !visiting.insert(id) {
        return Err("recursive or excessive-depth Java source calls are not implemented".into());
    }
    if let Some(height) = depths.get(&id) {
        visiting.remove(&id);
        return Ok(*height);
    }
    let mut height = 1;
    for child in &graph[&id] {
        height = height.max(1 + call_depth(*child, graph, visiting, depths, depth + 1)?);
    }
    if height > 128 {
        return Err("Java source call-depth budget exceeded".into());
    }
    visiting.remove(&id);
    depths.insert(id, height);
    Ok(height)
}

struct Calls<'tcx> {
    tcx: TyCtxt<'tcx>,
    checked: &'tcx TypeckResults<'tcx>,
    calls: Vec<DefId>,
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
        if self.depth >= 128 || self.remaining == 0 {
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
        })
        .and_then(|builtin| {
            if builtin {
                Ok(true)
            } else {
                crate::source_capabilities::AdditionInput::discover(
                    self.tcx,
                    self.checked,
                    expression,
                )
                .map(|input| input.is_some())
            }
        })
        .and_then(|builtin| {
            if builtin {
                Ok(true)
            } else {
                crate::source_capabilities::SubtractionInput::discover(
                    self.tcx,
                    self.checked,
                    expression,
                )
                .map(|input| input.is_some())
            }
        })
        .and_then(|builtin| {
            if builtin {
                Ok(true)
            } else {
                crate::source_capabilities::MultiplicationInput::discover(
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
                Ok(id) => self.calls.push(id),
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
