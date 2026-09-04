//! Shared Java capability registration machinery.

use portable_build::{
    Capability, CapabilityMapping, EmptyCapabilitySlots, LanguageCapabilityPlugin,
    LanguagePluginBuilder, MarkUnsupported, ReplaceMissing, language_plugin,
};
use portable_core_ir::CoreIntrinsicExpr;

use crate::{
    ast::{JavaExpr, JavaType},
    dialect::JavaDialect,
};

pub(crate) mod sealed {
    pub trait JavaCapabilityMapping {}
}

/// A Java mapping admitted by the sealed Java plugin builder.
pub trait JavaCapabilityMapping:
    sealed::JavaCapabilityMapping + CapabilityMapping<JavaDialect> + Copy + Send + Sync
{
}

/// Transparent registration wrapper used to prove that every Java plugin slot
/// is actually invoked. Its recorder is compiled only into this crate's tests.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ObservedJavaMapping<M>(M);

pub const fn observed<M>(mapping: M) -> ObservedJavaMapping<M> {
    ObservedJavaMapping(mapping)
}

impl<M: JavaCapabilityMapping> sealed::JavaCapabilityMapping for ObservedJavaMapping<M> {}
impl<M: JavaCapabilityMapping> JavaCapabilityMapping for ObservedJavaMapping<M> {}

impl<M> CapabilityMapping<JavaDialect> for ObservedJavaMapping<M>
where
    M: JavaCapabilityMapping,
{
    type Capability = M::Capability;
    type Context = M::Context;
    type Input = M::Input;
    type Output = M::Output;
    type Error = M::Error;

    fn lower(
        &self,
        context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        record_java_mapping_invocation::<M::Capability>();
        self.0.lower(context, input)
    }
}

#[cfg(test)]
std::thread_local! {
    static INVOCATION_LEDGER: std::cell::RefCell<std::collections::BTreeSet<&'static str>> =
        const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
}

#[cfg(test)]
fn record_java_mapping_invocation<C: Capability>() {
    INVOCATION_LEDGER.with(|ledger| {
        ledger.borrow_mut().insert(std::any::type_name::<C>());
    });
}

#[cfg(not(test))]
fn record_java_mapping_invocation<C: Capability>() {}

#[cfg(test)]
pub(crate) fn reset_java_mapping_invocations() {
    INVOCATION_LEDGER.with(|ledger| ledger.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn java_mapping_invocations() -> std::collections::BTreeSet<&'static str> {
    INVOCATION_LEDGER.with(|ledger| ledger.borrow().clone())
}

/// Consuming Java-specific wrapper which admits only sealed Java AST mappings.
///
/// A generic mapping whose output is not part of Java's checked AST cannot be
/// smuggled into this builder, even if it names a real portable capability:
///
/// ```compile_fail
/// use portable_backend_java::{
///     capabilities::java_plugin_builder,
///     dialect::JavaDialect,
/// };
/// use portable_build::{CapabilityMapping, I32Values};
/// struct SourceStringMapping;
/// impl CapabilityMapping<JavaDialect> for SourceStringMapping {
///     type Capability = I32Values;
///     type Context = ();
///     type Input = i32;
///     type Output = String;
///     type Error = ();
///     fn lower(&self, _: &mut (), value: i32) -> Result<String, ()> {
///         Ok(value.to_string())
///     }
/// }
/// let _ = java_plugin_builder().support(SourceStringMapping);
/// ```
pub struct JavaPluginBuilder<Slots = EmptyCapabilitySlots> {
    inner: LanguagePluginBuilder<JavaDialect, Slots>,
}

type JavaRegisteredSlots<Slots, M> = <Slots as ReplaceMissing<
    <<M as CapabilityMapping<JavaDialect>>::Capability as Capability>::Index,
    M,
>>::Output;

type JavaUnsupportedSlots<Slots, C> =
    <Slots as MarkUnsupported<<C as Capability>::Index, C>>::Output;

pub fn java_plugin_builder() -> JavaPluginBuilder {
    JavaPluginBuilder {
        inner: language_plugin(JavaDialect),
    }
}

impl<Slots> JavaPluginBuilder<Slots> {
    pub fn support<M>(self, mapping: M) -> JavaPluginBuilder<JavaRegisteredSlots<Slots, M>>
    where
        M: JavaCapabilityMapping,
        Slots: ReplaceMissing<<M::Capability as Capability>::Index, M>,
    {
        JavaPluginBuilder {
            inner: self.inner.support(mapping),
        }
    }

    pub fn unsupported<C>(self) -> JavaPluginBuilder<JavaUnsupportedSlots<Slots, C>>
    where
        C: Capability,
        Slots: MarkUnsupported<C::Index, C>,
    {
        JavaPluginBuilder {
            inner: self.inner.unsupported::<C>(),
        }
    }

    pub fn build(self) -> LanguageCapabilityPlugin<JavaDialect, Slots> {
        self.inner.build()
    }
}

#[doc(hidden)]
pub struct JavaIntrinsicMappingInput<C: Capability> {
    pub(crate) value: CoreIntrinsicExpr<JavaExpr>,
    pub(crate) result: JavaType,
    capability: std::marker::PhantomData<C>,
}

impl<C: Capability> JavaIntrinsicMappingInput<C> {
    pub(crate) fn new(value: CoreIntrinsicExpr<JavaExpr>, result: JavaType) -> Self {
        Self {
            value,
            result,
            capability: std::marker::PhantomData,
        }
    }
}

macro_rules! java_intrinsic_mapping {
    ($mapping:ident, $capability:ty) => {
        #[doc(hidden)]
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $mapping;

        impl crate::capabilities::support::sealed::JavaCapabilityMapping for $mapping {}
        impl crate::capabilities::support::JavaCapabilityMapping for $mapping {}

        impl portable_build::CapabilityMapping<crate::dialect::JavaDialect> for $mapping {
            type Capability = $capability;
            type Context = ();
            type Input = crate::capabilities::support::JavaIntrinsicMappingInput<$capability>;
            type Output = crate::lower::JavaIntrinsicExpr;
            type Error = Vec<portable_diagnostics::Diagnostic>;

            fn lower(
                &self,
                _context: &mut Self::Context,
                input: Self::Input,
            ) -> Result<Self::Output, Self::Error> {
                crate::lower::lower_intrinsic_expression(input.value, input.result)
            }
        }
    };
}

pub(crate) use java_intrinsic_mapping;
