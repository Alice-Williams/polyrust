//! Java dialect: catalogue.

use super::known_callables::JavaKnownCallable;
use super::known_constructors::JavaKnownConstructor;
use super::known_fields::JavaKnownField;
use super::known_methods::JavaKnownMethod;
use super::member_names::JavaMemberName;
use super::runtime_callables::JavaRuntimeCallable;
use super::runtime_helpers::{JavaHelperCapability, JavaRuntimeHelper};
use super::{
    JavaDialect, JavaImportKind, JavaInvocationKind, JavaNamespace, JavaPreludeSymbol,
    JavaQualifiedName, JavaStandardLibrary,
};
use crate::ast::{JavaFilePlacement, JavaIdentifier, JavaKnownType, JavaVisibility};
use portable_codegen::{
    CallablePattern, DependencyPolicy, FailureBehavior, KnownCallableSpec, KnownConstructorSpec,
    KnownFieldSpec, KnownMethodSpec, KnownTypeSpec, RuntimeCallableSpec, RuntimeHelperSpec,
    SymbolCatalogue, SymbolOrigin, TargetCallableSignature, TargetEffect, TypeParameterSpec,
    TypePattern,
};
use portable_diagnostics::SourceRef;
use std::collections::BTreeSet;

pub(super) fn java_symbol_catalogue() -> SymbolCatalogue<JavaDialect> {
    SymbolCatalogue {
        types: JavaKnownType::ALL
            .into_iter()
            .map(known_type_spec)
            .collect(),
        callables: JavaKnownCallable::ALL
            .into_iter()
            .map(known_callable_spec)
            .collect(),
        runtime_callables: JavaRuntimeCallable::ALL
            .into_iter()
            .map(runtime_callable_spec)
            .collect(),
        fields: [
            JavaKnownField::IntegerMinValue,
            JavaKnownField::IntegerMaxValue,
            JavaKnownField::LongMinValue,
            JavaKnownField::LongMaxValue,
            JavaKnownField::StandardCharsetsUtf8,
            JavaKnownField::CodingErrorReport,
        ]
        .into_iter()
        .map(known_field_spec)
        .collect(),
        constructors: JavaKnownConstructor::ALL
            .into_iter()
            .map(known_constructor_spec)
            .collect(),
        methods: JavaKnownMethod::ALL
            .into_iter()
            .map(known_method_spec)
            .collect(),
        helpers: JavaRuntimeHelper::ALL
            .into_iter()
            .enumerate()
            .map(|(order, helper)| RuntimeHelperSpec {
                id: helper,
                capability: helper_capability(helper),
                order: u32::try_from(order).expect("Java helper inventory fits u32"),
                name: JavaIdentifier::from_portable(helper.name()),
                alias_stem: helper.name().replace('.', "_"),
                namespace: JavaNamespace::Value,
                items: crate::runtime::helper_items(helper),
                placement: JavaFilePlacement::Runtime,
                visibility: JavaVisibility::Private,
                source: symbol_source("helper", helper.name()),
            })
            .collect(),
    }
}

pub(super) fn known_type_spec(value: JavaKnownType) -> KnownTypeSpec<JavaDialect> {
    let (origin, policy, qualified_name) = if value.implicit() {
        (
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang),
            DependencyPolicy::Implicit,
            None,
        )
    } else if value.runtime_nested() {
        (
            SymbolOrigin::Runtime(
                value
                    .runtime_helper()
                    .expect("runtime nested type owns helper"),
            ),
            DependencyPolicy::Qualified,
            Some(JavaQualifiedName::Type(value)),
        )
    } else {
        (
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21),
            DependencyPolicy::Import(JavaImportKind::Type(value)),
            Some(JavaQualifiedName::Type(value)),
        )
    };
    KnownTypeSpec {
        symbol: value,
        name: JavaIdentifier::from_portable(value.simple_name()),
        alias_stem: value.simple_name().to_owned(),
        qualified_name,
        origin,
        arity: match value {
            JavaKnownType::ArrayList
            | JavaKnownType::List
            | JavaKnownType::RuntimeResult
            | JavaKnownType::RuntimeOption => 1,
            JavaKnownType::Map | JavaKnownType::RuntimeValueResult => 2,
            _ => 0,
        },
        policy,
        dependency: None,
        source: symbol_source("type", value.qualified_name()),
    }
}

