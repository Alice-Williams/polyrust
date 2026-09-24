//! Target-only fixture preparation; metadata here is not evidence of rustc analysis.
use crate::{
    ast::*,
    dialect::{JavaDialect, JavaInvocationKind},
};
use portable_codegen::*;
use portable_diagnostics::SourceRef;
use std::{collections::BTreeMap, sync::Arc};

pub fn name(value: &str) -> JavaIdentifier {
    JavaIdentifier::new(value).unwrap()
}
pub fn int() -> JavaType {
    JavaType::primitive(JavaPrimitive::Int)
}
pub fn boolean() -> JavaType {
    JavaType::primitive(JavaPrimitive::Boolean)
}
fn source() -> SourceRef {
    SourceRef::logical(["java-dependency-fixture"])
}
pub fn id(crate_id: u64, hash: u64) -> RustDeclarationId {
    RustDeclarationId {
        crate_id,
        definition_path_hash: hash,
    }
}

pub struct Function {
    pub hash: u64,
    pub public: bool,
    pub name: JavaIdentifier,
    pub parameters: Vec<JavaParameter>,
    pub result: JavaType,
    pub body: JavaBlock,
}

pub fn functions(zero_value: i32) -> Vec<Function> {
    [
        (
            10,
            true,
            vec![],
            int(),
            JavaExpr::literal(int(), JavaLiteral::I32(zero_value)),
        ),
        (
            11,
            true,
            vec![int(), boolean(), int(), boolean()],
            int(),
            JavaExpr::local(int(), name("p0")),
        ),
        (
            12,
            true,
            vec![boolean()],
            boolean(),
            JavaExpr::local(boolean(), name("p0")),
        ),
        (
            13,
            false,
            vec![int()],
            int(),
            JavaExpr::local(int(), name("p0")),
        ),
    ]
    .into_iter()
    .map(|(hash, public, parameters, result, value)| Function {
        hash,
        public,
        name: name(&format!("fn{hash:016x}")),
        parameters: parameters
            .into_iter()
            .enumerate()
            .map(|(index, ty)| JavaParameter {
                ty,
                name: name(&format!("p{index}")),
                final_parameter: true,
            })
            .collect(),
        result,
        body: JavaBlock::new(vec![JavaStmt::Return(Some(value))]),
    })
    .collect()
}

pub fn package(crate_id: u64, functions: Vec<Function>) -> TargetAstPackage<JavaDialect> {
    package_with(crate_id, functions, |_, _, _| {})
}

pub fn package_with(
    crate_id: u64,
    functions: Vec<Function>,
    edit: impl FnOnce(
        &mut TargetAstBuilder<JavaDialect>,
        &mut Vec<GeneratedSymbolId>,
        &mut JavaTypeDeclaration,
    ),
) -> TargetAstPackage<JavaDialect> {
    package_configured(crate_id, functions, |_| {}, edit, Default::default())
}

pub fn package_with_exports(
    crate_id: u64,
    functions: Vec<Function>,
    edit: impl FnOnce(&mut RustCrateExports),
) -> TargetAstPackage<JavaDialect> {
    package_configured(crate_id, functions, edit, |_, _, _| {}, Default::default())
}

pub fn package_with_dependencies(
    crate_id: u64,
    functions: Vec<Function>,
    dependencies: crate::dialect::JavaDependencyBindings,
) -> TargetAstPackage<JavaDialect> {
    package_configured(crate_id, functions, |_| {}, |_, _, _| {}, dependencies)
}

