//! Consuming registration: absent, duplicate and wrong-capability slots differ.
use super::{
    Capability, DirectCalls, EntrySignatures, FunctionSignatures, LexicalControl, LiteralValues,
    Mapping, ObjectTypes, RecordInitializers, ResolvedPlaces, ScalarComparisons, SharedBorrows,
    Supports,
};
use crate::c_lower::Reader;
use portable_backend_c::ast::{CBlock, CFunctionType, CInitializer, CObjectType, CPlace, CValue};

// Bounds on the complete executable C signatures, never independent flags.
pub(crate) trait ReaderMapping<C: Capability, O>:
    for<'tcx> Mapping<Capability = C, Context<'tcx> = Reader<'tcx>, Output = O>
{
}
impl<C: Capability, O, M> ReaderMapping<C, O> for M where
    M: for<'tcx> Mapping<Capability = C, Context<'tcx> = Reader<'tcx>, Output = O>
{
}
pub(crate) trait EntryMapping:
    for<'tcx> Mapping<Capability = EntrySignatures, Context<'tcx> = (), Output = CFunctionType>
{
}
impl<M> EntryMapping for M where
    M: for<'tcx> Mapping<Capability = EntrySignatures, Context<'tcx> = (), Output = CFunctionType>
{
}

pub(crate) struct Missing;
#[derive(Clone, Copy)]
pub(crate) struct Bindings<L, C, P, B, T, R, S, F, D, G> {
    literal_values: L,
    scalar_comparisons: C,
    resolved_places: P,
    shared_borrows: B,
    object_types: T,
    record_initializers: R,
    lexical_control: S,
    entry_signatures: F,
    direct_calls: D,
    function_signatures: G,
}
pub(crate) struct Builder<
    L = Missing,
    C = Missing,
    P = Missing,
    B = Missing,
    T = Missing,
    R = Missing,
    S = Missing,
    F = Missing,
    D = Missing,
    G = Missing,
>(Bindings<L, C, P, B, T, R, S, F, D, G>);
impl Builder {
    pub(crate) fn new() -> Self {
        Self(Bindings {
            literal_values: Missing,
            scalar_comparisons: Missing,
            resolved_places: Missing,
            shared_borrows: Missing,
            object_types: Missing,
            record_initializers: Missing,
            lexical_control: Missing,
            entry_signatures: Missing,
            direct_calls: Missing,
            function_signatures: Missing,
        })
    }
}
// Each expansion replaces exactly one Missing field, preserving all others.
macro_rules! register {
    ($m:ident; $field:ident, $bound:path;
     [$($generic:ident),*]; [$($before:ty),*]; [$($after:ty),*]; [$($keep:ident),*]) => {
        impl<$($generic),*> Builder<$($before),*> {
            pub(crate) fn $field<$m: $bound>(self, mapping: $m) -> Builder<$($after),*> {
                Builder(Bindings { $field: mapping, $($keep: self.0.$keep),* })
            }
        }
    };
}
register!(M; literal_values, ReaderMapping<LiteralValues, CValue>;
    [C, P, B, T, R, S, F, D, G]; [Missing, C, P, B, T, R, S, F, D, G]; [M, C, P, B, T, R, S, F, D, G];
    [scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures]);

register!(M; scalar_comparisons, ReaderMapping<ScalarComparisons, CValue>;
    [L, P, B, T, R, S, F, D, G]; [L, Missing, P, B, T, R, S, F, D, G]; [L, M, P, B, T, R, S, F, D, G];
    [literal_values, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures]);

register!(M; resolved_places, ReaderMapping<ResolvedPlaces, CPlace>;
    [L, C, B, T, R, S, F, D, G]; [L, C, Missing, B, T, R, S, F, D, G]; [L, C, M, B, T, R, S, F, D, G];
    [literal_values, scalar_comparisons, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures]);

register!(M; shared_borrows, ReaderMapping<SharedBorrows, CValue>;
    [L, C, P, T, R, S, F, D, G]; [L, C, P, Missing, T, R, S, F, D, G]; [L, C, P, M, T, R, S, F, D, G];
    [literal_values, scalar_comparisons, resolved_places, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures]);