fn known_callable_spec(value: JavaKnownCallable) -> KnownCallableSpec<JavaDialect> {
    let signature = value.signature();
    let member = match value {
        JavaKnownCallable::ObjectsDeepEquals => JavaMemberName::DeepEquals,
        JavaKnownCallable::ObjectsRequireNonNull => JavaMemberName::RequireNonNull,
        JavaKnownCallable::DoubleToRawLongBits => JavaMemberName::DoubleToRawLongBits,
        JavaKnownCallable::DoubleFromLongBits => JavaMemberName::LongBitsToDouble,
        JavaKnownCallable::DoubleIsNaN => JavaMemberName::IsNaN,
        JavaKnownCallable::MathFloor => JavaMemberName::Floor,
        JavaKnownCallable::MathCeil => JavaMemberName::Ceil,
        JavaKnownCallable::ListCopyOf => JavaMemberName::CopyOf,
        JavaKnownCallable::ListOf => JavaMemberName::Of,
        JavaKnownCallable::BigIntegerValueOf => JavaMemberName::ValueOf,
        JavaKnownCallable::ByteToUnsignedInt => JavaMemberName::ToUnsignedInt,
        JavaKnownCallable::ByteBufferWrap => JavaMemberName::Wrap,
        JavaKnownCallable::CharacterIsHighSurrogate => JavaMemberName::IsHighSurrogate,
        JavaKnownCallable::CharacterIsLowSurrogate => JavaMemberName::IsLowSurrogate,
        JavaKnownCallable::CharacterCharCount => JavaMemberName::CharCount,
    };
    KnownCallableSpec {
        symbol: value,
        owner: Some(value.owner()),
        name: JavaIdentifier::from_portable(value.name()),
        alias_stem: value.name().to_owned(),
        qualified_name: Some(JavaQualifiedName::Callable(value)),
        origin: if value.owner().implicit() {
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang)
        } else {
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21)
        },
        signature: callable_pattern(&JavaDialect.coarse_signature(&signature)),
        visibility: JavaVisibility::Public,
        policy: DependencyPolicy::Member {
            owner: JavaQualifiedName::Type(value.owner()),
            member,
        },
        dependency: None,
        source: symbol_source("callable", value.qualified_name()),
    }
}

fn runtime_callable_spec(value: JavaRuntimeCallable) -> RuntimeCallableSpec<JavaDialect> {
    let signature = JavaDialect.coarse_signature(&value.signature());
    RuntimeCallableSpec {
        symbol: value,
        name: JavaIdentifier::from_portable(value.name()),
        alias_stem: value.name().to_owned(),
        qualified_name: Some(JavaQualifiedName::RuntimeCallable(value)),
        origin: SymbolOrigin::Runtime(value.helper()),
        signature: callable_pattern(&signature),
        policy: DependencyPolicy::Qualified,
        dependency: None,
        source: symbol_source("runtime-callable", value.name()),
    }
}

fn known_field_spec(value: JavaKnownField) -> KnownFieldSpec<JavaDialect> {
    KnownFieldSpec {
        symbol: value,
        owner: value.owner(),
        name: JavaIdentifier::from_portable(value.member().text()),
        origin: if value.owner().implicit() {
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang)
        } else {
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21)
        },
        ty: TypePattern::Exact(JavaDialect.registered_type(&value.ty())),
        policy: DependencyPolicy::Member {
            owner: JavaQualifiedName::Type(value.owner()),
            member: value.member(),
        },
        dependency: None,
        source: symbol_source("field", value.member().text()),
    }
}

