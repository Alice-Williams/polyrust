//! Consuming registration: absent, duplicate and wrong-capability slots differ.
use super::{
    BooleanNegation, Capability, DirectCalls, EntrySignatures, FunctionSignatures, IntegerBitwise,
    LexicalControl, LiteralValues, Mapping, ObjectTypes, RecordInitializers, ResolvedPlaces,
    ScalarComparisons, SharedBorrows, ShortCircuitBooleans, Supports,
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
pub(crate) struct Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I> {
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
    boolean_negation: N,
    short_circuit_booleans: H,
    integer_bitwise: I,
}
#[expect(
    clippy::type_complexity,
    reason = "Independent capability slots encode missing/duplicate registrations in types"
)]
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
    N = Missing,
    H = Missing,
    I = Missing,
>(Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I>);
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
            boolean_negation: Missing,
            short_circuit_booleans: Missing,
            integer_bitwise: Missing,
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
    [C, P, B, T, R, S, F, D, G, N, H, I]; [Missing, C, P, B, T, R, S, F, D, G, N, H, I]; [M, C, P, B, T, R, S, F, D, G, N, H, I];
    [scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);

register!(M; scalar_comparisons, ReaderMapping<ScalarComparisons, CValue>;
    [L, P, B, T, R, S, F, D, G, N, H, I]; [L, Missing, P, B, T, R, S, F, D, G, N, H, I]; [L, M, P, B, T, R, S, F, D, G, N, H, I];
    [literal_values, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);

register!(M; resolved_places, ReaderMapping<ResolvedPlaces, CPlace>;
    [L, C, B, T, R, S, F, D, G, N, H, I]; [L, C, Missing, B, T, R, S, F, D, G, N, H, I]; [L, C, M, B, T, R, S, F, D, G, N, H, I];
    [literal_values, scalar_comparisons, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);

register!(M; shared_borrows, ReaderMapping<SharedBorrows, CValue>;
    [L, C, P, T, R, S, F, D, G, N, H, I]; [L, C, P, Missing, T, R, S, F, D, G, N, H, I]; [L, C, P, M, T, R, S, F, D, G, N, H, I];
    [literal_values, scalar_comparisons, resolved_places, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);

register!(M; object_types, ReaderMapping<ObjectTypes, CObjectType>;
    [L, C, P, B, R, S, F, D, G, N, H, I]; [L, C, P, B, Missing, R, S, F, D, G, N, H, I]; [L, C, P, B, M, R, S, F, D, G, N, H, I];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);

register!(M; record_initializers, ReaderMapping<RecordInitializers, CInitializer>;
    [L, C, P, B, T, S, F, D, G, N, H, I]; [L, C, P, B, T, Missing, S, F, D, G, N, H, I]; [L, C, P, B, T, M, S, F, D, G, N, H, I];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);

register!(M; lexical_control, ReaderMapping<LexicalControl, CBlock>;
    [L, C, P, B, T, R, F, D, G, N, H, I]; [L, C, P, B, T, R, Missing, F, D, G, N, H, I]; [L, C, P, B, T, R, M, F, D, G, N, H, I];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);

register!(M; entry_signatures, EntryMapping;
    [L, C, P, B, T, R, S, D, G, N, H, I]; [L, C, P, B, T, R, S, Missing, D, G, N, H, I]; [L, C, P, B, T, R, S, M, D, G, N, H, I];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);

impl<L, C, P, B, T, R, S, F, D, G, N, H, I> Builder<L, C, P, B, T, R, S, F, D, G, N, H, I>
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
    N: ReaderMapping<BooleanNegation, CValue>,
    H: ReaderMapping<ShortCircuitBooleans, CValue>,
    I: ReaderMapping<IntegerBitwise, CValue>,
{
    #[expect(
        clippy::type_complexity,
        reason = "Preserve each executable mapping type at the completed builder boundary"
    )]
    pub(crate) fn build(self) -> Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I> {
        self.0
    }
}
macro_rules! support {
    ($capability:ty, $slot:ident, $field:ident, $bound:path) => {
        impl<L, C, P, B, T, R, S, F, D, G, N, H, I> Supports<$capability>
            for Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I>
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
    [L, C, P, B, T, R, S, F, G, N, H, I]; [L, C, P, B, T, R, S, F, Missing, G, N, H, I]; [L, C, P, B, T, R, S, F, M, G, N, H, I];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise]);
register!(M; function_signatures, FunctionMapping;
    [L, C, P, B, T, R, S, F, D, N, H, I]; [L, C, P, B, T, R, S, F, D, Missing, N, H, I]; [L, C, P, B, T, R, S, F, D, M, N, H, I];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, boolean_negation, short_circuit_booleans, integer_bitwise]);
support!(DirectCalls, D, direct_calls, ReaderMapping<DirectCalls, CValue>);
support!(FunctionSignatures, G, function_signatures, FunctionMapping);
register!(M; boolean_negation, ReaderMapping<BooleanNegation, CValue>;
    [L, C, P, B, T, R, S, F, D, G, H, I]; [L, C, P, B, T, R, S, F, D, G, Missing, H, I]; [L, C, P, B, T, R, S, F, D, G, M, H, I];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, short_circuit_booleans, integer_bitwise]);
support!(BooleanNegation, N, boolean_negation, ReaderMapping<BooleanNegation, CValue>);
register!(M; short_circuit_booleans, ReaderMapping<ShortCircuitBooleans, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, I]; [L, C, P, B, T, R, S, F, D, G, N, Missing, I]; [L, C, P, B, T, R, S, F, D, G, N, M, I];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, integer_bitwise]);
support!(ShortCircuitBooleans, H, short_circuit_booleans, ReaderMapping<ShortCircuitBooleans, CValue>);

register!(M; integer_bitwise, ReaderMapping<IntegerBitwise, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H]; [L, C, P, B, T, R, S, F, D, G, N, H, Missing]; [L, C, P, B, T, R, S, F, D, G, N, H, M];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans]);
support!(IntegerBitwise, I, integer_bitwise, ReaderMapping<IntegerBitwise, CValue>);
