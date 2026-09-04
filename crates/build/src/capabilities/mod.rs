//! Closed, target-independent capability catalogue.
//!
//! Each capability marker lives in its own module. This module owns the
//! registry machinery and assigns every marker a stable type-level index.

mod bool_values;
mod boolean_logic;
mod bytes_operations;
mod bytes_values;
mod char_values;
mod checked_integer_arithmetic;
mod checked_integer_shifts;
mod conditionals;
mod constants;
mod enums;
mod equality;
mod f64_values;
mod floating_point_arithmetic;
mod floating_point_inspection;
mod functions;
mod i32_values;
mod i64_values;
mod integer_bitwise;
mod integer_conversions;
mod interfaces;
mod list_operations;
mod list_values;
mod local_bindings;
mod loops;
mod modules;
mod option_operations;
mod option_values;
mod ordering;
mod pattern_matching;
mod portable_tests;
mod records;
mod result_operations;
mod result_propagation;
mod result_values;
mod string_concatenation;
mod string_inspection;
mod string_transformation;
mod text_values;
mod type_aliases;
mod unit_values;
mod utf8_conversions;
mod wrapping_integer_arithmetic;

pub use bool_values::BoolValues;
pub use boolean_logic::BooleanLogic;
pub use bytes_operations::BytesOperations;
pub use bytes_values::BytesValues;
pub use char_values::CharValues;
pub use checked_integer_arithmetic::CheckedIntegerArithmetic;
pub use checked_integer_shifts::CheckedIntegerShifts;
pub use conditionals::Conditionals;
pub use constants::Constants;
pub use enums::Enums;
pub use equality::Equality;
pub use f64_values::F64Values;
pub use floating_point_arithmetic::FloatingPointArithmetic;
pub use floating_point_inspection::FloatingPointInspection;
pub use functions::Functions;
pub use i32_values::I32Values;
pub use i64_values::I64Values;
pub use integer_bitwise::IntegerBitwise;
pub use integer_conversions::IntegerConversions;
pub use interfaces::Interfaces;
pub use list_operations::ListOperations;
pub use list_values::ListValues;
pub use local_bindings::LocalBindings;
pub use loops::Loops;
pub use modules::Modules;
pub use option_operations::OptionOperations;
pub use option_values::OptionValues;
pub use ordering::Ordering;
pub use pattern_matching::PatternMatching;
pub use portable_tests::PortableTests;
pub use records::Records;
pub use result_operations::ResultOperations;
pub use result_propagation::ResultPropagation;
pub use result_values::ResultValues;
pub use string_concatenation::StringConcatenation;
pub use string_inspection::StringInspection;
pub use string_transformation::StringTransformation;
pub use text_values::TextValues;
pub use type_aliases::TypeAliases;
pub use unit_values::UnitValues;
pub use utf8_conversions::Utf8Conversions;
pub use wrapping_integer_arithmetic::WrappingIntegerArithmetic;

use std::marker::PhantomData;

mod sealed {
    pub trait Capability {}
    pub trait Requirements {}
}

/// A capability which can be required by typed portable syntax.
pub trait Capability: sealed::Capability {
    /// Position of this capability in the closed plugin mapping catalogue.
    type Index;
}

/// A structural compile-time tree of inferred requirements.
pub trait Requirements: sealed::Requirements {}

/// The empty requirement tree.
#[derive(Clone, Copy, Debug)]
pub struct NoneRequired;

/// One required capability followed by another requirement tree.
#[derive(Clone, Copy, Debug)]
pub struct Requires<F: Capability, Tail: Requirements = NoneRequired>(PhantomData<(F, Tail)>);

/// The conjunction of two requirement trees.
#[derive(Clone, Copy, Debug)]
pub struct All<Left: Requirements, Right: Requirements>(PhantomData<(Left, Right)>);

impl sealed::Requirements for NoneRequired {}
impl Requirements for NoneRequired {}
impl<F: Capability, Tail: Requirements> sealed::Requirements for Requires<F, Tail> {}
impl<F: Capability, Tail: Requirements> Requirements for Requires<F, Tail> {}
impl<Left: Requirements, Right: Requirements> sealed::Requirements for All<Left, Right> {}
impl<Left: Requirements, Right: Requirements> Requirements for All<Left, Right> {}

