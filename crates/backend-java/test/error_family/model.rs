//! Small typed fixture vocabulary; no source strings enter generation.
use portable_backend_java::{ast::*, dialect::*};
use portable_codegen::*;
use portable_diagnostics::SourceRef;

pub fn source() -> SourceRef {
    SourceRef::logical(["java-error-family-proof"])
}
pub fn name(value: &str) -> JavaIdentifier {
    JavaIdentifier::new(value).unwrap()
}
pub fn reference(id: GeneratedTypeId) -> JavaType {
    JavaType::Reference(JavaTypeName::Generated(id))
}
pub fn int() -> JavaType {
    JavaType::primitive(JavaPrimitive::Int)
}
pub fn parameter(ty: JavaType, value: &str) -> JavaParameter {
    JavaParameter {
        ty,
        name: name(value),
        final_parameter: true,
    }
}
pub fn register(
    builder: &mut TargetAstBuilder<JavaDialect>,
    value: &str,
    kind: JavaDeclarationKind,
) -> GeneratedTypeId {
    builder.generated_type(GeneratedType {
        name: value.into(),
        kind,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
        source: source(),
    })
}
pub fn declaration(
    id: GeneratedTypeId,
    value: &str,
    kind: JavaDeclarationKind,
) -> JavaTypeDeclaration {
    JavaTypeDeclaration {
        declared: Some(id),
        kind,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: name(value),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![],
    }
}
pub fn upcast(value: JavaExpr, owner: GeneratedTypeId, interface: GeneratedTypeId) -> JavaExpr {
    JavaExpr {
        ty: reference(interface),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::InterfaceCoercion {
            implementation: JavaInterfaceWitness::for_generated_adapter(owner, interface),
            target: reference(interface),
            value: Box::new(value),
        },
    }
}
pub fn signature(parameters: Vec<JavaType>, result: JavaType) -> JavaMethodSignature {
    JavaMethodSignature {
        receiver: None,
        parameters,
        result,
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    }
}
pub fn nonnull(value: JavaExpr) -> JavaExpr {
    let ty = value.ty.clone();
    JavaExpr {
        ty: ty.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable: JavaKnownCallable::ObjectsRequireNonNull,
                signature: signature(vec![ty.clone()], ty),
            },
            receiver: None,
            arguments: vec![value],
        },
    }
}
pub fn method(
    builder: &mut TargetAstBuilder<JavaDialect>,
    value: &str,
    parameters: Vec<JavaParameter>,
    result: JavaExpr,
) -> JavaMember {
    let signature = signature(
        parameters.iter().map(|p| p.ty.clone()).collect(),
        result.ty.clone(),
    );
    let id = builder.callable(GeneratedCallable {
        name: value.into(),
        signature: TargetCallableSignature {
            invocation: JavaInvocationKind::Static,
            receiver: None,
            parameters: signature
                .parameters
                .iter()
                .map(|ty| JavaDialect.registered_type(ty))
                .collect(),
            return_type: JavaDialect.registered_type(&signature.result),
        },
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
        source: source(),
    });
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Callable(id),
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters: vec![],
        return_type: result.ty.clone(),
        name: name(value),
        parameters,
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(result))])),
    })
}
pub fn symbols(node: &JavaTypeDeclaration, into: &mut Vec<GeneratedSymbolId>) {
    into.extend(node.declared.map(GeneratedSymbolId::Type));
    for member in &node.members {
        match member {
            JavaMember::NestedType(child) => symbols(child, into),
            JavaMember::EnumConstant(value) => into.push(GeneratedSymbolId::Value(value.declared)),
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Callable(id),
                ..
            }) => into.push(GeneratedSymbolId::Callable(*id)),
            _ => {}
        }
    }
}
pub fn finish(
    mut builder: TargetAstBuilder<JavaDialect>,
    facade: JavaTypeDeclaration,
) -> TargetAstPackage<JavaDialect> {
    let mut declared = vec![];
    symbols(&facade, &mut declared);
    let file = builder.file(TargetFile::new(
        RelativeOutputPath::new("src/main/java/org/polyrust/generated/ErrorFixture.java").unwrap(),
        SourceRole::PublicApi,
        JavaPackage::Generated,
        JavaFilePlacement::Main,
        vec![JavaFileItem::Type {
            declared,
            conformances: JavaConformanceInventory::structural().into(),
            package_metadata: None,
            dependencies: Default::default(),
            declaration: facade.into(),
        }],
        JavaSourceFileKind::CompilationUnit,
        source(),
    ));
    builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        vec![TargetFileMember::Source(file)],
        source(),
    ));
    builder.build()
}
pub fn certify(
    ast: TargetAstPackage<JavaDialect>,
) -> Result<RenderReadyPackage<JavaDialect>, String> {
    let checked = verify_unresolved_package(&JavaDialect, ast).map_err(|e| format!("{e:?}"))?;
    let linked = TargetLinker::new(JavaDialect)
        .link_ast(&checked)
        .map_err(|e| format!("{e:?}"))?;
    certify_resolved_package(&JavaDialect, linked).map_err(|e| format!("{e:?}"))
}
