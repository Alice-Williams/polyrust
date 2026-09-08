use super::*;
use crate::{
    ast::*,
    capabilities as c,
    lower::{bool_literal, i32_literal, identifier},
};
use c::support::{JavaValueNode, plans::JavaRepresentation as R};
use portable_codegen::{
    GeneratedOrigin, GeneratedType, GeneratedValue, SourceRole, SynthesisReason, TargetAstBuilder,
    TargetFile, TargetTypeRef,
};
use portable_diagnostics::SourceRef;
use portable_ir::v0::Visibility;
mod controls;
mod values;

pub(super) fn symbols() -> (
    portable_codegen::GeneratedTypeId,
    portable_codegen::GeneratedValueId,
) {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let source = SourceRef::logical(["mapping-plan-test"]);
    let origin = GeneratedOrigin::Synthesized(SynthesisReason::TestHarness);
    let ty = builder.generated_type(GeneratedType {
        name: "Choice".into(),
        kind: JavaDeclarationKind::Enum,
        visibility: JavaVisibility::Public,
        origin: origin.clone(),
        source: source.clone(),
    });
    let value = builder.value(GeneratedValue {
        visibility: JavaVisibility::Public,
        name: "FIRST".into(),
        ty: TargetTypeRef::Generated(ty),
        origin,
        source,
    });
    (ty, value)
}
#[test]
fn declaration_type_and_erasure_categories_reject_corruption() {
    let (ty, constant) = symbols();
    let (plan, mut output) = checked(
        c::JavaFunctions,
        c::functions::JavaFunctionsInput::Declaration(Box::new(
            c::functions::JavaFunctionDeclarationInput {
                declared: JavaMethodDeclaration::Structural,
                visibility: Visibility::Public,
                name: "read".into(),
                parameters: vec![],
                return_type: JavaType::primitive(JavaPrimitive::Int),
                body: JavaBlock::new(vec![JavaStmt::Return(Some(i32_literal(1)))]),
            },
        )),
    );
    assert_eq!(plan.representation(), R::Declaration);
    let c::functions::JavaFunctionsNode::Declaration(method) = &mut output else {
        panic!()
    };
    method.modifiers.retain(|m| *m != JavaModifier::Static);
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaConstants,
        c::constants::JavaConstantsInput::Declaration {
            declared: constant,
            visibility: Visibility::Public,
            name: "FIRST".into(),
            ty: JavaType::primitive(JavaPrimitive::Int),
            initializer: Box::new(i32_literal(1)),
        },
    );
    let c::constants::JavaConstantsNode::Declaration(field) = &mut output else {
        panic!()
    };
    field.initializer = None;
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaTypeAliases,
        c::type_aliases::JavaTypeAliasInput {
            name: "Count".into(),
            target: JavaType::primitive(JavaPrimitive::Int),
        },
    );
    assert_eq!(plan.representation(), R::Erased);
    output._target = JavaType::primitive(JavaPrimitive::Long);
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaRecords,
        c::records::JavaRecordsInput::Type { record: ty },
    );
    let c::records::JavaRecordsNode::Type(actual) = &mut output else {
        panic!()
    };
    *actual = JavaType::known(JavaKnownType::String);
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaEnums,
        c::enums::JavaEnumsInput::Declaration {
            declared: ty,
            visibility: Visibility::Public,
            name: "Choice".into(),
            variants: vec![c::enums::JavaEnumVariantInput {
                declared: constant,
                name: "FIRST".into(),
            }],
        },
    );
    let c::enums::JavaEnumsNode::Declaration(actual) = &mut output else {
        panic!()
    };
    actual[0].members.clear();
    assert!(!plan.verify_output(&output));
}

#[test]
fn interface_catalogue_conformance_and_declarations_are_checked() {
    let (ty, _) = symbols();
    let (plan, mut output) = checked(
        c::JavaInterfaces,
        c::interfaces::JavaInterfacesInput::UninhabitedType {
            interface: ty,
            name: "Empty".into(),
            source: SourceRef::logical(["empty"]),
        },
    );
    let c::interfaces::JavaInterfacesNode::UninhabitedType(actual) = &mut output else {
        panic!()
    };
    actual.visibility = JavaVisibility::Public;
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaInterfaces,
        c::interfaces::JavaInterfacesInput::Declaration(Box::new(
            c::interfaces::JavaInterfaceDeclarationInput {
                declared: ty,
                visibility: Visibility::Public,
                name: "Readable".into(),
                permits: vec![],
                methods: vec![],
                uninhabited: None,
            },
        )),
    );
    let c::interfaces::JavaInterfacesNode::Declaration(actual) = &mut output else {
        panic!()
    };
    actual[0].kind = JavaDeclarationKind::Interface;
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaInterfaces,
        c::interfaces::JavaInterfacesInput::Conformance(Box::new(
            c::interfaces::JavaInterfaceConformanceInput {
                interfaces: vec![ty],
                methods: vec![],
            },
        )),
    );
    assert_eq!(plan.representation(), R::InterfaceDispatch);
    let c::interfaces::JavaInterfacesNode::Conformance(actual) = &mut output else {
        panic!()
    };
    actual.heritage = JavaHeritage::None;
    assert!(!plan.verify_output(&output));
}

#[test]
fn generated_file_and_harness_roles_are_checked() {
    let (ty, _) = symbols();
    let (plan, output) = checked(
        c::JavaModules,
        c::modules::JavaModuleInput {
            conformances: JavaConformanceInventory::structural(),
            entry: ty,
            declared: vec![],
            members: vec![],
        },
    );
    let corrupt = TargetFile::new(
        output.path().clone(),
        SourceRole::NativeTest,
        *output.module(),
        *output.placement(),
        output.items().to_vec(),
        *output.source_kind(),
        output.source().clone(),
    );
    assert!(!plan.verify_output(&corrupt));
    let corrupt_source = TargetFile::new(
        output.path().clone(),
        output.role(),
        *output.module(),
        *output.placement(),
        output.items().to_vec(),
        *output.source_kind(),
        SourceRef::logical(["wrong-attribution"]),
    );
    assert!(!plan.verify_output(&corrupt_source));
    let (plan, mut output) = checked(
        c::JavaPortableTests,
        c::portable_tests::JavaPortableTestsInput::Harness(
            c::portable_tests::JavaPortableTestHarnessInput {
                class_name: "GeneratedTest".into(),
                cases: vec![],
                expected_test_count: 0,
            },
        ),
    );
    let c::portable_tests::JavaPortableTestsNode::Harness(actual) = &mut output else {
        panic!()
    };
    actual.members.clear();
    assert!(!plan.verify_output(&output));
}
