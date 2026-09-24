//! Scalar API headers still introduce every owned tag into the consumer.
use super::nominal_fixture::{self as fixture, key};
use crate::{ast::*, dialect::*};
use portable_codegen::*;
use std::sync::Arc;

pub(super) fn producer(crate_id: u64, tags: &[&str], function_name: &str) -> CDependencyApi {
    let mut r = CRegistry::new();
    let header = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_producer_{crate_id}.h")).unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let source = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("producer_{crate_id}.c")).unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let ty = CObjectType::scalar(CScalarType::I32);
    let mut declarations = vec![];
    for name in tags {
        let record = r.declare_struct(&header, key(name)).unwrap();
        let owner = CAggregateRef::Struct(record);
        let tag = r
            .register_member(&owner, key("tag"), CObjectType::scalar(CScalarType::Bool))
            .unwrap();
        let payload = r
            .register_member(&owner, key("payload"), ty.clone())
            .unwrap();
        r.define_aggregate(&owner, vec![tag, payload]).unwrap();
        declarations.push(CFileItem::Declaration(
            CDeclarations::new(&r, header.clone())
                .unwrap()
                .aggregate(owner)
                .unwrap(),
        ));
    }
    let mut function_key = key(function_name);
    function_key.origin = CGeneratedOrigin::RustSource(Arc::new(
        fixture::origins(crate_id, &[function_name.to_owned()]).remove(0),
    ));
    let function = r
        .register_function(
            &header,
            function_key,
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
                vec![CParameterType::new(ty).unwrap()],
            ),
        )
        .unwrap();
    let scope = r.register_scope(&function, None, key("body")).unwrap();
    let parameter = r
        .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
        .unwrap();
    let e = CExpressions::new(&r);
    let s = CStatements::new(&r, function.clone()).unwrap();
    let body = s
        .block(
            scope,
            vec![
                s.return_statement(Some(
                    e.read(e.parameter(parameter.clone()).unwrap()).unwrap(),
                ))
                .unwrap(),
            ],
        )
        .unwrap();
    declarations.push(CFileItem::Declaration(
        CDeclarations::new(&r, header.clone())
            .unwrap()
            .function_prototype(function.clone(), CLinkage::External)
            .unwrap(),
    ));
    let definition = CFileItem::Definition(
        CDeclarations::new(&r, source.clone())
            .unwrap()
            .function_definition(function, CLinkage::External, vec![parameter], body)
            .unwrap(),
    );
    let files = vec![
        CDeclarations::new(&r, header)
            .unwrap()
            .source_file(declarations)
            .unwrap(),
        CDeclarations::new(&r, source)
            .unwrap()
            .source_file(vec![definition])
            .unwrap(),
    ];
    CDependencyApi::from_certificate(
        fixture::certify(fixture::Fixture {
            registry: r.freeze(),
            files,
            functions: vec![],
        })
        .unwrap(),
    )
    .unwrap()
}
