//! Consuming registration: absent, duplicate and wrong-capability slots differ.
use super::{
    BooleanNegation, Capability, DirectCalls, EagerBooleans, EntrySignatures, FloatingAbsolute,
    FloatingNaN, FloatingNegation, FloatingTruncation, FunctionSignatures, ImportState,
    IntegerBitwise, LexicalControl, LiteralValues, LocalConstants, Mapping, ObjectTypes,
    PublicConstantImports, PublicConstantReads, PublicConstants, RecordInitializers,
    ResolvedPlaces, ScalarComparisons, ScalarConstants, SharedBorrows, ShortCircuitBooleans,
    Supports, UnitEffects, WrappingNegation,
};
use crate::c_lower::{Reader, package::State};
use portable_backend_c::ast::CObjectRef;
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
pub(crate) struct Bindings<
    L,
    C,
    P,
    B,
    T,
    R,
    S,
    F,
    D,
    G,
    N,
    H,
    I,
    E,
    K,
    Q,
    U,
    V,
    W,
    X,
    Y,
    Z,
    AA,
    AB,
    AC,
> {
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
    local_constants: Q,
    public_constants: U,
    public_constant_reads: V,
    public_constant_imports: W,
    unit_effects: X,
    wrapping_negation: Y,
    floating_negation: Z,
    floating_nan: AA,
    floating_absolute: AB,
    floating_truncation: AC,
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
    Q = Missing,
    U = Missing,
    V = Missing,
    W = Missing,
    X = Missing,
    Y = Missing,
    Z = Missing,
    AA = Missing,
    AB = Missing,
    AC = Missing,
>(Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC>);
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
            local_constants: Missing,
            public_constants: Missing,
            public_constant_reads: Missing,
            public_constant_imports: Missing,
            unit_effects: Missing,
            wrapping_negation: Missing,
            floating_negation: Missing,
            floating_nan: Missing,
            floating_absolute: Missing,
            floating_truncation: Missing,
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
    [C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [Missing, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [M, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);

register!(M; scalar_comparisons, ReaderMapping<ScalarComparisons, CValue>;
    [L, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, Missing, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, M, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);

register!(M; resolved_places, ReaderMapping<ResolvedPlaces, CPlace>;
    [L, C, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, Missing, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, M, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);

register!(M; shared_borrows, ReaderMapping<SharedBorrows, CValue>;
    [L, C, P, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, Missing, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, M, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);

register!(M; object_types, ReaderMapping<ObjectTypes, CObjectType>;
    [L, C, P, B, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, Missing, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, M, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);

register!(M; record_initializers, ReaderMapping<RecordInitializers, CInitializer>;
    [L, C, P, B, T, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, Missing, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, M, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);

register!(M; lexical_control, ReaderMapping<LexicalControl, CBlock>;
    [L, C, P, B, T, R, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, Missing, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, M, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);

register!(M; entry_signatures, EntryMapping;
    [L, C, P, B, T, R, S, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, Missing, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, M, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);

impl<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC>
    Builder<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC>
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
    E: ReaderMapping<EagerBooleans, CValue>,
    K: ReaderMapping<ScalarConstants, CValue>,
    Q: ReaderMapping<LocalConstants, ()>,
    U: DeclarationMapping,
    V: ReaderMapping<PublicConstantReads, CValue>,
    W: ImportMapping,
    X: ReaderMapping<UnitEffects, Vec<portable_backend_c::ast::CStatement>>,
    Y: ReaderMapping<WrappingNegation, CValue>,
    Z: ReaderMapping<FloatingNegation, CValue>,
    AA: ReaderMapping<FloatingNaN, CValue>,
    AB: ReaderMapping<FloatingAbsolute, CValue>,
    AC: ReaderMapping<FloatingTruncation, CValue>,
{
    #[expect(
        clippy::type_complexity,
        reason = "Preserve each executable mapping type at the completed builder boundary"
    )]
    pub(crate) fn build(
        self,
    ) -> Bindings<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC>
    {
        self.0
    }
}
macro_rules! support {
    ($capability:ty, $slot:ident, $field:ident, $bound:path) => {
        impl<L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC>
            Supports<$capability>
            for Bindings<
                L,
                C,
                P,
                B,
                T,
                R,
                S,
                F,
                D,
                G,
                N,
                H,
                I,
                E,
                K,
                Q,
                U,
                V,
                W,
                X,
                Y,
                Z,
                AA,
                AB,
                AC,
            >
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
    [L, C, P, B, T, R, S, F, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, Missing, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, M, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
register!(M; function_signatures, FunctionMapping;
    [L, C, P, B, T, R, S, F, D, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, Missing, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, M, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(DirectCalls, D, direct_calls, ReaderMapping<DirectCalls, CValue>);
support!(FunctionSignatures, G, function_signatures, FunctionMapping);
register!(M; boolean_negation, ReaderMapping<BooleanNegation, CValue>;
    [L, C, P, B, T, R, S, F, D, G, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, Missing, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, M, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(BooleanNegation, N, boolean_negation, ReaderMapping<BooleanNegation, CValue>);
register!(M; short_circuit_booleans, ReaderMapping<ShortCircuitBooleans, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, Missing, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, M, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(ShortCircuitBooleans, H, short_circuit_booleans, ReaderMapping<ShortCircuitBooleans, CValue>);

register!(M; integer_bitwise, ReaderMapping<IntegerBitwise, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, H, Missing, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, H, M, E, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(IntegerBitwise, I, integer_bitwise, ReaderMapping<IntegerBitwise, CValue>);

register!(M; eager_booleans, ReaderMapping<EagerBooleans, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, H, I, Missing, K, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, H, I, M, K, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(EagerBooleans, E, eager_booleans, ReaderMapping<EagerBooleans, CValue>);

register!(M; scalar_constants, ReaderMapping<ScalarConstants, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, H, I, E, Missing, Q, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, H, I, E, M, Q, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(ScalarConstants, K, scalar_constants, ReaderMapping<ScalarConstants, CValue>);

register!(M; local_constants, ReaderMapping<LocalConstants, ()>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Missing, U, V, W, X, Y, Z, AA, AB, AC]; [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, M, U, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(LocalConstants, Q, local_constants, ReaderMapping<LocalConstants, ()>);

pub(crate) trait DeclarationMapping:
    for<'tcx> Mapping<Capability = PublicConstants, Context<'tcx> = State, Output = CObjectRef>
{
}
impl<M> DeclarationMapping for M where
    M: for<'tcx> Mapping<Capability = PublicConstants, Context<'tcx> = State, Output = CObjectRef>
{
}

register!(M; public_constants, DeclarationMapping;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, V, W, X, Y, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, Missing, V, W, X, Y, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, M, V, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(PublicConstants, U, public_constants, DeclarationMapping);

register!(M; public_constant_reads, ReaderMapping<PublicConstantReads, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, W, X, Y, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, Missing, W, X, Y, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, M, W, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(PublicConstantReads, V, public_constant_reads, ReaderMapping<PublicConstantReads, CValue>);

/// Imports require executable registration against an original producer witness.
pub(crate) trait ImportMapping:
    for<'tcx> Mapping<
        Capability = PublicConstantImports,
        Context<'tcx> = ImportState,
        Output = CObjectRef,
    >
{
}
impl<M> ImportMapping for M where
    M: for<'tcx> Mapping<
            Capability = PublicConstantImports,
            Context<'tcx> = ImportState,
            Output = CObjectRef,
        >
{
}
register!(M; public_constant_imports, ImportMapping;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, X, Y, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, Missing, X, Y, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, M, X, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(
    PublicConstantImports,
    W,
    public_constant_imports,
    ImportMapping
);

register!(M; unit_effects, ReaderMapping<UnitEffects, Vec<portable_backend_c::ast::CStatement>>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, Y, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, Missing, Y, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, M, Y, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, wrapping_negation, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(UnitEffects, X, unit_effects, ReaderMapping<UnitEffects, Vec<portable_backend_c::ast::CStatement>>);

register!(M; wrapping_negation, ReaderMapping<WrappingNegation, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Missing, Z, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, M, Z, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, floating_negation, floating_nan, floating_absolute, floating_truncation]);
support!(WrappingNegation, Y, wrapping_negation, ReaderMapping<WrappingNegation, CValue>);

register!(M; floating_negation, ReaderMapping<FloatingNegation, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Missing, AA, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, M, AA, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_nan, floating_absolute, floating_truncation]);
support!(FloatingNegation, Z, floating_negation, ReaderMapping<FloatingNegation, CValue>);

register!(M; floating_nan, ReaderMapping<FloatingNaN, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, Missing, AB, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, M, AB, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_absolute, floating_truncation]);
support!(FloatingNaN, AA, floating_nan, ReaderMapping<FloatingNaN, CValue>);

register!(M; floating_absolute, ReaderMapping<FloatingAbsolute, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, Missing, AC];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, M, AC];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_truncation]);
support!(FloatingAbsolute, AB, floating_absolute, ReaderMapping<FloatingAbsolute, CValue>);

register!(M; floating_truncation, ReaderMapping<FloatingTruncation, CValue>;
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, Missing];
    [L, C, P, B, T, R, S, F, D, G, N, H, I, E, K, Q, U, V, W, X, Y, Z, AA, AB, M];
    [literal_values, scalar_comparisons, resolved_places, shared_borrows, object_types, record_initializers, lexical_control, entry_signatures, direct_calls, function_signatures, boolean_negation, short_circuit_booleans, integer_bitwise, eager_booleans, scalar_constants, local_constants, public_constants, public_constant_reads, public_constant_imports, unit_effects, wrapping_negation, floating_negation, floating_nan, floating_absolute]);
support!(FloatingTruncation, AC, floating_truncation, ReaderMapping<FloatingTruncation, CValue>);
