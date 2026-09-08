use super::blocks::{JavaBlock, JavaLocalFinality};
use super::completion::block_guarantees_exit;
use super::declaration_model::{
    JavaAnnotation, JavaCompileFailField, JavaConstructor, JavaDeclarationKind, JavaEnumConstant,
    JavaField, JavaHeritage, JavaMember, JavaMethod, JavaMethodDeclaration, JavaModifier,
    JavaParameter, JavaRecordComponent, JavaRecordComponentOrigin, JavaTypeDeclaration,
    JavaVisibility,
};
use super::exceptions::admitted_throwable_is_supertype_of;
use super::expression_model::{
    JavaBinaryOperator, JavaCallableRef, JavaLiteral, JavaMemberOrigin, JavaMethodSignature,
    JavaNullPurpose, JavaPrecedence, JavaUnaryOperator, JavaValueRef,
};
use super::expression_nodes::{JavaConstructorRef, JavaExpr, JavaExprKind, JavaFieldRef};
use super::file_model::JavaFileItem;
use super::identifiers::{JAVA_KEYWORDS, JavaIdentifier};
use super::lexical_blocks::verify_block_scope;
use super::lexical_scope::JavaLexicalScope;
use super::operator_signatures::{
    binary_signature_matches, literal_matches_type, unary_signature_matches,
};
use super::resolved_files::{JavaFilePlacement, JavaPackage, JavaSourceFileKind};
use super::runtime_members::JavaRuntimeMember;
use super::statement_model::{JavaCatch, JavaPattern, JavaStmt, JavaSwitchArm};
use super::types::{
    JavaArrayOwnership, JavaArrayOwnershipTransition, JavaKnownType, JavaPrimitive, JavaType,
    JavaTypeName, JavaTypeUse, JavaWildcardBound,
};
use crate::dialect::{
    JavaDialect, JavaInvocationKind, JavaKnownConstructor, JavaKnownMethod, JavaRuntimeHelper,
};
use portable_codegen::{GeneratedSymbolId, GeneratedTypeId, GeneratedValueId, TargetTypeRef};
use portable_core_ir::CoreFieldId;
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;
fn verifier_source(label: &str) -> portable_diagnostics::SourceRef {
    portable_diagnostics::SourceRef::logical(["java-ast-verifier-test", label])
}

fn fixture_declaration(members: Vec<JavaMember>) -> JavaTypeDeclaration {
    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("Fixture"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members,
    }
}

fn ordinary_owner_fixture() -> (
    portable_codegen::TargetAstBuilder<JavaDialect>,
    GeneratedTypeId,
) {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let owner = builder.generated_type(portable_codegen::GeneratedType {
        name: "Fixture".to_owned(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: verifier_source("ordinary-owner"),
    });
    (builder, owner)
}

fn verify_fixture(
    builder: portable_codegen::TargetAstBuilder<JavaDialect>,
    declarations: Vec<(Vec<GeneratedSymbolId>, JavaTypeDeclaration)>,
) -> Result<(), Vec<portable_diagnostics::Diagnostic>> {
    verify_file_items(
        builder,
        portable_codegen::SourceRole::PublicApi,
        JavaFilePlacement::Main,
        declarations
            .into_iter()
            .map(|(declared, declaration)| JavaFileItem::Type {
                conformances: crate::ast::JavaConformanceInventory::structural().into(),
                declared,
                declaration,
            })
            .collect(),
    )
}

fn verify_file_items(
    mut builder: portable_codegen::TargetAstBuilder<JavaDialect>,
    role: portable_codegen::SourceRole,
    placement: JavaFilePlacement,
    items: Vec<JavaFileItem>,
) -> Result<(), Vec<portable_diagnostics::Diagnostic>> {
    let group_role = match role {
        portable_codegen::SourceRole::PublicApi => portable_codegen::FileGroupRole::PublicApi,
        portable_codegen::SourceRole::Implementation => {
            portable_codegen::FileGroupRole::Implementation
        }
        portable_codegen::SourceRole::Runtime => portable_codegen::FileGroupRole::Runtime,
        portable_codegen::SourceRole::NativeTest => portable_codegen::FileGroupRole::NativeTests,
        portable_codegen::SourceRole::Conformance => portable_codegen::FileGroupRole::Conformance,
        portable_codegen::SourceRole::NegativeTest => {
            portable_codegen::FileGroupRole::NegativeTests
        }
    };
    let path = match placement {
        JavaFilePlacement::Runtime => "src/main/java/org/polyrust/generated/Runtime.java",
        JavaFilePlacement::Main => "src/main/java/org/polyrust/generated/Fixture.java",
        JavaFilePlacement::NativeTest
        | JavaFilePlacement::Conformance
        | JavaFilePlacement::NegativeTest => "src/test/java/org/polyrust/generated/Fixture.java",
    };
    let file = builder.file(portable_codegen::TargetFile::new(
        portable_codegen::RelativeOutputPath::new(path).unwrap(),
        role,
        JavaPackage::Generated,
        placement,
        items,
        JavaSourceFileKind::CompilationUnit,
        verifier_source("file"),
    ));
    builder.group(portable_codegen::TargetFileGroup::new(
        group_role,
        vec![portable_codegen::TargetFileMember::Source(file)],
        verifier_source("group"),
    ));
    portable_codegen::verify_target_ast(&builder.build())
}

fn structural_method(
    name: &str,
    return_type: JavaType,
    parameters: Vec<JavaParameter>,
    body: JavaBlock,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters: vec![],
        return_type,
        name: JavaIdentifier::from_portable(name),
        parameters,
        body: Some(body),
    })
}