register!(M; object_types, ReaderMapping<ObjectTypes, CObjectType>;
    [L, C, P, B, R, S, F, D, G]; [L, C, P, B, Missing, R, S, F, D, G]; [L, C, P, B, M, R, S, F, D, G];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures]);

register!(M; record_initializers, ReaderMapping<RecordInitializers, CInitializer>;
    [L, C, P, B, T, S, F, D, G]; [L, C, P, B, T, Missing, S, F, D, G]; [L, C, P, B, T, M, S, F, D, G];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, lexical_control, entry_signatures, direct_calls, function_signatures]);

register!(M; lexical_control, ReaderMapping<LexicalControl, CBlock>;
    [L, C, P, B, T, R, F, D, G]; [L, C, P, B, T, R, Missing, F, D, G]; [L, C, P, B, T, R, M, F, D, G];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, entry_signatures, direct_calls, function_signatures]);

register!(M; entry_signatures, EntryMapping;
    [L, C, P, B, T, R, S, D, G]; [L, C, P, B, T, R, S, Missing, D, G]; [L, C, P, B, T, R, S, M, D, G];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, direct_calls, function_signatures]);

impl<L, C, P, B, T, R, S, F, D, G> Builder<L, C, P, B, T, R, S, F, D, G>
where
    L: ReaderMapping<LiteralValues, CValue>,
    C: ReaderMapping<ScalarComparisons, CValue>,
    P: ReaderMapping<ResolvedPlaces, CPlace>,
    B: ReaderMapping<SharedBorrows, CValue>,
    T: ReaderMapping<ObjectTypes, CObjectType>,
    R: ReaderMapping<RecordInitializers, CInitializer>,
    S: ReaderMapping<LexicalControl, CBlock>,
    F: EntryMapping,
    D: ReaderMapping<DirectCalls, CValue>,
    G: FunctionMapping,
{
    pub(crate) fn build(self) -> Bindings<L, C, P, B, T, R, S, F, D, G> {
        self.0
    }
}
macro_rules! support {
    ($capability:ty, $slot:ident, $field:ident, $bound:path) => {
        impl<L, C, P, B, T, R, S, F, D, G> Supports<$capability>
            for Bindings<L, C, P, B, T, R, S, F, D, G>
        where
            $slot: $bound,
        {
            type Mapping = $slot;
            fn mapping(&self) -> $slot {
                self.$field
            }
        }
    };
}
support!(LiteralValues, L, literal_values, ReaderMapping<LiteralValues, CValue>);
support!(ScalarComparisons, C, scalar_comparisons, ReaderMapping<ScalarComparisons, CValue>);
support!(ResolvedPlaces, P, resolved_places, ReaderMapping<ResolvedPlaces, CPlace>);
support!(SharedBorrows, B, shared_borrows, ReaderMapping<SharedBorrows, CValue>);
support!(ObjectTypes, T, object_types, ReaderMapping<ObjectTypes, CObjectType>);
support!(RecordInitializers, R, record_initializers, ReaderMapping<RecordInitializers, CInitializer>);
support!(LexicalControl, S, lexical_control, ReaderMapping<LexicalControl, CBlock>);
support!(EntrySignatures, F, entry_signatures, EntryMapping);

pub(crate) trait FunctionMapping:
    for<'tcx> Mapping<Capability = FunctionSignatures, Context<'tcx> = (), Output = CFunctionType>
{
}
impl<M> FunctionMapping for M where
    M: for<'tcx> Mapping<
            Capability = FunctionSignatures,
            Context<'tcx> = (),
            Output = CFunctionType,
        >
{
}
register!(M; direct_calls, ReaderMapping<DirectCalls, CValue>;
    [L, C, P, B, T, R, S, F, G]; [L, C, P, B, T, R, S, F, Missing, G]; [L, C, P, B, T, R, S, F, M, G];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, function_signatures]);
register!(M; function_signatures, FunctionMapping;
    [L, C, P, B, T, R, S, F, D]; [L, C, P, B, T, R, S, F, D, Missing]; [L, C, P, B, T, R, S, F, D, M];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls]);
support!(DirectCalls, D, direct_calls, ReaderMapping<DirectCalls, CValue>);
support!(FunctionSignatures, G, function_signatures, FunctionMapping);
