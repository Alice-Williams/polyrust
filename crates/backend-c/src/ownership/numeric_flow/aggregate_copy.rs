//! Copy an actual source subobject snapshot; do not recreate clean field defaults.
use super::{
    E, Engine, State,
    provenance::Origin,
    storage::{Key, Root},
};
use crate::ast::{CObjectType, CObjectTypeKind, CValue, CValueKind};

impl<'a> Engine<'a> {
    pub(super) fn aggregate_copy(
        &mut self,
        value: &'a CValue,
        ty: &CObjectType,
        destination: &Key,
        before: &State<'a>,
        state: &mut State<'a>,
    ) -> Result<(), E> {
        if !matches!(
            ty.canonical().kind(),
            CObjectTypeKind::Struct(_) | CObjectTypeKind::Union(_)
        ) {
            return Ok(());
        }
        if let CValueKind::Read(place) = value.kind()
            && let Some(source) = Key::place(place, &mut self.layouts)
        {
            state.copy(before, &source, destination);
            // Unmaterialized mutable global fields are interprocedural inputs,
            // not known-clean static initializers. Exact copied cells override
            // this fallback through their retained per-field provenance.
            if matches!(source.root(), Root::Global(_)) {
                state.fallback_origin(destination.root(), Origin::Aggregate(value));
            }
        } else {
            state.poison(destination.root(), Origin::Aggregate(value));
        }
        Ok(())
    }
}
