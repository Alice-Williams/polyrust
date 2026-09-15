//! Closed source grammar: signature roles never stand in for leaf effects.
use super::super::{LinearError as Error, Result, source::local};
use super::{frame, frame::Frame};
use crate::owned_source::{
    BoxConstructionInput,
    local_call::{CallRole, LocalCallInput},
};
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::ty::{Ty, TyCtxt};

pub(super) enum Operation<'tcx> {
    Allocate {
        input: BoxConstructionInput<'tcx>,
        binding: Option<HirId>,
    },
    Move(HirId),
    Call {
        input: LocalCallInput<'tcx>,
        binding: Option<HirId>,
    },
}
#[derive(Clone, Copy)]
pub(super) enum Finish {
    Read,
    Owner,
    Direct,
}
pub(super) struct Plan<'tcx> {
    pub frame: Frame<'tcx>,
    pub box_ty: Ty<'tcx>,
    pub operations: Vec<Operation<'tcx>>,
    pub finish: Finish,
}
impl<'tcx> Plan<'tcx> {
    pub fn call(&self) -> Result<&LocalCallInput<'tcx>> {
        let mut calls = self.operations.iter().filter_map(|op| match op {
            Operation::Call { input, .. } => Some(input),
            _ => None,
        });
        let call = calls.next().ok_or(Error::Call)?;
        if calls.next().is_some() {
            return Err(Error::Call);
        }
        Ok(call)
    }
}

pub(super) fn entry(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    let frame = frame::read(tcx, owner)?;
    if frame.signature.inputs() != [tcx.types.i32] || frame.signature.output() != tcx.types.i32 {
        return Err(Error::Signature);
    }
    let mut calls = frame
        .declarations
        .iter()
        .map(|(_, e)| *e)
        .chain(std::iter::once(frame.scopes.exit().value()))
        .filter_map(|e| LocalCallInput::read(tcx, owner, e).ok());
    let call = calls.next().ok_or(Error::Call)?;
    if calls.next().is_some() {
        return Err(Error::Call);
    }
    let role = call.role();
    let box_ty = call.box_ty();
    drop(calls);
    parse(tcx, frame, box_ty, Form::Entry(role))
}
pub(super) fn leaf<'tcx>(tcx: TyCtxt<'tcx>, call: &LocalCallInput<'tcx>) -> Result<Plan<'tcx>> {
    let frame = frame::read(tcx, call.callee())?;
    if frame.signature != call.signature() {
        return Err(Error::Signature);
    }
    parse(tcx, frame, call.box_ty(), Form::Leaf(call.role()))
}
#[derive(Clone, Copy)]
enum Form {
    Entry(CallRole),
    Leaf(CallRole),
}

fn parse<'tcx>(
    tcx: TyCtxt<'tcx>,
    frame: Frame<'tcx>,
    box_ty: Ty<'tcx>,
    form: Form,
) -> Result<Plan<'tcx>> {
    let checked = tcx.typeck(frame.owner);
    let mut operations = Vec::new();
    let mut current = match form {
        Form::Leaf(CallRole::Consumer | CallRole::Relay) => Some(frame.parameter),
        _ => None,
    };
    let mut allocated = false;
    let mut called = false;
    for (id, expression) in &frame.declarations {
        if checked.node_type(*id) != box_ty {
            return Err(Error::SourceIdentity);
        }
        if current.is_some_and(|owner| local(checked, expression) == Ok(owner)) {
            operations.push(Operation::Move(*id));
        } else if current.is_none() {
            match form {
                Form::Entry(CallRole::Producer) => {
                    let input =
                        local_call(tcx, &frame, expression, frame.parameter, CallRole::Producer)?;
                    operations.push(Operation::Call {
                        input,
                        binding: Some(*id),
                    });
                    called = true;
                }
                Form::Entry(CallRole::Consumer | CallRole::Relay)
                | Form::Leaf(CallRole::Producer) => {
                    let input = allocation(tcx, &frame, expression, box_ty)?;
                    operations.push(Operation::Allocate {
                        input,
                        binding: Some(*id),
                    });
                    allocated = true;
                }
                _ => return Err(Error::BodyShape),
            }
        } else if matches!(form, Form::Entry(CallRole::Relay)) && !called {
            let input = local_call(tcx, &frame, expression, current.unwrap(), CallRole::Relay)?;
            operations.push(Operation::Call {
                input,
                binding: Some(*id),
            });
            called = true;
        } else {
            return Err(Error::BodyShape);
        }
        current = Some(*id);
    }
    let expression = frame.scopes.exit().value();
    let finish = match form {
        Form::Leaf(CallRole::Producer) if !allocated => {
            if current.is_some() {
                return Err(Error::BodyShape);
            }
            let input = allocation(tcx, &frame, expression, box_ty)?;
            operations.push(Operation::Allocate {
                input,
                binding: None,
            });
            Finish::Direct
        }
        Form::Leaf(CallRole::Producer | CallRole::Relay) => {
            if Some(local(checked, expression)?) != current {
                return Err(Error::SourceIdentity);
            }
            Finish::Owner
        }
        Form::Entry(CallRole::Consumer) => {
            let input = local_call(
                tcx,
                &frame,
                expression,
                current.ok_or(Error::BodyShape)?,
                CallRole::Consumer,
            )?;
            operations.push(Operation::Call {
                input,
                binding: None,
            });
            Finish::Direct
        }
        Form::Entry(CallRole::Producer | CallRole::Relay) | Form::Leaf(CallRole::Consumer) => {
            if matches!(form, Form::Entry(_)) && !called {
                return Err(Error::Call);
            }
            let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = expression.kind else {
                return Err(Error::Read);
            };
            if Some(local(checked, operand)?) != current
                || checked.expr_ty(operand) != box_ty
                || checked.expr_ty(expression) != tcx.types.i32
            {
                return Err(Error::Read);
            }
            Finish::Read
        }
    };
    Ok(Plan {
        frame,
        box_ty,
        operations,
        finish,
    })
}
fn allocation<'tcx>(
    tcx: TyCtxt<'tcx>,
    frame: &Frame<'tcx>,
    expression: &'tcx hir::Expr<'tcx>,
    box_ty: Ty<'tcx>,
) -> Result<BoxConstructionInput<'tcx>> {
    let input =
        BoxConstructionInput::read(tcx, frame.owner, expression).map_err(Error::Constructor)?;
    if input.result() != box_ty
        || local(tcx.typeck(frame.owner), input.argument())? != frame.parameter
    {
        return Err(Error::Argument);
    }
    Ok(input)
}
fn local_call<'tcx>(
    tcx: TyCtxt<'tcx>,
    frame: &Frame<'tcx>,
    expression: &'tcx hir::Expr<'tcx>,
    argument: HirId,
    role: CallRole,
) -> Result<LocalCallInput<'tcx>> {
    let input = LocalCallInput::read(tcx, frame.owner, expression).map_err(|_| Error::Call)?;
    if input.role() != role || local(tcx.typeck(frame.owner), input.argument())? != argument {
        return Err(Error::Argument);
    }
    Ok(input)
}
