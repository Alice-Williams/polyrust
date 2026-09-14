//! A public scalar prototype and a private helper share one exact registry.
use crate::ast::*;
use portable_codegen::RelativeOutputPath;

pub(crate) struct Fixture {
    pub registry: CFrozenRegistry,
    pub files: Vec<CSourceFile>,
    pub public: CFunctionRef,
    pub helper: CFunctionRef,
    pub record: Option<CStructRef>,
}

#[derive(Clone, Copy)]
pub(crate) enum RecordLayout {
    Absent,
    Implementation,
    Header,
}

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}

pub(super) fn fixture() -> Fixture {
    with_origins(key("api").origin, key("helper").origin)
}

pub(super) fn with_origins(
    public_origin: CGeneratedOrigin,
    helper_origin: CGeneratedOrigin,
) -> Fixture {
    build(
        public_origin,
        helper_origin,
        "polyrust_crate_api.h",
        "crate_api.c",
        RecordLayout::Absent,
        CScalarType::I32,
    )
}

pub(super) fn with_layout(layout: RecordLayout) -> Fixture {
    build(
        key("api").origin,
        key("helper").origin,
        "polyrust_crate_api.h",
        "crate_api.c",
        layout,
        CScalarType::I32,
    )
}

pub(super) fn with_paths(header: &str, implementation: &str) -> Fixture {
    build(
        key("api").origin,
        key("helper").origin,
        header,
        implementation,
        RecordLayout::Absent,
        CScalarType::I32,
    )
}

pub(crate) fn with_source_shape(
    public: CGeneratedOrigin,
    helper: CGeneratedOrigin,
    layout: RecordLayout,
    scalar: CScalarType,
) -> Fixture {
    build(
        public,
        helper,
        "polyrust_crate_api.h",
        "crate_api.c",
        layout,
        scalar,
    )
}

fn build(
    public_origin: CGeneratedOrigin,
    helper_origin: CGeneratedOrigin,
    header_path: &str,
    implementation_path: &str,
    layout: RecordLayout,
    scalar: CScalarType,
) -> Fixture {
    let mut registry = CRegistry::new();
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(header_path).unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let implementation = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(implementation_path).unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let scalar = CObjectType::scalar(scalar);
    let signature = CFunctionType::new(
        CReturnType::Value(CReturnValue::new(scalar.clone()).unwrap()),
        vec![CParameterType::new(scalar.clone()).unwrap()],
    );
    let public = registry
        .register_function(
            &header,
            CDeclarationKey {
                origin: public_origin,
                ..key("api")
            },
            signature.clone(),
        )
        .unwrap();
    let helper = registry
        .register_function(
            &implementation,
            CDeclarationKey {
                origin: helper_origin,
                ..key("helper")
            },
            signature,
        )
        .unwrap();
    let public_parameter = registry
        .register_parameter(&public, 0, key("input"), CConstness::Unqualified)
        .unwrap();
    let helper_parameter = registry
        .register_parameter(&helper, 0, key("input"), CConstness::Unqualified)
        .unwrap();
    let public_scope = registry.register_scope(&public, None, key("body")).unwrap();
    let helper_scope = registry.register_scope(&helper, None, key("body")).unwrap();
    let result = registry
        .register_local(&public_scope, key("result"), scalar.clone())
        .unwrap();
    let record = match layout {
        RecordLayout::Absent => None,
        RecordLayout::Implementation | RecordLayout::Header => {
            let owner_file = match layout {
                RecordLayout::Implementation => &implementation,
                RecordLayout::Header => &header,
                RecordLayout::Absent => unreachable!(),
            };
            let record = registry.declare_struct(owner_file, key("record")).unwrap();
            let owner = CAggregateRef::Struct(record.clone());
            let member = registry
                .register_member(&owner, key("secret"), scalar)
                .unwrap();
            registry
                .define_aggregate(&owner, vec![member.clone()])
                .unwrap();
            let local = registry
                .register_local(
                    &helper_scope,
                    key("state"),
                    CObjectType::structure(record.clone()),
                )
                .unwrap();
            Some((record, member, local))
        }
    };
    let expressions = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, public.clone()).unwrap();
    let input = expressions
        .read(expressions.parameter(public_parameter.clone()).unwrap())
        .unwrap();
    let call = expressions
        .call_value(expressions.direct(helper.clone()).unwrap(), vec![input])
        .unwrap();
    let initial = expressions.expression_initializer(call).unwrap();
    let declaration = statements.declare(result.clone(), Some(initial)).unwrap();
    let value = expressions
        .read(expressions.local(result).unwrap())
        .unwrap();
    let public_body = statements
        .block(
            public_scope,
            vec![
                declaration,
                statements.return_statement(Some(value)).unwrap(),
            ],
        )
        .unwrap();
    let statements = CStatements::new(&registry, helper.clone()).unwrap();
    let value = expressions
        .read(expressions.parameter(helper_parameter.clone()).unwrap())
        .unwrap();
    let mut helper_statements = Vec::new();
    let value = if let Some((record, member, local)) = &record {
        let initializer = expressions
            .struct_initializer(
                record.clone(),
                vec![(
                    member.clone(),
                    expressions.expression_initializer(value).unwrap(),
                )],
            )
            .unwrap();
        helper_statements.push(
            statements
                .declare(local.clone(), Some(initializer))
                .unwrap(),
        );
        expressions
            .read(
                expressions
                    .member(expressions.local(local.clone()).unwrap(), member.clone())
                    .unwrap(),
            )
            .unwrap()
    } else {
        value
    };
    helper_statements.push(statements.return_statement(Some(value)).unwrap());
    let helper_body = statements.block(helper_scope, helper_statements).unwrap();
    let header_builder = CDeclarations::new(&registry, header).unwrap();
    let mut header_items = vec![CFileItem::Declaration(
        header_builder
            .function_prototype(public.clone(), CLinkage::External)
            .unwrap(),
    )];
    if let Some((record, _, _)) = &record
        && matches!(layout, RecordLayout::Header)
    {
        header_items.insert(
            0,
            CFileItem::Declaration(
                header_builder
                    .aggregate(CAggregateRef::Struct(record.clone()))
                    .unwrap(),
            ),
        );
    }
    let header = header_builder.source_file(header_items).unwrap();
    let source_builder = CDeclarations::new(&registry, implementation).unwrap();
    let mut source_items = vec![
        CFileItem::Declaration(
            source_builder
                .function_prototype(helper.clone(), CLinkage::Internal)
                .unwrap(),
        ),
        CFileItem::Definition(
            source_builder
                .function_definition(
                    public.clone(),
                    CLinkage::External,
                    vec![public_parameter],
                    public_body,
                )
                .unwrap(),
        ),
        CFileItem::Definition(
            source_builder
                .function_definition(
                    helper.clone(),
                    CLinkage::Internal,
                    vec![helper_parameter],
                    helper_body,
                )
                .unwrap(),
        ),
    ];
    if let Some((record, _, _)) = &record
        && matches!(layout, RecordLayout::Implementation)
    {
        source_items.insert(
            0,
            CFileItem::Declaration(
                source_builder
                    .aggregate(CAggregateRef::Struct(record.clone()))
                    .unwrap(),
            ),
        );
    }
    let source = source_builder.source_file(source_items).unwrap();
    Fixture {
        registry: registry.freeze(),
        files: vec![header, source],
        public,
        helper,
        record: record.map(|(record, _, _)| record),
    }
}
