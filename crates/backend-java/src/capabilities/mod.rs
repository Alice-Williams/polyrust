//! Java capability mappings and their exhaustive registration.

use portable_build::{Enums, LanguageCapabilityPlugin, capability_slots};

mod bool_values;
mod boolean_logic;
mod bytes_operations;
mod bytes_values;
mod char_values;
mod checked_integer_arithmetic;
mod checked_integer_shifts;
mod conditionals;
mod constants;
mod dispatch;
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
mod support;
mod text_values;
mod type_aliases;
mod unit_values;
mod utf8_conversions;
mod wrapping_integer_arithmetic;

pub use bool_values::JavaBoolValues;
pub use boolean_logic::JavaBooleanLogic;
pub use bytes_operations::JavaBytesOperations;
pub(crate) use bytes_values::JavaBytesInput;
pub use bytes_values::JavaBytesValues;
pub use char_values::JavaCharValues;
pub use checked_integer_arithmetic::JavaCheckedIntegerArithmetic;
pub use checked_integer_shifts::JavaCheckedIntegerShifts;
pub use conditionals::JavaConditionals;
pub use constants::JavaConstants;
pub use equality::JavaEquality;
pub use f64_values::JavaF64Values;
pub use floating_point_arithmetic::JavaFloatingPointArithmetic;
pub use floating_point_inspection::JavaFloatingPointInspection;
pub use functions::JavaFunctions;
pub use functions::JavaFunctionsNode;
pub use i32_values::JavaI32Values;
pub use i64_values::JavaI64Values;
pub use integer_bitwise::JavaIntegerBitwise;
pub use integer_conversions::JavaIntegerConversions;
pub use interfaces::JavaInterfaces;
pub use list_operations::JavaListOperations;
pub(crate) use list_values::JavaListInput;
pub use list_values::JavaListValues;
pub use local_bindings::JavaLocalBindings;
pub use loops::JavaLoops;
pub use modules::JavaModules;
pub use option_operations::JavaOptionOperations;
pub(crate) use option_values::JavaOptionInput;
pub use option_values::JavaOptionValues;
pub use ordering::JavaOrdering;
pub use pattern_matching::JavaPatternMatching;
pub use portable_tests::JavaPortableTests;
pub use records::JavaRecords;
pub use records::JavaRecordsNode;
pub use result_operations::JavaResultOperations;
pub use result_propagation::JavaResultPropagation;
pub(crate) use result_values::JavaResultInput;
pub use result_values::JavaResultValues;
pub use string_concatenation::JavaStringConcatenation;
pub use string_inspection::JavaStringInspection;
pub use string_transformation::JavaStringTransformation;
pub use support::{JavaCapabilityMapping, JavaPluginBuilder, java_plugin_builder};
pub use text_values::JavaTextValues;
pub use type_aliases::JavaTypeAliases;
pub use unit_values::JavaUnitValues;
pub use utf8_conversions::JavaUtf8Conversions;
pub use wrapping_integer_arithmetic::JavaWrappingIntegerArithmetic;

pub(crate) use conditionals::{
    JavaConditionalValueInput, JavaConditionalsInput, JavaConditionalsNode,
};
pub(crate) use constants::{JavaConstantsInput, JavaConstantsNode};
pub(crate) use dispatch::{JavaIntrinsicFamily, classify_intrinsic};
pub(crate) use functions::{JavaFunctionDeclarationInput, JavaFunctionsInput};
pub(crate) use interfaces::{
    JavaConcreteInterfaceCallInput, JavaInterfaceCallInput, JavaInterfaceDeclarationInput,
    JavaInterfaceImplementationInput, JavaInterfaceMethodInput, JavaInterfacesInput,
    JavaInterfacesNode,
};
pub(crate) use local_bindings::JavaLocalBindingInput;
pub(crate) use loops::JavaLoopsInput;
pub(crate) use modules::JavaModuleInput;
pub(crate) use pattern_matching::{
    JavaLoweredPattern, JavaMatchArmInput, JavaMatchInput, JavaPatternFieldBindingInput,
    JavaPatternInput, JavaPatternMatchPlan, JavaPatternMatchingInput, JavaPatternMatchingNode,
};
pub(crate) use portable_tests::{
    JavaPortableTestCaseInput, JavaPortableTestExpectation, JavaPortableTestHarnessInput,
    JavaPortableTestsInput, JavaPortableTestsNode,
};
pub(crate) use records::{JavaRecordDeclarationInput, JavaRecordsInput};
pub(crate) use result_propagation::{JavaResultPropagationInput, JavaResultPropagationPlan};
pub(crate) use type_aliases::JavaTypeAliasInput;

