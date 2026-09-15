//! Target-only construction. Independent source/rustc integration is a later gate.
use super::source_dependency_fixture as f;
use crate::{
    ast::*,
    dialect::{JavaDependencyApi, JavaDialect},
};
use portable_codegen::*;
use portable_diagnostics::SourceRef;

pub fn values() -> Vec<JavaLiteral> {
    vec![
        JavaLiteral::Boolean(false),
        JavaLiteral::Boolean(true),
        JavaLiteral::I32(i32::MIN),
        JavaLiteral::I32(i32::MAX),
        JavaLiteral::I64(i64::MIN),
        JavaLiteral::I64(i64::MAX),
        JavaLiteral::I64(9_007_199_254_740_993),
        JavaLiteral::I32(62),
    ]
}
pub fn ty(value: &JavaLiteral) -> JavaType {
    JavaType::primitive(match value {
        JavaLiteral::Boolean(_) => JavaPrimitive::Boolean,
        JavaLiteral::I32(_) => JavaPrimitive::Int,
        JavaLiteral::I64(_) => JavaPrimitive::Long,
        _ => panic!("fixture scalar"),
    })
}
pub struct Fixture {
    pub builder: TargetAstBuilder<JavaDialect>,
    pub declared: Vec<GeneratedSymbolId>,
    pub facade: JavaTypeDeclaration,
    pub crate_id: u64,
}
impl Fixture {
    pub fn new(mixed: bool) -> Self {
        Self::with_exports(mixed, |_| {})
    }
    pub fn with_exports(mixed: bool, edit: impl FnOnce(&mut RustCrateExports)) -> Self {
        Self::configured(mixed, edit, |_, _| {})
    }
    pub fn configured(
        mixed: bool,
        edit: impl FnOnce(&mut RustCrateExports),
        mut registration: impl FnMut(usize, &mut GeneratedValue<JavaDialect>),
    ) -> Self {
        let crate_id = 0x35c;
        let values = values();
        // Reuse only the metadata fixture: replace its method declarations with
        // separately registered values below, never reuse its certificate.
        let functions = values
            .iter()
            .enumerate()
            .flat_map(|(i, value)| {
                [false, true]
                    .into_iter()
                    .filter(move |method| !method || mixed)
                    .map(move |method| f::Function {
                        hash: 10 + i as u64 + if method { 100 } else { 0 },
                        public: true,
                        name: f::name(&format!("item{i}_{method}")),
                        parameters: vec![],
                        result: ty(value),
                        body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                            ty(value),
                            value.clone(),
                        )))]),
                    })
            })
            .collect();
        let metadata = f::package_with_exports(crate_id, functions, edit);
        let origin = |hash| {
            metadata
                .callables()
                .find(|item| {
                    matches!(&item.origin,
                GeneratedOrigin::RustSource(source) if source.declaration == f::id(crate_id, hash))
                })
                .unwrap()
                .origin
                .clone()
        };
        let source = || SourceRef::logical(["java-source-constant"]);
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
            name: f::name("Generated"),
            parameters: vec![],
            body: JavaBlock::new(vec![]),
        })];
        for (i, value) in values.into_iter().enumerate() {
            let ty = ty(&value);
            let name = f::name(&format!("constant{i}"));
            let mut value_registration = GeneratedValue {
                name: name.as_str().into(),
                ty: JavaDialect.registered_type(&ty),
                visibility: JavaVisibility::Public,
                origin: origin(10 + i as u64),
                source: source(),
            };
            let GeneratedOrigin::RustSource(constant_origin) = &mut value_registration.origin
            else {
                panic!("source")
            };
            std::sync::Arc::make_mut(constant_origin).documentation =
                vec![format!(" Constant {i} documentation.")];
            registration(i, &mut value_registration);
            let id = builder.value(value_registration);
            declared.push(GeneratedSymbolId::Value(id));
            members.push(JavaMember::Field(JavaField {
                declared: Some(id),
                modifiers: vec![
                    JavaModifier::Public,
                    JavaModifier::Static,
                    JavaModifier::Final,
                ],
                ty: ty.clone(),
                name,
                initializer: Some(JavaExpr::literal(ty.clone(), value)),
            }));
            if mixed {
                let signature = JavaMethodSignature {
                    receiver: None,
                    parameters: vec![],
                    result: ty.clone(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                };
                let name = f::name(&format!("read{i}"));
                let callable = builder.callable(GeneratedCallable {
                    name: name.as_str().into(),
                    signature: JavaDialect.coarse_signature(&signature),
                    visibility: JavaVisibility::Public,
                    origin: origin(110 + i as u64),
                    source: source(),
                });
                declared.push(GeneratedSymbolId::Callable(callable));
                members.push(JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Callable(callable),
                    annotations: vec![],
                    modifiers: vec![JavaModifier::Public, JavaModifier::Static],
                    type_parameters: vec![],
                    return_type: ty.clone(),
                    name,
                    parameters: vec![],
                    body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                        ty,
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Value(JavaValueRef::Generated(
                            GeneratedSymbolId::Value(id),
                        )),
                    }))])),
                }));
            }
        }
        Self {
            builder,
            declared,
            crate_id,
            facade: JavaTypeDeclaration {
                declared: Some(facade),
                kind: JavaDeclarationKind::FinalClass,
                visibility: JavaVisibility::Public,
                modifiers: vec![],
                name: f::name("Generated"),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: vec![],
                members,
            },
        }
    }
    pub fn field(&mut self, index: usize) -> &mut JavaField {
        self.facade
            .members
            .iter_mut()
            .filter_map(|member| match member {
                JavaMember::Field(field) => Some(field),
                _ => None,
            })
            .nth(index)
            .unwrap()
    }
    pub fn finish(self) -> TargetAstPackage<JavaDialect> {
        let Self {
            mut builder,
            declared,
            facade,
            crate_id,
        } = self;
        let module = JavaPackage::RustCrate(crate_id);
        let source = SourceRef::logical(["java-source-constant"]);
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
                dependencies: Default::default(),
                declaration: Box::new(facade),
            }],
            JavaSourceFileKind::CompilationUnit,
            source.clone(),
        ));
        builder.group(TargetFileGroup::new(
            FileGroupRole::PublicApi,
            vec![TargetFileMember::Source(file)],
            source,
        ));
        builder.build()
    }
}
pub fn api(mixed: bool) -> JavaDependencyApi {
    JavaDependencyApi::from_certificate(f::certify(Fixture::new(mixed).finish())).unwrap()
}
pub fn admit(draft: TargetAstPackage<JavaDialect>) -> Result<JavaDependencyApi, String> {
    let verified = verify_unresolved_package(&JavaDialect, draft).map_err(|e| format!("{e:?}"))?;
    let linked = TargetLinker::new(JavaDialect)
        .link_ast(&verified)
        .map_err(|e| format!("{e:?}"))?;
    let ready = certify_resolved_package(&JavaDialect, linked).map_err(|e| format!("{e:?}"))?;
    JavaDependencyApi::from_certificate(ready)
}