/// A typed executable mapping registered by one target dialect.
pub trait CapabilityMapping<D>: 'static {
    type Capability: Capability;
    type Context;
    type Input;
    type Output;
    type Error;

    fn lower(
        &self,
        context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error>;
}

/// Compile-time evidence that a plugin stores one executable capability mapping.
pub trait Supports<F: Capability> {
    type Dialect;
    type Mapping: CapabilityMapping<Self::Dialect, Capability = F>;

    fn mapping(&self) -> &Self::Mapping;
}

/// Compile-time evidence that a dialect implements a complete requirement tree.
pub trait SupportsAll<R: Requirements> {}

impl<D> SupportsAll<NoneRequired> for D {}

impl<D, F, Tail> SupportsAll<Requires<F, Tail>> for D
where
    F: Capability,
    Tail: Requirements,
    D: Supports<F> + SupportsAll<Tail>,
{
}

impl<D, Left, Right> SupportsAll<All<Left, Right>> for D
where
    Left: Requirements,
    Right: Requirements,
    D: SupportsAll<Left> + SupportsAll<Right>,
{
}

/// First position in a type-level capability-slot list.
pub enum Here {}

/// A later position in a type-level capability-slot list.
pub enum There<Index> {
    #[doc(hidden)]
    Never(std::convert::Infallible, PhantomData<Index>),
}

macro_rules! capability_markers_at {
    ($index:ty;) => {};
    ($index:ty; $name:ty $(, $tail:ty)* $(,)?) => {
        impl sealed::Capability for $name {}
        impl Capability for $name {
            type Index = $index;
        }
        capability_markers_at!(There<$index>; $($tail),*);
    };
}

macro_rules! missing_slots {
    () => { CapabilitySlotEnd };
    ($head:ident $(, $tail:ident)* $(,)?) => {
        CapabilitySlots<Missing, missing_slots!($($tail),*)>
    };
}

macro_rules! capability_catalogue {
    ($($name:ident),+ $(,)?) => {
        capability_markers_at!(Here; $($name),+);

        /// Empty mapping state containing one missing slot per portable capability.
        pub type EmptyCapabilitySlots = missing_slots!($($name),+);
    };
}

/// Marker stored in an unregistered capability slot.
#[derive(Clone, Copy, Debug, Default)]
pub struct Missing;

/// A catalogued capability explicitly unavailable in one target plugin.
///
/// Unlike `Implemented<M>`, this slot intentionally does not implement
/// `RegisteredMapping`, so it cannot establish a `Supports<C>` witness.
#[derive(Clone, Copy, Debug, Default)]
pub struct Unsupported<C: Capability>(PhantomData<C>);

impl<C: Capability> Unsupported<C> {
    const fn new() -> Self {
        Self(PhantomData)
    }
}

/// A capability slot containing its executable mapping.
#[derive(Clone, Copy, Debug)]
pub struct Implemented<M>(M);

/// One slot followed by the remaining closed capability catalogue.
#[derive(Clone, Copy, Debug)]
pub struct CapabilitySlots<Head, Tail> {
    head: Head,
    tail: Tail,
}

/// End of the closed capability catalogue.
#[derive(Clone, Copy, Debug, Default)]
pub struct CapabilitySlotEnd;

#[doc(hidden)]
pub trait SlotAt<Index> {
    type Slot;

    fn slot(&self) -> &Self::Slot;
}

impl<Head, Tail> SlotAt<Here> for CapabilitySlots<Head, Tail> {
    type Slot = Head;

    fn slot(&self) -> &Self::Slot {
        &self.head
    }
}

impl<Head, Tail, Index> SlotAt<There<Index>> for CapabilitySlots<Head, Tail>
where
    Tail: SlotAt<Index>,
{
    type Slot = Tail::Slot;

    fn slot(&self) -> &Self::Slot {
        self.tail.slot()
    }
}

#[doc(hidden)]
pub trait ReplaceMissing<Index, Mapping> {
    type Output;

    fn replace(self, mapping: Mapping) -> Self::Output;
}

#[doc(hidden)]
pub trait MarkUnsupported<Index, C: Capability> {
    type Output;

