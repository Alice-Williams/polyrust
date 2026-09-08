use super::JavaDialect;
use super::catalogue::{java_symbol_catalogue, known_type_spec};
use super::file_checks::verify_java_file_identity;
use super::known_callables::JavaKnownCallable;
use super::known_constructors::JavaKnownConstructor;
use super::known_methods::JavaKnownMethod;
use super::runtime_callables::JavaRuntimeCallable;
use super::runtime_helpers::JavaRuntimeHelper;
use crate::ast::{
    JavaBlock, JavaCompileFailField, JavaConstructor, JavaDeclarationKind, JavaExpr, JavaField,
    JavaFileItem, JavaFilePlacement, JavaHeritage, JavaIdentifier, JavaKnownType, JavaLiteral,
    JavaMember, JavaMethodSignature, JavaModifier, JavaPackage, JavaPrimitive, JavaSourceFileKind,
    JavaType, JavaTypeDeclaration, JavaVisibility,
};
use portable_codegen::{DependencyPolicy, TargetFile};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use std::collections::BTreeSet;

fn source(label: &str) -> SourceRef {
    SourceRef::logical(["java-verifier-test", label])
}

fn declaration(heritage: JavaHeritage, members: Vec<JavaMember>) -> JavaTypeDeclaration {
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("Fixture"),
        type_parameters: vec![],
        record_components: vec![],
        heritage,
        permits: vec![],
        members,
    }
}

fn verify_file_at_path(
    path: &str,
    declaration: JavaTypeDeclaration,
    role: portable_codegen::SourceRole,
    placement: JavaFilePlacement,
    group_role: portable_codegen::FileGroupRole,
) -> Result<(), Vec<Diagnostic>> {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let file = builder.file(TargetFile::new(
        portable_codegen::RelativeOutputPath::new(path).unwrap(),
        role,
        JavaPackage::Generated,
        placement,
        vec![JavaFileItem::Type {
            conformances: crate::ast::JavaConformanceInventory::structural().into(),
            declared: vec![],
            declaration,
        }],
        JavaSourceFileKind::CompilationUnit,
        source("file"),
    ));
    builder.group(portable_codegen::TargetFileGroup::new(
        group_role,
        vec![portable_codegen::TargetFileMember::Source(file)],
        source("group"),
    ));
    portable_codegen::verify_target_ast(&builder.build())
}

fn verify_single_file(
    declaration: JavaTypeDeclaration,
    role: portable_codegen::SourceRole,
    placement: JavaFilePlacement,
    group_role: portable_codegen::FileGroupRole,
) -> Result<(), Vec<Diagnostic>> {
    verify_file_at_path(
        "src/main/java/org/polyrust/generated/Fixture.java",
        declaration,
        role,
        placement,
        group_role,
    )
}

#[test]
fn closed_catalogues_have_unique_qualified_names() {
    assert_eq!(known_type_spec(JavaKnownType::LinkedHashMap).arity, 2);
    let type_names = JavaKnownType::ALL
        .into_iter()
        .map(JavaKnownType::qualified_name)
        .collect::<BTreeSet<_>>();
    assert_eq!(type_names.len(), JavaKnownType::ALL.len());

    let callable_names = JavaKnownCallable::ALL
        .into_iter()
        .map(JavaKnownCallable::qualified_name)
        .collect::<BTreeSet<_>>();
    assert_eq!(callable_names.len(), JavaKnownCallable::ALL.len());

    let runtime_names = JavaRuntimeCallable::ALL
        .into_iter()
        .map(JavaRuntimeCallable::qualified_name)
        .collect::<BTreeSet<_>>();
    assert_eq!(runtime_names.len(), JavaRuntimeCallable::ALL.len());
}

#[test]
fn every_callable_accepts_its_authoritative_signature_and_rejects_mutation() {
    for callable in JavaKnownCallable::ALL {
        let signature = callable.signature();
        assert!(callable.accepts(&signature), "{callable:?}");
        let mut invalid = signature;
        invalid.pure = !invalid.pure;
        assert!(!callable.accepts(&invalid), "{callable:?}");
    }
    for method in JavaKnownMethod::ALL {
        let signature = method.signature();
        assert!(method.accepts(&signature), "{method:?}");
        let mut invalid = signature;
        invalid.nullable_result = !invalid.nullable_result;
        assert!(!method.accepts(&invalid), "{method:?}");
    }
    for callable in JavaRuntimeCallable::ALL {
        let signature = callable.signature();
        assert!(callable.accepts(&signature), "{callable:?}");
        let mut invalid = signature;
        invalid.receiver = Some(JavaType::known(JavaKnownType::Object));
        assert!(!callable.accepts(&invalid), "{callable:?}");
    }
}