pub(crate) fn package_configured(
    crate_id: u64,
    functions: Vec<Function>,
    exports_edit: impl FnOnce(&mut RustCrateExports),
    edit: impl FnOnce(
        &mut TargetAstBuilder<JavaDialect>,
        &mut Vec<GeneratedSymbolId>,
        &mut JavaTypeDeclaration,
    ),
    dependencies: crate::dialect::JavaDependencyBindings,
) -> TargetAstPackage<JavaDialect> {
    let root = id(crate_id, 1);
    let location = RustSourceLocation {
        file: "src/lib.rs".into(),
        line: 1,
        column: 1,
    };
    let module = Arc::new(RustModuleDocumentation {
        declaration: root,
        parent: None,
        location: location.clone(),
        documentation: vec![" Source owner documentation.".into()],
    });
    let ancestry: RustModuleAncestry = vec![module].into();
    let bindings = functions
        .iter()
        .filter(|function| function.public)
        .flat_map(|function| {
            ["export", "alias"].map(|prefix| {
                (
                    RustExportName {
                        namespace: RustExportNamespace::Value,
                        name: format!("{prefix}{}", function.hash),
                    },
                    RustExportTarget::Declaration(id(crate_id, function.hash)),
                )
            })
        })
        .collect();
    let mut exports = RustCrateExports {
        root,
        modules: BTreeMap::from([(root, bindings)]),
        module_ancestries: BTreeMap::from([(root, ancestry.clone())]),
    };
    exports_edit(&mut exports);
    let exports = Arc::new(exports);
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let facade = builder.generated_type(GeneratedType {
        name: "Generated".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
        source: source(),
    });
    let mut declared = vec![GeneratedSymbolId::Type(facade)];
    let mut members = vec![JavaMember::Constructor(JavaConstructor {
        modifiers: vec![JavaModifier::Private],
        name: name("Generated"),
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    })];
    for function in functions {
        let visibility = if function.public {
            JavaVisibility::Public
        } else {
            JavaVisibility::Private
        };
        let signature = JavaMethodSignature {
            receiver: None,
            parameters: function
                .parameters
                .iter()
                .map(|parameter| parameter.ty.clone())
                .collect(),
            result: function.result.clone(),
            checked_exceptions: vec![],
            nullable_result: false,
            pure: true,
        };
        let callable = builder.callable(GeneratedCallable {
            name: function.name.as_str().into(),
            signature: JavaDialect.coarse_signature(&signature),
            visibility,
            origin: GeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                declaration: id(crate_id, function.hash),
                node: RustSourceNode::Declaration,
                module: root,
                location: location.clone(),
                visibility: if function.public {
                    RustVisibility::Public
                } else {
                    RustVisibility::RestrictedTo(root)
                },
                externally_reachable: function.public,
                documentation: vec![format!(" Function {} documentation.", function.hash)],
                module_ancestors: ancestry.clone(),
                crate_exports: exports.clone(),
            })),
            source: source(),
        });
        declared.push(GeneratedSymbolId::Callable(callable));
        members.push(JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Callable(callable),
            annotations: vec![],
            modifiers: vec![
                if function.public {
                    JavaModifier::Public
                } else {
                    JavaModifier::Private
                },
                JavaModifier::Static,
            ],
            type_parameters: vec![],
            return_type: function.result,
            name: function.name,
            parameters: function.parameters,
            body: Some(function.body),
        }));
        assert_eq!(signature.receiver, None);
        assert_eq!(
            JavaDialect.coarse_signature(&signature).invocation,
            JavaInvocationKind::Static
        );
    }
    let module = JavaPackage::RustCrate(crate_id);
    let mut declaration = JavaTypeDeclaration {
        declared: Some(facade),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: name("Generated"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members,
    };
    edit(&mut builder, &mut declared, &mut declaration);
    let file = builder.file(TargetFile::new(
        RelativeOutputPath::new(format!(
            "{}Generated.java",
            module.source_directory(JavaFilePlacement::Main)
        ))
        .unwrap(),
        SourceRole::PublicApi,
        module,
        JavaFilePlacement::Main,
        vec![JavaFileItem::Type {
            declared,
            conformances: JavaConformanceInventory::structural().into(),
            package_metadata: None,
            dependencies,
            declaration: Box::new(declaration),
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

pub fn certify(package: TargetAstPackage<JavaDialect>) -> RenderReadyPackage<JavaDialect> {
    let verified = verify_unresolved_package(&JavaDialect, package).unwrap();
    let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
    certify_resolved_package(&JavaDialect, linked).unwrap()
}

pub fn record_package(
    edit: impl FnOnce(&mut crate::tests::source_record_fixture::Fixture),
) -> TargetAstPackage<JavaDialect> {
    let mut fixture = crate::tests::source_documentation_fixture::fixture(
        crate::tests::source_record_fixture::Facade::EntryPoint,
    );
    fixture
        .facade
        .members
        .push(JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Private],
            name: name("Generated"),
            parameters: vec![],
            body: JavaBlock::new(vec![]),
        }));
    edit(&mut fixture);
    fixture.finish()
}