    fn mark_unsupported(self) -> Self::Output;
}

impl<Tail, C: Capability> MarkUnsupported<Here, C> for CapabilitySlots<Missing, Tail> {
    type Output = CapabilitySlots<Unsupported<C>, Tail>;

    fn mark_unsupported(self) -> Self::Output {
        CapabilitySlots {
            head: Unsupported::new(),
            tail: self.tail,
        }
    }
}

impl<Head, Tail, Index, C> MarkUnsupported<There<Index>, C> for CapabilitySlots<Head, Tail>
where
    C: Capability,
    Tail: MarkUnsupported<Index, C>,
{
    type Output = CapabilitySlots<Head, Tail::Output>;

    fn mark_unsupported(self) -> Self::Output {
        CapabilitySlots {
            head: self.head,
            tail: self.tail.mark_unsupported(),
        }
    }
}

impl<Tail, Mapping> ReplaceMissing<Here, Mapping> for CapabilitySlots<Missing, Tail> {
    type Output = CapabilitySlots<Implemented<Mapping>, Tail>;

    fn replace(self, mapping: Mapping) -> Self::Output {
        CapabilitySlots {
            head: Implemented(mapping),
            tail: self.tail,
        }
    }
}

impl<Head, Tail, Index, Mapping> ReplaceMissing<There<Index>, Mapping>
    for CapabilitySlots<Head, Tail>
where
    Tail: ReplaceMissing<Index, Mapping>,
{
    type Output = CapabilitySlots<Head, Tail::Output>;

    fn replace(self, mapping: Mapping) -> Self::Output {
        CapabilitySlots {
            head: self.head,
            tail: self.tail.replace(mapping),
        }
    }
}

#[doc(hidden)]
pub trait RegisteredMapping<D, F: Capability> {
    type Mapping: CapabilityMapping<D, Capability = F>;

    fn mapping(&self) -> &Self::Mapping;
}

impl<D, F, M> RegisteredMapping<D, F> for Implemented<M>
where
    F: Capability,
    M: CapabilityMapping<D, Capability = F>,
{
    type Mapping = M;

    fn mapping(&self) -> &Self::Mapping {
        &self.0
    }
}

/// Consuming builder for a target's executable capability mappings.
pub struct LanguagePluginBuilder<D, Slots = EmptyCapabilitySlots> {
    dialect: D,
    slots: Slots,
}

/// Completed typed capability plugin. Its slot state determines `Supports<C>`.
#[derive(Clone, Copy, Debug)]
pub struct LanguageCapabilityPlugin<D, Slots> {
    dialect: D,
    slots: Slots,
}

type RegisteredSlots<D, Slots, M> = <Slots as ReplaceMissing<
    <<M as CapabilityMapping<D>>::Capability as Capability>::Index,
    M,
>>::Output;

/// Starts an empty executable capability registry for `dialect`.
pub fn language_plugin<D>(dialect: D) -> LanguagePluginBuilder<D> {
    LanguagePluginBuilder {
        dialect,
        slots: empty_capability_slots(),
    }
}

impl<D, Slots> LanguagePluginBuilder<D, Slots> {
    /// Registers the capability inferred from `mapping` exactly once.
    pub fn support<M>(self, mapping: M) -> LanguagePluginBuilder<D, RegisteredSlots<D, Slots, M>>
    where
        M: CapabilityMapping<D>,
        Slots: ReplaceMissing<<M::Capability as Capability>::Index, M>,
    {
        LanguagePluginBuilder {
            dialect: self.dialect,
            slots: self.slots.replace(mapping),
        }
    }

    /// Records an exhaustive, explicit unsupported decision for one capability.
    pub fn unsupported<F>(
        self,
    ) -> LanguagePluginBuilder<D, <Slots as MarkUnsupported<F::Index, F>>::Output>
    where
        F: Capability,
        Slots: MarkUnsupported<F::Index, F>,
    {
        LanguagePluginBuilder {
            dialect: self.dialect,
            slots: self.slots.mark_unsupported(),
        }
    }

    pub fn build(self) -> LanguageCapabilityPlugin<D, Slots> {
        LanguageCapabilityPlugin {
            dialect: self.dialect,
            slots: self.slots,
        }
    }
}

