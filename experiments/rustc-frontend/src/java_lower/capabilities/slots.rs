//! Consuming registration: absent, duplicate and wrong-capability slots differ.
use super::{
    BooleanNegation, Capability, DirectCalls, EagerBooleans, EntrySignatures, FunctionSignatures,
    IntegerBitwise, LexicalControl, LiteralValues, Mapping, ObjectTypes, RecordInitializers,
    ResolvedPlaces, ScalarComparisons, ScalarConstants, SharedBorrows, ShortCircuitBooleans,
    Supports,
};
use crate::java_lower::Reader;
use crate::java_lower::representation::{Place, TypePlan, Value};
use portable_backend_java::ast::{JavaBlock, JavaMethodSignature};

// Bounds on the complete executable Java signatures, never independent flags.
pub(crate) trait ReaderMapping<C: Capability, O>:
    for<'tcx> Mapping<Capability = C, Context<'tcx> = Reader<'tcx>, Output = O>
{
}
impl<C: Capability, O, M> ReaderMapping<C, O> for M where
    M: for<'tcx> Mapping<Capability = C, Context<'tcx> = Reader<'tcx>, Output = O>
{
}
pub(crate) trait EntryMapping:
    for<'tcx> Mapping<Capability = EntrySignatures, Context<'tcx> = (), Output = JavaMethodSignature>
{
}
impl<M> EntryMapping for M where
    M: for<'tcx> Mapping<
            Capability = EntrySignatures,
            Context<'tcx> = (),
            Output = JavaMethodSignature,
        >
{
}

