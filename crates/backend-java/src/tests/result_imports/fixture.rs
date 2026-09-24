//! Target-only source metadata; no claim of compiler Result admission.
use crate::{
    ast::*,
    dialect::*,
    tests::{
        scalar_result_families::fixture::Family, source_dependency_fixture as source,
        source_package_fixture,
    },
};
use portable_codegen::*;

pub fn owner_package() -> (TargetAstPackage<JavaDialect>, [JavaScalarResultTypes; 2]) {
    let mut selections = None;
    let package = source::package_with(7, vec![], |builder, declared, facade| {
        let first = Family::new(builder, "");
        let second = Family::new(builder, "Other");
        selections = Some([first.types, second.types]);
        for child in first
            .declarations()
            .into_iter()
            .chain(second.declarations())
        {
            declared.push(GeneratedSymbolId::Type(child.declared.unwrap()));
            facade.members.push(JavaMember::NestedType(child));
        }
    });
    (
        source_package_fixture::attach(package, source_package_fixture::graph()),
        selections.unwrap(),
    )
}
pub fn owner() -> JavaDependencyApi {
    let (package, types) = owner_package();
    JavaDependencyApi::from_certificate_with_results(source::certify(package), &types).unwrap()
}
pub fn consumer(
    scope: JavaDependencyBindings,
    result: JavaType,
    parameters: Vec<JavaType>,
    value: JavaExpr,
) -> TargetAstPackage<JavaDialect> {
    consumer_at(9, scope, result, parameters, value)
}
pub fn consumer_at(
    crate_id: u64,
    scope: JavaDependencyBindings,
    result: JavaType,
    parameters: Vec<JavaType>,
    value: JavaExpr,
) -> TargetAstPackage<JavaDialect> {
    source::package_with_dependencies(
        crate_id,
        vec![source::Function {
            hash: 10,
            public: true,
            name: source::name("probe"),
            parameters: parameters
                .into_iter()
                .enumerate()
                .map(|(index, ty)| JavaParameter {
                    ty,
                    name: source::name(&format!("p{index}")),
                    final_parameter: true,
                })
                .collect(),
            result,
            body: JavaBlock::new(vec![JavaStmt::Return(Some(value))]),
        }],
        scope,
    )
}
pub fn reject(package: TargetAstPackage<JavaDialect>) {
    let Err(errors) = verify_unresolved_package(&JavaDialect, package)
        .and_then(|checked| TargetLinker::new(JavaDialect).link_ast(&checked))
        .and_then(|linked| certify_resolved_package(&JavaDialect, linked))
    else {
        panic!("invalid nominal consumer certified")
    };
    assert!(!errors.is_empty());
}
pub fn text(package: &RenderReadyPackage<JavaDialect>) -> String {
    let rendered = render_certified_package(&crate::render::JavaRenderer, package).unwrap();
    assert_eq!(rendered.files().len(), 1);
    let OutputContents::Text(text) = rendered.files()[0].contents() else {
        panic!("text")
    };
    text.clone()
}