impl<D, Slots> LanguageCapabilityPlugin<D, Slots> {
    pub const fn dialect(&self) -> &D {
        &self.dialect
    }

    pub fn mapping_for<F>(&self) -> &<Self as Supports<F>>::Mapping
    where
        F: Capability,
        Self: Supports<F>,
    {
        <Self as Supports<F>>::mapping(self)
    }
}

impl<D, Slots, F> Supports<F> for LanguageCapabilityPlugin<D, Slots>
where
    F: Capability,
    Slots: SlotAt<F::Index>,
    Slots::Slot: RegisteredMapping<D, F> + 'static,
{
    type Dialect = D;
    type Mapping = <Slots::Slot as RegisteredMapping<D, F>>::Mapping;

    fn mapping(&self) -> &Self::Mapping {
        self.slots.slot().mapping()
    }
}

macro_rules! capability_slots_value {
    () => { CapabilitySlotEnd };
    ($head:ident $(, $tail:ident)* $(,)?) => {
        CapabilitySlots {
            head: Missing,
            tail: capability_slots_value!($($tail),*),
        }
    };
}

/// Builds the concrete slot-state type for a plugin which implements every
/// catalogue capability in catalogue order.
#[macro_export]
macro_rules! implemented_capability_slots {
    () => { $crate::CapabilitySlotEnd };
    ($head:ty $(, $tail:ty)* $(,)?) => {
        $crate::CapabilitySlots<
            $crate::Implemented<$head>,
            $crate::implemented_capability_slots!($($tail),*)
        >
    };
}

/// Builds a complete slot state with explicit implemented/unsupported rows.
#[macro_export]
macro_rules! capability_slots {
    () => { $crate::CapabilitySlotEnd };
    (implemented $head:ty, $($tail:tt)*) => {
        $crate::CapabilitySlots<
            $crate::Implemented<$head>,
            $crate::capability_slots!($($tail)*)
        >
    };
    (unsupported $capability:ty, $($tail:tt)*) => {
        $crate::CapabilitySlots<
            $crate::Unsupported<$capability>,
            $crate::capability_slots!($($tail)*)
        >
    };
}

fn empty_capability_slots() -> EmptyCapabilitySlots {
    capability_slots_value!(
        Functions,
        Records,
        BoolValues,
        I32Values,
        I64Values,
        F64Values,
        TextValues,
        BooleanLogic,
        Equality,
        Ordering,
        CheckedIntegerArithmetic,
        WrappingIntegerArithmetic,
        FloatingPointArithmetic,
        StringConcatenation,
        CharValues,
        BytesValues,
        ListValues,
        OptionValues,
        ResultValues,
        IntegerBitwise,
        CheckedIntegerShifts,
        FloatingPointInspection,
        StringInspection,
        StringTransformation,
        BytesOperations,
        ListOperations,
        OptionOperations,
        ResultOperations,
        IntegerConversions,
        Utf8Conversions,
        Modules,
        Constants,
        TypeAliases,
        Enums,
        Interfaces,
        PortableTests,
        LocalBindings,
        Conditionals,
        Loops,
        PatternMatching,
        ResultPropagation,
        UnitValues,
    )
}

capability_catalogue!(
    Functions,
    Records,
    BoolValues,
    I32Values,
    I64Values,
    F64Values,
    TextValues,
    BooleanLogic,
    Equality,
    Ordering,
    CheckedIntegerArithmetic,
    WrappingIntegerArithmetic,
    FloatingPointArithmetic,
    StringConcatenation,
    CharValues,
    BytesValues,
    ListValues,
    OptionValues,
    ResultValues,
    IntegerBitwise,
    CheckedIntegerShifts,
    FloatingPointInspection,
    StringInspection,
    StringTransformation,
    BytesOperations,
    ListOperations,
    OptionOperations,
    ResultOperations,
    IntegerConversions,
    Utf8Conversions,
    Modules,
    Constants,
    TypeAliases,
    Enums,
    Interfaces,
    PortableTests,
    LocalBindings,
    Conditionals,
    Loops,
    PatternMatching,
    ResultPropagation,
    UnitValues,
);