#[test]
fn known_method_signatures_distinguish_receiver_parameter_and_result_conversions() {
    let wildcard_list = JavaType::generic(
        JavaKnownType::List,
        vec![JavaType::Wildcard { bound: None }],
    );
    assert!(JavaKnownMethod::ListGet.accepts(&JavaMethodSignature {
        receiver: Some(wildcard_list),
        parameters: vec![JavaType::primitive(JavaPrimitive::Int)],
        result: JavaType::known(JavaKnownType::Object),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    }));
    assert!(
        !JavaKnownMethod::ObjectEquals.accepts(&JavaMethodSignature {
            receiver: Some(JavaType::primitive(JavaPrimitive::Int)),
            parameters: vec![JavaType::primitive(JavaPrimitive::Int)],
            result: JavaType::primitive(JavaPrimitive::Boolean),
            checked_exceptions: vec![],
            nullable_result: false,
            pure: true,
        })
    );
    assert!(
        !JavaKnownMethod::StringLength.accepts(&JavaMethodSignature {
            receiver: Some(JavaType::known(JavaKnownType::String)),
            parameters: vec![],
            result: JavaType::primitive(JavaPrimitive::Int).boxed(),
            checked_exceptions: vec![],
            nullable_result: false,
            pure: true,
        })
    );
}

#[test]
fn generic_runtime_results_allow_only_matching_unboxing() {
    let option_long = JavaType::generic(
        JavaKnownType::RuntimeOption,
        vec![JavaType::Boxed(JavaPrimitive::Long)],
    );
    let signature = |result| JavaMethodSignature {
        receiver: None,
        parameters: vec![option_long.clone()],
        result,
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };

    assert!(
        JavaRuntimeCallable::OptionValue
            .accepts(&signature(JavaType::primitive(JavaPrimitive::Long),))
    );
    assert!(
        !JavaRuntimeCallable::OptionValue
            .accepts(&signature(JavaType::primitive(JavaPrimitive::Int),))
    );
}

#[test]
fn every_constructor_accepts_its_generic_signature_and_rejects_wrong_arity() {
    for constructor in JavaKnownConstructor::ALL {
        let (owner, parameters) = constructor.signature();
        assert!(constructor.accepts(&owner, &parameters), "{constructor:?}");
        let mut invalid = parameters;
        invalid.push(JavaType::known(JavaKnownType::Object));
        assert!(!constructor.accepts(&owner, &invalid), "{constructor:?}");
    }
}

#[test]
fn catalogue_inventory_and_dependency_policies_are_exhaustive() {
    let catalogue = java_symbol_catalogue();
    assert_eq!(catalogue.types.len(), JavaKnownType::ALL.len());
    assert_eq!(catalogue.callables.len(), JavaKnownCallable::ALL.len());
    assert_eq!(
        catalogue.runtime_callables.len(),
        JavaRuntimeCallable::ALL.len()
    );
    assert_eq!(
        catalogue.constructors.len(),
        JavaKnownConstructor::ALL.len()
    );
    assert_eq!(catalogue.methods.len(), JavaKnownMethod::ALL.len());
    assert_eq!(catalogue.helpers.len(), JavaRuntimeHelper::ALL.len());

    for ty in JavaKnownType::ALL {
        let policy = known_type_spec(ty).policy;
        match (ty.implicit(), ty.runtime_nested(), policy) {
            (true, false, DependencyPolicy::Implicit)
            | (false, true, DependencyPolicy::Qualified)
            | (false, false, DependencyPolicy::Import(_)) => {}
            combination => panic!("invalid dependency policy: {combination:?}"),
        }
    }
    for helper in JavaRuntimeHelper::ALL {
        assert!(
            !crate::runtime::helper_items(helper).is_empty(),
            "{helper:?}"
        );
    }
}

#[test]
fn negative_nodes_and_heritage_exceptions_are_confined_and_fail_closed() {
    let compile_fail = JavaMember::CompileFailField(JavaCompileFailField {
        modifiers: vec![JavaModifier::Final],
        expected_type: JavaType::generic(
            JavaKnownType::RuntimeOption,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        ),
        name: JavaIdentifier::from_portable("invalid"),
        initializer: JavaExpr::literal(
            JavaType::known(JavaKnownType::String),
            JavaLiteral::String("missing".to_owned()),
        ),
    });
    let diagnostics = verify_single_file(
        declaration(JavaHeritage::None, vec![compile_fail]),
        portable_codegen::SourceRole::PublicApi,
        JavaFilePlacement::Main,
        portable_codegen::FileGroupRole::PublicApi,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::InvalidStructure)
    );

    let diagnostics = verify_single_file(
        declaration(JavaHeritage::None, vec![]),
        portable_codegen::SourceRole::NegativeTest,
        JavaFilePlacement::NegativeTest,
        portable_codegen::FileGroupRole::NegativeTests,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::InvalidStructure)
    );

    let diagnostics = verify_single_file(
        declaration(
            JavaHeritage::Interfaces(vec![JavaType::known(JavaKnownType::String)]),
            vec![],
        ),
        portable_codegen::SourceRole::PublicApi,
        JavaFilePlacement::Main,
        portable_codegen::FileGroupRole::PublicApi,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::InterfaceNonconformance })
    );
}