fn parameter(ty: JavaType, name: &str) -> JavaParameter {
    JavaParameter {
        ty,
        name: JavaIdentifier::from_portable(name),
        final_parameter: true,
    }
}

fn instanceof(value: JavaExpr, target: JavaType, binding: &str) -> JavaExpr {
    JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Boolean),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(value),
            target,
            binding: Some(JavaIdentifier::from_portable(binding)),
        },
    }
}

fn this_field(owner: JavaType, ty: JavaType, name: &str) -> JavaExpr {
    JavaExpr {
        ty: ty.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Field {
            receiver: Box::new(JavaExpr {
                ty: owner,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::This),
            }),
            field: JavaFieldRef::Structural {
                name: JavaIdentifier::from_portable(name),
                ty,
            },
        },
    }
}

fn fixture_core_field() -> CoreFieldId {
    let checked = portable_check::v0::check_program(
        portable_ir::v0::from_json(include_bytes!(
            "../../../build/testdata/registration.poly.json"
        ))
        .expect("fixture parses"),
    )
    .expect("fixture checks");
    let core = portable_core_ir::lower_checked(&checked).expect("fixture lowers to CoreIR");
    core.records()
        .first()
        .and_then(|record| record.fields.first())
        .copied()
        .expect("fixture contains a record field")
}

fn fixture_core_implementation_method(
    owner: GeneratedTypeId,
    target_method: portable_codegen::GeneratedInterfaceMethodId,
) -> (
    portable_core_ir::CoreImplementationMethodId,
    super::JavaImplementationWitness,
) {
    let checked = portable_check::v0::check_program(
        portable_ir::v0::from_json(include_bytes!(
            "../../../build/testdata/registration.poly.json"
        ))
        .expect("fixture parses"),
    )
    .expect("fixture checks");
    let core = portable_core_ir::lower_checked(&checked).expect("fixture lowers to CoreIR");
    let method = core
        .implementations()
        .first()
        .and_then(|implementation| implementation.methods.first())
        .copied()
        .expect("fixture contains an implementation method");
    (
        method,
        super::JavaImplementationWitness::from_checked(&core, method, owner, target_method),
    )
}

#[path = "ast/access.rs"]
mod access;
#[path = "ast/array_ownership.rs"]
mod array_ownership;
#[path = "ast/boxed_casts.rs"]
mod boxed_casts;
#[path = "ast/catch_order.rs"]
mod catch_order;
#[path = "ast/compiler_oracle.rs"]
mod compiler_oracle;
#[path = "ast/constructor_assignment.rs"]
mod constructor_assignment;
#[path = "ast/declaration_grammar.rs"]
mod declaration_grammar;
#[path = "ast/enums.rs"]
mod enums;
#[path = "ast/field_context.rs"]
mod field_context;
#[path = "ast/foreach_and_erasure.rs"]
mod foreach_and_erasure;
#[path = "ast/initializers_and_exceptions.rs"]
mod initializers_and_exceptions;
#[path = "ast/interface_conformance.rs"]
mod interface_conformance;
#[path = "ast/interface_signatures.rs"]
mod interface_signatures;
#[path = "ast/lexical_flow.rs"]
mod lexical_flow;
#[path = "ast/literal_limits.rs"]
mod literal_limits;
#[path = "ast/member_catalogue.rs"]
mod member_catalogue;
#[path = "ast/modifiers.rs"]
mod modifiers;
#[path = "ast/negative_members.rs"]
mod negative_members;
#[path = "ast/pattern_bindings.rs"]
mod pattern_bindings;
#[path = "ast/privileged_literals.rs"]
mod privileged_literals;
#[path = "ast/qualifier_bindings.rs"]
mod qualifier_bindings;
#[path = "ast/sealed_permits.rs"]
mod sealed_permits;
#[path = "ast/statement_grammar.rs"]
mod statement_grammar;
#[path = "ast/synthetic_owner_fixtures.rs"]
mod synthetic_owner_fixtures;
#[path = "ast/type_and_operator_checks.rs"]
mod type_and_operator_checks;
#[path = "ast/types_and_casts.rs"]
mod types_and_casts;