fn known_constructor_spec(value: JavaKnownConstructor) -> KnownConstructorSpec<JavaDialect> {
    let (owner, parameters) = value.signature();
    let signature = TargetCallableSignature {
        invocation: JavaInvocationKind::Constructor,
        receiver: None,
        parameters: parameters
            .iter()
            .map(|value| JavaDialect.registered_type(value))
            .collect(),
        return_type: JavaDialect.registered_type(&owner),
    };
    let owner_type = value.owner();
    KnownConstructorSpec {
        symbol: value,
        owner: owner_type,
        name: JavaIdentifier::from_portable(owner_type.simple_name()),
        alias_stem: owner_type.simple_name().to_owned(),
        qualified_name: Some(JavaQualifiedName::Type(owner_type)),
        origin: if owner_type.runtime_nested() {
            SymbolOrigin::Runtime(
                owner_type
                    .runtime_helper()
                    .expect("runtime constructor owns helper"),
            )
        } else if owner_type.implicit() {
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang)
        } else {
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21)
        },
        signature: callable_pattern(&signature),
        visibility: JavaVisibility::Public,
        policy: if owner_type.runtime_nested() {
            DependencyPolicy::Qualified
        } else if owner_type.implicit() {
            DependencyPolicy::Implicit
        } else {
            DependencyPolicy::Import(JavaImportKind::Type(owner_type))
        },
        dependency: None,
        source: symbol_source("constructor", owner_type.qualified_name()),
    }
}

fn known_method_spec(value: JavaKnownMethod) -> KnownMethodSpec<JavaDialect> {
    let signature = value.signature();
    KnownMethodSpec {
        symbol: value,
        owner: value.owner(),
        name: JavaIdentifier::from_portable(value.name().text()),
        origin: if value.owner().implicit() {
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang)
        } else {
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21)
        },
        signature: callable_pattern(&JavaDialect.coarse_signature(&signature)),
        visibility: JavaVisibility::Public,
        policy: DependencyPolicy::Member {
            owner: JavaQualifiedName::Type(value.owner()),
            member: value.name(),
        },
        dependency: None,
        source: symbol_source("method", value.name().text()),
    }
}

fn callable_pattern(
    signature: &TargetCallableSignature<JavaDialect>,
) -> CallablePattern<JavaDialect> {
    CallablePattern {
        invocation: signature.invocation,
        type_parameters: Vec::<TypeParameterSpec<JavaDialect>>::new(),
        receiver: signature.receiver.clone().map(TypePattern::Exact),
        parameters: signature
            .parameters
            .iter()
            .cloned()
            .map(TypePattern::Exact)
            .collect(),
        result: TypePattern::Exact(signature.return_type.clone()),
        failure: FailureBehavior::Infallible,
        effects: BTreeSet::<TargetEffect>::new(),
    }
}

fn helper_capability(helper: JavaRuntimeHelper) -> JavaHelperCapability {
    match helper {
        JavaRuntimeHelper::Core => JavaHelperCapability::Failures,
        JavaRuntimeHelper::TaggedValues => JavaHelperCapability::TaggedValues,
        JavaRuntimeHelper::CheckedIntegers => JavaHelperCapability::CheckedArithmetic,
        JavaRuntimeHelper::FloatBits => JavaHelperCapability::ExactFloatBits,
        JavaRuntimeHelper::Unicode => JavaHelperCapability::UnicodeScalars,
        JavaRuntimeHelper::Bytes => JavaHelperCapability::ImmutableBytes,
        JavaRuntimeHelper::ImmutableLists => JavaHelperCapability::ImmutableLists,
        JavaRuntimeHelper::StringOperations => JavaHelperCapability::StringOperations,
        JavaRuntimeHelper::Interfaces => JavaHelperCapability::InterfaceDispatch,
    }
}

fn symbol_source(category: &str, name: &str) -> SourceRef {
    SourceRef::logical(["java-catalogue", category, name])
}
