//! Exact capacity matrices and resource-only typed error boundary.

use super::{JavaResourceError, check, types};
use crate::ast::{
    JavaBlock, JavaDeclarationKind, JavaFileItem, JavaHeritage, JavaIdentifier, JavaMember,
    JavaMethod, JavaMethodDeclaration, JavaModifier, JavaParameter, JavaPrimitive, JavaType,
    JavaTypeDeclaration, JavaVisibility,
};
use portable_codegen::{TypedGenerationError, TypedPipelineStage};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

pub(super) fn method(ty: JavaType, count: usize, instance: bool) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: if instance {
            vec![JavaModifier::Public]
        } else {
            vec![JavaModifier::Public, JavaModifier::Static]
        },
        type_parameters: vec![],
        return_type: JavaType::Primitive(JavaPrimitive::Void),
        name: JavaIdentifier::new("method").unwrap(),
        parameters: (0..count)
            .map(|index| JavaParameter {
                ty: ty.clone(),
                name: JavaIdentifier::new(format!("p{index}")).unwrap(),
                final_parameter: true,
            })
            .collect(),
        body: Some(JavaBlock::new(vec![])),
    })
}

pub(super) fn declaration(members: Vec<JavaMember>) -> JavaTypeDeclaration {
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::new("Fixture").unwrap(),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members,
    }
}

pub(super) fn item(declaration: JavaTypeDeclaration) -> JavaFileItem {
    JavaFileItem::Type {
        declared: vec![],
        conformances: crate::ast::JavaConformanceInventory::structural().into(),
        declaration,
    }
}

#[test]
fn parameter_slot_boundaries_distinguish_primitives_boxes_and_receivers() {
    for (ty, count, instance, accepted) in [
        (JavaType::Primitive(JavaPrimitive::Int), 255, false, true),
        (JavaType::Primitive(JavaPrimitive::Int), 256, false, false),
        (JavaType::Primitive(JavaPrimitive::Int), 254, true, true),
        (JavaType::Primitive(JavaPrimitive::Int), 255, true, false),
        (JavaType::Primitive(JavaPrimitive::Long), 127, false, true),
        (JavaType::Primitive(JavaPrimitive::Long), 128, false, false),
        (JavaType::Primitive(JavaPrimitive::Double), 127, true, true),
        (JavaType::Primitive(JavaPrimitive::Double), 128, true, false),
        (JavaType::Boxed(JavaPrimitive::Long), 255, false, true),
        (JavaType::Boxed(JavaPrimitive::Double), 254, true, true),
    ] {
        let file = item(declaration(vec![method(ty.clone(), count, instance)]));
        let result = check(vec![("Fixture.java", vec![&file])]);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{ty:?}/{count}/{instance}: {result:?}"
        );
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .all(|error| error.code == DiagnosticCode::TargetResourceLimit)
            );
            assert!(
                errors
                    .iter()
                    .any(|error| error.message.contains("parameter slots"))
            );
        }
    }
}

#[test]
fn names_and_nested_binary_paths_have_separate_budgets() {
    for (length, accepted) in [(65_535, true), (65_536, false)] {
        let JavaMember::Method(mut value) =
            method(JavaType::Primitive(JavaPrimitive::Int), 0, false)
        else {
            unreachable!()
        };
        value.name = JavaIdentifier::new("m".repeat(length)).unwrap();
        let file = item(declaration(vec![JavaMember::Method(value)]));
        assert_eq!(check(vec![("Fixture.java", vec![&file])]).is_ok(), accepted);
    }
    let mut child = declaration(vec![]);
    child.name = JavaIdentifier::new("N".repeat(65_520)).unwrap();
    let file = item(declaration(vec![JavaMember::NestedType(child)]));
    let errors = check(vec![("Fixture.java", vec![&file])]).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("binary class name bytes"))
    );
}