pub(crate) struct Missing;
#[derive(Clone, Copy)]
pub(crate) struct Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K> {
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
    eager_booleans: E,
    scalar_constants: K,
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
    E = Missing,
    K = Missing,
>(Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K>);
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
            eager_booleans: Missing,
            scalar_constants: Missing,
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
register!(M; literal_values, ReaderMapping<LiteralValues, Value>;
    [C, P, B, T, R, S, F, D, G, N, H, I, E, K]; [Missing, C, P, B, T, R, S, F, D, G, N, H, I, E, K]; [M, C, P, B, T, R, S, F, D, G, N, H, I, E, K];
    [scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);

register!(M; scalar_comparisons, ReaderMapping<ScalarComparisons, Value>;
    [L, P, B, T, R, S, F, D, G, N, H, I, E, K]; [L, Missing, P, B, T, R, S, F, D, G, N, H, I, E, K]; [L, M, P, B, T, R, S, F, D, G, N, H, I, E, K];
    [literal_values, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);

register!(M; resolved_places, ReaderMapping<ResolvedPlaces, Place>;
    [L, C, B, T, R, S, F, D, G, N, H, I, E, K]; [L, C, Missing, B, T, R, S, F, D, G, N, H, I, E, K]; [L, C, M, B, T, R, S, F, D, G, N, H, I, E, K];
    [literal_values, scalar_comparisons, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);

register!(M; shared_borrows, ReaderMapping<SharedBorrows, Value>;
    [L, C, P, T, R, S, F, D, G, N, H, I, E, K]; [L, C, P, Missing, T, R, S, F, D, G, N, H, I, E, K]; [L, C, P, M, T, R, S, F, D, G, N, H, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);

register!(M; object_types, ReaderMapping<ObjectTypes, TypePlan>;
    [L, C, P, B, R, S, F, D, G, N, H, I, E, K]; [L, C, P, B, Missing, R, S, F, D, G, N, H, I, E, K]; [L, C, P, B, M, R, S, F, D, G, N, H, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);

register!(M; record_initializers, ReaderMapping<RecordInitializers, Value>;
    [L, C, P, B, T, S, F, D, G, N, H, I, E, K]; [L, C, P, B, T, Missing, S, F, D, G, N, H, I, E, K]; [L, C, P, B, T, M, S, F, D, G, N, H, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);

register!(M; lexical_control, ReaderMapping<LexicalControl, JavaBlock>;
    [L, C, P, B, T, R, F, D, G, N, H, I, E, K]; [L, C, P, B, T, R, Missing, F, D, G, N, H, I, E, K]; [L, C, P, B, T, R, M, F, D, G, N, H, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);

register!(M; entry_signatures, EntryMapping;
    [L, C, P, B, T, R, S, D, G, N, H, I, E, K]; [L, C, P, B, T, R, S, Missing, D, G, N, H, I, E, K]; [L, C, P, B, T, R, S, M, D, G, N, H, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);

impl<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K>
    Builder<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K>
where
    L: ReaderMapping<LiteralValues, Value>,
    C: ReaderMapping<ScalarComparisons, Value>,
    P: ReaderMapping<ResolvedPlaces, Place>,
    B: ReaderMapping<SharedBorrows, Value>,
    T: ReaderMapping<ObjectTypes, TypePlan>,
    R: ReaderMapping<RecordInitializers, Value>,
    S: ReaderMapping<LexicalControl, JavaBlock>,
    F: EntryMapping,
    D: ReaderMapping<DirectCalls, Value>,
    G: FunctionMapping,
    N: ReaderMapping<BooleanNegation, Value>,
    H: ReaderMapping<ShortCircuitBooleans, Value>,
    I: ReaderMapping<IntegerBitwise, Value>,
    E: ReaderMapping<EagerBooleans, Value>,
    K: ReaderMapping<ScalarConstants, Value>,
{
    #[expect(
        clippy::type_complexity,
        reason = "Preserve each executable mapping type at the completed builder boundary"
    )]
    pub(crate) fn build(self) -> Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K> {
        self.0
    }
}
macro_rules! support {
    ($capability:ty, $slot:ident, $field:ident, $bound:path) => {
        impl<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K> Supports<$capability>
            for Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K>
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
support!(LiteralValues, L, literal_values, ReaderMapping<LiteralValues, Value>);
support!(ScalarComparisons, C, scalar_comparisons, ReaderMapping<ScalarComparisons, Value>);
support!(ResolvedPlaces, P, resolved_places, ReaderMapping<ResolvedPlaces, Place>);
support!(SharedBorrows, B, shared_borrows, ReaderMapping<SharedBorrows, Value>);
support!(ObjectTypes, T, object_types, ReaderMapping<ObjectTypes, TypePlan>);
support!(RecordInitializers, R, record_initializers, ReaderMapping<RecordInitializers, Value>);
support!(LexicalControl, S, lexical_control, ReaderMapping<LexicalControl, JavaBlock>);
support!(EntrySignatures, F, entry_signatures, EntryMapping);

pub(crate) trait FunctionMapping:
    for<'tcx> Mapping<Capability = FunctionSignatures, Context<'tcx> = (), Output = JavaMethodSignature>
{
}
impl<M> FunctionMapping for M where
    M: for<'tcx> Mapping<
            Capability = FunctionSignatures,
            Context<'tcx> = (),
            Output = JavaMethodSignature,
        >
{
}
register!(M; direct_calls, ReaderMapping<DirectCalls, Value>;
    [L, C, P, B, T, R, S, F, G, N, H, I, E, K]; [L, C, P, B, T, R, S, F, Missing, G, N, H, I, E, K]; [L, C, P, B, T, R, S, F, M, G, N, H, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);
register!(M; function_signatures, FunctionMapping;
    [L, C, P, B, T, R, S, F, D, N, H, I, E, K]; [L, C, P, B, T, R, S, F, D, Missing, N, H, I, E, K]; [L, C, P, B, T, R, S, F, D, M, N, H, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);
support!(DirectCalls, D, direct_calls, ReaderMapping<DirectCalls, Value>);
support!(FunctionSignatures, G, function_signatures, FunctionMapping);
register!(M; boolean_negation, ReaderMapping<BooleanNegation, Value>;
    [L, C, P, B, T, R, S, F, D, G, H, I, E, K]; [L, C, P, B, T, R, S, F, D, G, Missing, H, I, E, K]; [L, C, P, B, T, R, S, F, D, G, M, H, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants]);
support!(BooleanNegation, N, boolean_negation, ReaderMapping<BooleanNegation, Value>);
register!(M; short_circuit_booleans, ReaderMapping<ShortCircuitBooleans, Value>;
    [L, C, P, B, T, R, S, F, D, G, N, I, E, K]; [L, C, P, B, T, R, S, F, D, G, N, Missing, I, E, K]; [L, C, P, B, T, R, S, F, D, G, N, M, I, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, integer_bitwise, eager_booleans, scalar_constants]);
support!(ShortCircuitBooleans, H, short_circuit_booleans, ReaderMapping<ShortCircuitBooleans, Value>);

register!(M; integer_bitwise, ReaderMapping<IntegerBitwise, Value>;
    [L, C, P, B, T, R, S, F, D, G, N, H, E, K]; [L, C, P, B, T, R, S, F, D, G, N, H, Missing, E, K]; [L, C, P, B, T, R, S, F, D, G, N, H, M, E, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, eager_booleans, scalar_constants]);
support!(IntegerBitwise, I, integer_bitwise, ReaderMapping<IntegerBitwise, Value>);

register!(M; eager_booleans, ReaderMapping<EagerBooleans, Value>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, K]; [L, C, P, B, T, R, S, F, D, G, N, H, I, Missing, K]; [L, C, P, B, T, R, S, F, D, G, N, H, I, M, K];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, scalar_constants]);
support!(EagerBooleans, E, eager_booleans, ReaderMapping<EagerBooleans, Value>);

register!(M; scalar_constants, ReaderMapping<ScalarConstants, Value>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E]; [L, C, P, B, T, R, S, F, D, G, N, H, I, E, Missing]; [L, C, P, B, T, R, S, F, D, G, N, H, I, E, M];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans]);
support!(ScalarConstants, K, scalar_constants, ReaderMapping<ScalarConstants, Value>);