use crate::dialect::JavaDialect;

pub type JavaCapabilitySlots = capability_slots!(
    implemented JavaFunctions,
    implemented JavaRecords,
    implemented JavaBoolValues,
    implemented JavaI32Values,
    implemented JavaI64Values,
    implemented JavaF64Values,
    implemented JavaTextValues,
    implemented JavaBooleanLogic,
    implemented JavaEquality,
    implemented JavaOrdering,
    implemented JavaCheckedIntegerArithmetic,
    implemented JavaWrappingIntegerArithmetic,
    implemented JavaFloatingPointArithmetic,
    implemented JavaStringConcatenation,
    implemented JavaCharValues,
    implemented JavaBytesValues,
    implemented JavaListValues,
    implemented JavaOptionValues,
    implemented JavaResultValues,
    implemented JavaIntegerBitwise,
    implemented JavaCheckedIntegerShifts,
    implemented JavaFloatingPointInspection,
    implemented JavaStringInspection,
    implemented JavaStringTransformation,
    implemented JavaBytesOperations,
    implemented JavaListOperations,
    implemented JavaOptionOperations,
    implemented JavaResultOperations,
    implemented JavaIntegerConversions,
    implemented JavaUtf8Conversions,
    implemented JavaModules,
    implemented JavaConstants,
    implemented JavaTypeAliases,
    unsupported Enums,
    implemented JavaInterfaces,
    implemented JavaPortableTests,
    implemented JavaLocalBindings,
    implemented JavaConditionals,
    implemented JavaLoops,
    implemented JavaPatternMatching,
    implemented JavaResultPropagation,
    implemented JavaUnitValues,
);

pub type JavaCapabilitySet = LanguageCapabilityPlugin<JavaDialect, JavaCapabilitySlots>;

pub(crate) fn java_capabilities() -> JavaCapabilitySet {
    java_plugin_builder()
        .support(JavaFunctions)
        .support(JavaRecords)
        .support(JavaBoolValues)
        .support(JavaI32Values)
        .support(JavaI64Values)
        .support(JavaF64Values)
        .support(JavaTextValues)
        .support(JavaBooleanLogic)
        .support(JavaEquality)
        .support(JavaOrdering)
        .support(JavaCheckedIntegerArithmetic)
        .support(JavaWrappingIntegerArithmetic)
        .support(JavaFloatingPointArithmetic)
        .support(JavaStringConcatenation)
        .support(JavaCharValues)
        .support(JavaBytesValues)
        .support(JavaListValues)
        .support(JavaOptionValues)
        .support(JavaResultValues)
        .support(JavaIntegerBitwise)
        .support(JavaCheckedIntegerShifts)
        .support(JavaFloatingPointInspection)
        .support(JavaStringInspection)
        .support(JavaStringTransformation)
        .support(JavaBytesOperations)
        .support(JavaListOperations)
        .support(JavaOptionOperations)
        .support(JavaResultOperations)
        .support(JavaIntegerConversions)
        .support(JavaUtf8Conversions)
        .support(JavaModules)
        .support(JavaConstants)
        .support(JavaTypeAliases)
        .unsupported::<Enums>()
        .support(JavaInterfaces)
        .support(JavaPortableTests)
        .support(JavaLocalBindings)
        .support(JavaConditionals)
        .support(JavaLoops)
        .support(JavaPatternMatching)
        .support(JavaResultPropagation)
        .support(JavaUnitValues)
        .build()
}