#[test]
fn linked_runtime_members_are_verified_in_the_combined_class() {
    let shell = JavaFileItem::Type {
        conformances: crate::ast::JavaConformanceInventory::structural().into(),
        declared: vec![],
        declaration: JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Runtime"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Private],
                name: JavaIdentifier::from_portable("Runtime"),
                parameters: vec![],
                body: JavaBlock::new(vec![]),
            })],
        },
    };
    let fragment = JavaFileItem::RuntimeMembers {
        helper: JavaRuntimeHelper::Interfaces,
        members: vec![JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Private, JavaModifier::Final],
            ty: JavaType::primitive(JavaPrimitive::Int),
            name: JavaIdentifier::from_portable("x"),
            initializer: None,
        })],
    };
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let file = builder.file(TargetFile::new(
        portable_codegen::RelativeOutputPath::new(
            "src/main/java/org/polyrust/generated/Runtime.java",
        )
        .unwrap(),
        portable_codegen::SourceRole::Runtime,
        JavaPackage::Generated,
        JavaFilePlacement::Runtime,
        vec![shell.clone(), fragment.clone()],
        JavaSourceFileKind::CompilationUnit,
        source("combined-runtime"),
    ));
    builder.group(portable_codegen::TargetFileGroup::new(
        portable_codegen::FileGroupRole::Runtime,
        vec![portable_codegen::TargetFileMember::Source(file)],
        source("combined-runtime-group"),
    ));
    let diagnostics = portable_codegen::verify_target_ast(&builder.build()).unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidControlFlow
            && diagnostic
                .message
                .contains("without assigning blank final field `x`")
    }));

    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let file = builder.file(TargetFile::new(
        portable_codegen::RelativeOutputPath::new("Fixture.java").unwrap(),
        portable_codegen::SourceRole::PublicApi,
        JavaPackage::Generated,
        JavaFilePlacement::Main,
        vec![fragment],
        JavaSourceFileKind::CompilationUnit,
        source("misplaced-runtime-fragment"),
    ));
    builder.group(portable_codegen::TargetFileGroup::new(
        portable_codegen::FileGroupRole::PublicApi,
        vec![portable_codegen::TargetFileMember::Source(file)],
        source("misplaced-runtime-fragment-group"),
    ));
    let diagnostics = portable_codegen::verify_target_ast(&builder.build()).unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("confined to the runtime file")
    }));

    let duplicate = |helper| JavaFileItem::RuntimeMembers {
        helper,
        members: vec![JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Private, JavaModifier::Static],
            ty: JavaType::primitive(JavaPrimitive::Int),
            name: JavaIdentifier::from_portable("duplicate"),
            initializer: None,
        })],
    };
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let file = builder.file(TargetFile::new(
        portable_codegen::RelativeOutputPath::new(
            "src/main/java/org/polyrust/generated/Runtime.java",
        )
        .unwrap(),
        portable_codegen::SourceRole::Runtime,
        JavaPackage::Generated,
        JavaFilePlacement::Runtime,
        vec![
            shell,
            duplicate(JavaRuntimeHelper::Core),
            duplicate(JavaRuntimeHelper::Interfaces),
        ],
        JavaSourceFileKind::CompilationUnit,
        source("duplicate-runtime-members"),
    ));
    builder.group(portable_codegen::TargetFileGroup::new(
        portable_codegen::FileGroupRole::Runtime,
        vec![portable_codegen::TargetFileMember::Source(file)],
        source("duplicate-runtime-members-group"),
    ));
    let diagnostics = portable_codegen::verify_target_ast(&builder.build()).unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::DuplicateDeclaration
            && diagnostic
                .message
                .contains("field conflicts with another field")
    }));
}

#[test]
fn public_top_level_type_must_match_its_java_filename() {
    let mut wrong = declaration(JavaHeritage::None, vec![]);
    wrong.visibility = JavaVisibility::Public;
    wrong.name = JavaIdentifier::from_portable("Wrong");
    let diagnostics = verify_file_at_path(
        "Other.java",
        wrong,
        portable_codegen::SourceRole::PublicApi,
        JavaFilePlacement::Main,
        portable_codegen::FileGroupRole::PublicApi,
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic
                .message
                .contains("must be declared in `Wrong.java`")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::UnsafeOutputPath
            && diagnostic.message.contains("canonical package directory")
    }));

    let diagnostics = verify_java_file_identity(
        portable_codegen::SourceRole::PublicApi,
        "src/test/java/org/polyrust/generated/Fixture.java",
        &JavaPackage::Generated,
        &JavaFilePlacement::NativeTest,
        false,
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("role and file placement")
    }));

    let mut forged_runtime = declaration(JavaHeritage::None, vec![]);
    forged_runtime.visibility = JavaVisibility::Public;
    forged_runtime.name = JavaIdentifier::from_portable("Runtime");
    let diagnostics = verify_file_at_path(
        "src/main/java/org/polyrust/generated/Runtime.java",
        forged_runtime,
        portable_codegen::SourceRole::PublicApi,
        JavaFilePlacement::Main,
        portable_codegen::FileGroupRole::PublicApi,
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("exact runtime role")
    }));
    assert!(
        verify_java_file_identity(
            portable_codegen::SourceRole::Runtime,
            "src/main/java/org/polyrust/generated/Runtime.java",
            &JavaPackage::Generated,
            &JavaFilePlacement::Runtime,
            true,
        )
        .is_empty()
    );
}