#[test]
fn descriptor_aggregation_and_array_depth_use_classfile_encoding() {
    let mut errors = vec![];
    let mut names = super::Names::new();
    let mut builder = portable_codegen::TargetAstBuilder::new(crate::dialect::JavaDialect);
    let id = builder.generated_type(portable_codegen::GeneratedType {
        name: "LongName".to_owned(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: SourceRef::logical(["resource-test"]),
    });
    names.insert(id, 65_534);
    // A locally spelled type still has a package/enclosing binary path. Use
    // the final member spelling, not its shorter pre-allocation preference.
    let mut nominal = declaration(vec![]);
    nominal.declared = Some(id);
    let mut allocated = super::Names::new();
    let local = super::Names::from([(id, 65_530)]);
    super::collect_names(&nominal, 23, &local, &mut allocated);
    assert_eq!(allocated[&id], 65_553);
    allocated.insert(id, 65_540);
    super::collect_names(&nominal, 23, &local, &mut allocated);
    assert_eq!(
        allocated[&id], 65_540,
        "authoritative qualified spelling wins"
    );
    let encoded = types::encoding(
        &JavaType::Reference(crate::ast::JavaTypeName::Generated(id)),
        &names,
        "Fixture.java",
        &mut errors,
    );
    assert_eq!(encoded.descriptor, 65_536);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("type descriptor bytes"))
    );
    for dimensions in [255, 256] {
        let ty = (0..dimensions).fold(JavaType::Primitive(JavaPrimitive::Int), |component, _| {
            JavaType::Array {
                component: Box::new(component),
                ownership: crate::ast::JavaArrayOwnership::InternalMutable,
            }
        });
        let mut errors = vec![];
        types::encoding(&ty, &names, "Fixture.java", &mut errors);
        assert_eq!(errors.is_empty(), dimensions == 255);
    }
}

#[test]
fn typed_resource_errors_cannot_disguise_syntax_or_capability_defects() {
    let diagnostic =
        |code| Diagnostic::error(code, "fixture", SourceRef::logical(["resource-test"]));
    for codes in [
        vec![],
        vec![DiagnosticCode::InvalidStructure],
        vec![DiagnosticCode::TargetResourceLimit],
        vec![
            DiagnosticCode::TargetResourceLimit,
            DiagnosticCode::UnsupportedCapability,
        ],
    ] {
        let failure = TypedGenerationError::Phase {
            stage: TypedPipelineStage::UnresolvedVerification,
            diagnostics: codes.into_iter().map(diagnostic).collect(),
        };
        assert!(
            std::panic::catch_unwind(|| JavaResourceError::from_typed_failure(failure)).is_err()
        );
    }
    let error = JavaResourceError::from_typed_failure(TypedGenerationError::Phase {
        stage: TypedPipelineStage::TargetResourceValidation,
        diagnostics: vec![diagnostic(DiagnosticCode::TargetResourceLimit)],
    });
    assert_eq!(
        error.diagnostics()[0].code,
        DiagnosticCode::TargetResourceLimit
    );
}

#[test]
fn oversized_typed_name_returns_resource_error_without_an_invariant_panic() {
    use portable_build::{I32, PortableName, portable_name, typed_list, typed_program};
    const NAME: PortableName = PortableName::checked(match std::str::from_utf8(&[b'n'; 65_536]) {
        Ok(value) => value,
        Err(_) => panic!("ASCII name"),
    });
    let program = typed_program(portable_name!("oversized_name"), |builder| {
        builder
            .function(NAME, typed_list![], I32::TYPE, |body, _| body.i32(1))
            .builder
    });
    let error = crate::JavaBackend.generate_typed(&program).unwrap_err();
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|error| error.code == DiagnosticCode::TargetResourceLimit
                && error.message.contains("identifier bytes"))
    );
}

#[test]
fn shared_output_size_limit_is_a_resource_error_not_a_syntax_panic() {
    use portable_build::{Text, portable_name, typed_list, typed_program};
    let program = typed_program(portable_name!("large_output"), |builder| {
        builder
            .function(
                portable_name!("text"),
                typed_list![],
                Text::TYPE,
                |body, _| {
                    // Escaped output is >8 MiB, while every individual Java constant
                    // remains valid and non-folding chunk assembly is small.
                    body.text("\0".repeat(3 * 1024 * 1024))
                },
            )
            .builder
    });
    let error = crate::JavaBackend.generate_typed(&program).unwrap_err();
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|error| error.code == DiagnosticCode::TargetResourceLimit
                && error.message.contains("FileTooLarge"))
    );
}
