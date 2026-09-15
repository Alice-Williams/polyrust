//! Nominal record/field identity only; not aggregate ownership correspondence.
use crate::source_capabilities::Capability;
use rustc_abi::FieldIdx;
use rustc_hir::{
    self as hir,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{self, AdtDef, GenericArgsRef, Ty, TyCtxt};
use std::collections::HashSet;

#[cfg(owned_record_proof)]
#[path = "../../test/owned_record/allocator.rs"]
pub(crate) mod allocator;

fn check_field<'tcx>(
    actual: Ty<'tcx>,
    expected: Ty<'tcx>,
    initializer: Ty<'tcx>,
) -> Result<(), RecordError> {
    if actual != expected || initializer != actual {
        return Err(RecordError::UnsupportedField);
    }
    Ok(())
}

pub(crate) struct OwnedRecordConstruction;
impl Capability for OwnedRecordConstruction {
    type Input<'tcx> = RecordConstructionInput<'tcx>;
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RecordError {
    WrongOwner,
    NonCanonicalNode,
    NotCompleteStruct,
    UnsupportedRecord,
    UnsupportedField,
    FieldCoverage,
    Adjustment,
}

pub(crate) struct RecordField<'tcx> {
    index: FieldIdx,
    declaration: DefId,
    ty: Ty<'tcx>,
    initializer: &'tcx hir::Expr<'tcx>,
}
impl<'tcx> RecordField<'tcx> {
    pub(crate) fn index(&self) -> FieldIdx {
        self.index
    }
    pub(crate) fn declaration(&self) -> DefId {
        self.declaration
    }
    pub(crate) fn ty(&self) -> Ty<'tcx> {
        self.ty
    }
    pub(crate) fn initializer(&self) -> &'tcx hir::Expr<'tcx> {
        self.initializer
    }
}

pub(crate) struct RecordConstructionInput<'tcx> {
    owner: LocalDefId,
    expression: &'tcx hir::Expr<'tcx>,
    definition: AdtDef<'tcx>,
    arguments: GenericArgsRef<'tcx>,
    result: Ty<'tcx>,
    fields: Vec<RecordField<'tcx>>,
}
impl<'tcx> RecordConstructionInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        owner: LocalDefId,
        expression: &hir::Expr<'tcx>,
    ) -> Result<Self, RecordError> {
        if expression.hir_id.owner.def_id != owner {
            return Err(RecordError::WrongOwner);
        }
        let hir::Node::Expr(canonical) = tcx.hir_node(expression.hir_id) else {
            return Err(RecordError::NonCanonicalNode);
        };
        if !std::ptr::eq(canonical, expression) {
            return Err(RecordError::NonCanonicalNode);
        }
        let expression = canonical;
        let hir::ExprKind::Struct(_, initializers, hir::StructTailExpr::None) = expression.kind
        else {
            return Err(RecordError::NotCompleteStruct);
        };
        let checked = tcx.typeck(owner);
        if !checked.expr_adjustments(expression).is_empty() {
            return Err(RecordError::Adjustment);
        }
        let result = checked.expr_ty(expression);
        let &ty::Adt(definition, arguments) = result.kind() else {
            return Err(RecordError::UnsupportedRecord);
        };
        if !definition.did().is_local()
            || !definition.is_struct()
            || !arguments.is_empty()
            || tcx.generics_of(definition.did()).count() != 0
            || definition.has_dtor(tcx)
        {
            return Err(RecordError::UnsupportedRecord);
        }
        let repr = definition.repr();
        if repr.int.is_some()
            || repr.align.is_some()
            || repr.pack.is_some()
            || repr.scalable.is_some()
            || !repr.flags.is_empty()
        {
            return Err(RecordError::UnsupportedRecord);
        }
        let variant = definition.non_enum_variant();
        if variant.ctor_kind().is_some() || variant.fields.is_empty() || variant.fields.len() > 128
        {
            return Err(RecordError::UnsupportedRecord);
        }
        let constructor = tcx
            .get_diagnostic_item(rustc_span::sym::box_new)
            .ok_or(RecordError::UnsupportedField)?;
        if tcx.generics_of(constructor).count() != 1 {
            return Err(RecordError::UnsupportedField);
        }
        let constructor_args = tcx.mk_args(&[tcx.types.i32.into()]);
        let signature = tcx
            .fn_sig(constructor)
            .instantiate(tcx, constructor_args)
            .skip_binder();
        let box_ty = signature.output();
        if signature.inputs() != [tcx.types.i32] {
            return Err(RecordError::UnsupportedField);
        }
        let mut seen = HashSet::new();
        let mut fields = Vec::new();
        for field in initializers {
            let index = checked.field_index(field.hir_id);
            let declaration = variant
                .fields
                .get(index)
                .ok_or(RecordError::FieldCoverage)?;
            if !seen.insert(index) {
                return Err(RecordError::FieldCoverage);
            }
            let ty = tcx
                .try_normalize_erasing_regions(
                    ty::TypingEnv::fully_monomorphized(),
                    declaration.ty(tcx, arguments),
                )
                .map_err(|_| RecordError::UnsupportedField)?;
            check_field(ty, box_ty, checked.expr_ty(field.expr))?;
            if !checked.expr_adjustments(field.expr).is_empty() {
                return Err(RecordError::Adjustment);
            }
            fields.push(RecordField {
                index,
                declaration: declaration.did,
                ty,
                initializer: field.expr,
            });
        }
        if fields.len() != variant.fields.len() {
            return Err(RecordError::FieldCoverage);
        }
        Ok(Self {
            owner,
            expression,
            definition,
            arguments,
            result,
            fields,
        })
    }
    pub(crate) fn owner(&self) -> LocalDefId {
        self.owner
    }
    pub(crate) fn expression(&self) -> &'tcx hir::Expr<'tcx> {
        self.expression
    }
    pub(crate) fn definition(&self) -> AdtDef<'tcx> {
        self.definition
    }
    pub(crate) fn arguments(&self) -> GenericArgsRef<'tcx> {
        self.arguments
    }
    pub(crate) fn result(&self) -> Ty<'tcx> {
        self.result
    }
    /// Source initializer order; each entry retains its independent field index.
    pub(crate) fn fields(&self) -> &[RecordField<'tcx>] {
        &self.fields
    }
}
