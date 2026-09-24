//! Private by-value result helpers behind a scalar external entry point.
use super::tests::key;
use crate::ast::*;
use portable_codegen::RelativeOutputPath;

#[derive(Clone, Copy, Debug, Default)]
pub(super) enum Mutation {
    #[default]
    None,
    Public,
    LateDeclaration,
    Recursive,
    SwappedTag,
    LostPayload,
    Uninitialized,
}

pub(super) fn fixture(mode: Mutation) -> (CFrozenRegistry, CSourceFile) {
    let (registry, mut files) = build(mode, None, false);
    (registry, files.pop().unwrap())
}

#[derive(Clone, Copy)]
pub(super) enum PublicApi {
    ResultSignatures,
    ScalarOnly,
}

pub(super) fn public_fixture(
    mode: Mutation,
    api: PublicApi,
) -> (CFrozenRegistry, Vec<CSourceFile>) {
    build(mode, Some(api), false)
}

pub(super) fn public_collision_fixture() -> (CFrozenRegistry, Vec<CSourceFile>) {
    build(Mutation::None, Some(PublicApi::ResultSignatures), true)
}

fn build(
    mode: Mutation,
    api: Option<PublicApi>,
    collisions: bool,
) -> (CFrozenRegistry, Vec<CSourceFile>) {
    let key = |name: &str| {
        let mut key = key(name);
        if api.is_some() {
            key.origin = CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter);
        }
        key
    };
    let mut r = CRegistry::new();
    let file = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new("result.c").unwrap(),
            role: if api.is_some() {
                CFileRole::GeneratedSource
            } else {
                CFileRole::TestSource
            },
        })
        .unwrap();
    let header = api.map(|_| {
        r.register_file(CFileKey {
            path: RelativeOutputPath::new("polyrust_result.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap()
    });
    let public =
        |index| index == 3 || (matches!(api, Some(PublicApi::ResultSignatures)) && index != 1);
    let i32_ty = CObjectType::scalar(CScalarType::I32);
    let bool_ty = CObjectType::scalar(CScalarType::Bool);
    let record = r
        .declare_struct(header.as_ref().unwrap_or(&file), key("result"))
        .unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let tag = r
        .register_member(&owner, key("success"), bool_ty.clone())
        .unwrap();
    let payload = r
        .register_member(
            &owner,
            key(if collisions { "entry" } else { "payload" }),
            i32_ty.clone(),
        )
        .unwrap();
    r.define_aggregate(&owner, vec![tag.clone(), payload.clone()])
        .unwrap();
    let second = collisions.then(|| {
        let record = r
            .declare_struct(header.as_ref().unwrap(), key("other_result"))
            .unwrap();
        let owner = CAggregateRef::Struct(record);
        let tag = r
            .register_member(&owner, key("success"), bool_ty.clone())
            .unwrap();
        let payload = r
            .register_member(&owner, key("entry"), i32_ty.clone())
            .unwrap();
        r.define_aggregate(&owner, vec![tag, payload]).unwrap();
        owner
    });
    let ty = CObjectType::structure(record);
    let signatures = [
        (
            "construct",
            ty.clone(),
            vec![bool_ty.clone(), i32_ty.clone()],
        ),
        ("forward", ty.clone(), vec![ty.clone()]),
        ("inspect", i32_ty.clone(), vec![ty.clone(), bool_ty.clone()]),
        (
            "entry",
            i32_ty.clone(),
            vec![bool_ty.clone(), i32_ty.clone(), bool_ty],
        ),
    ];
    let origins = api.map(|_| public_origins(&public));
    let functions: Vec<_> = signatures
        .iter()
        .enumerate()
        .map(|(index, (name, result, params))| {
            let mut function_key = key(name);
            if let Some(origins) = &origins {
                function_key.origin =
                    CGeneratedOrigin::RustSource(std::sync::Arc::new(origins[index].clone()));
            }
            r.register_function(
                if public(index) {
                    header.as_ref().unwrap_or(&file)
                } else {
                    &file
                },
                function_key,
                CFunctionType::new(
                    CReturnType::Value(CReturnValue::new(result.clone()).unwrap()),
                    params
                        .iter()
                        .map(|ty| CParameterType::new(ty.clone()).unwrap())
                        .collect(),
                ),
            )
            .unwrap()
        })
        .collect();
    let linkage = |index| {
        if public(index) || matches!(mode, Mutation::Public) {
            CLinkage::External
        } else {
            CLinkage::Internal
        }
    };
    let d = CDeclarations::new(&r, header.as_ref().unwrap_or(&file).clone()).unwrap();
    let aggregate = CFileItem::Declaration(d.aggregate(owner).unwrap());
    let mut items = vec![];
    let mut header_items = vec![];
    if header.is_some() {
        header_items.push(aggregate.clone());
        if let Some(owner) = second {
            header_items.push(CFileItem::Declaration(d.aggregate(owner).unwrap()));
        }
    } else {
        items.push(aggregate.clone());
    }
    for (index, function) in functions.iter().enumerate() {
        let prototype = CFileItem::Declaration(
            CDeclarations::new(&r, function.file().clone())
                .unwrap()
                .function_prototype(function.clone(), linkage(index))
                .unwrap(),
        );
        if header.as_ref() == Some(function.file()) {
            header_items.push(prototype);
        } else {
            items.push(prototype);
        }
    }
    if matches!(mode, Mutation::LateDeclaration) {
        let declarations = if header.is_some() {
            &mut header_items
        } else {
            &mut items
        };
        declarations.remove(0);
        declarations.push(aggregate);
    }
    for (index, function) in functions.iter().enumerate() {
        let scope = r.register_scope(function, None, key("body")).unwrap();
        let parameters: Vec<_> = (0..signatures[index].2.len())
            .map(|position| {
                r.register_parameter(
                    function,
                    position,
                    key(&format!("input{position}")),
                    CConstness::Unqualified,
                )
                .unwrap()
            })
            .collect();
        let local =
            (index != 2).then(|| r.register_local(&scope, key("value"), ty.clone()).unwrap());
        let forwarded = (index == 3).then(|| {
            r.register_local(&scope, key("forwarded"), ty.clone())
                .unwrap()
        });
        let branches = (index == 2).then(|| {
            let yes = r
                .register_scope(function, Some(&scope), key("yes"))
                .unwrap();
            let no = r.register_scope(function, Some(&scope), key("no")).unwrap();
            let nested_yes = r
                .register_scope(function, Some(&no), key("nested_yes"))
                .unwrap();
            let nested_no = r
                .register_scope(function, Some(&no), key("nested_no"))
                .unwrap();
            (yes, no, nested_yes, nested_no)
        });
        let e = CExpressions::new(&r);
        let s = CStatements::new(&r, function.clone()).unwrap();
        let read = |position: usize| {
            e.read(e.parameter(parameters[position].clone()).unwrap())
                .unwrap()
        };
        let integer = |value| {
            e.literal(CLiteral::Signed(CSignedLiteral::I32(value)))
                .unwrap()
        };
        let return_block = |scope, value| {
            s.block(scope, vec![s.return_statement(Some(value)).unwrap()])
                .unwrap()
        };
        let call = |target: usize, values| {
            e.call_value(e.direct(functions[target].clone()).unwrap(), values)
                .unwrap()
        };
        let body = match index {
            0 => {
                let local = local.unwrap();
                let tag_value = if matches!(mode, Mutation::SwappedTag) {
                    e.numeric_conversion(
                        CScalarType::Bool,
                        e.unary(CUnaryOperator::LogicalNot, read(0)).unwrap(),
                    )
                    .unwrap()
                } else {
                    read(0)
                };
                let payload_value = if matches!(mode, Mutation::LostPayload) {
                    integer(0)
                } else {
                    read(1)
                };
                let CObjectTypeKind::Struct(record) = ty.kind() else {
                    unreachable!()
                };
                let initializer = e
                    .struct_initializer(
                        record.clone(),
                        vec![
                            (tag.clone(), e.expression_initializer(tag_value).unwrap()),
                            (
                                payload.clone(),
                                e.expression_initializer(payload_value).unwrap(),
                            ),
                        ],
                    )
                    .unwrap();
                let mut statements = vec![
                    s.discard(read(0)).unwrap(),
                    s.discard(read(1)).unwrap(),
                    s.declare(
                        local.clone(),
                        (!matches!(mode, Mutation::Uninitialized)).then_some(initializer),
                    )
                    .unwrap(),
                ];
                statements.push(
                    s.return_statement(Some(e.read(e.local(local).unwrap()).unwrap()))
                        .unwrap(),
                );
                s.block(scope, statements).unwrap()
            }
            1 => {
                let local = local.unwrap();
                let value = if matches!(mode, Mutation::Recursive) {
                    call(1, vec![read(0)])
                } else {
                    read(0)
                };
                // An explicit copy proves initialization survives aggregate transport.
                let initial = e.expression_initializer(value).unwrap();
                s.block(
                    scope,
                    vec![
                        s.declare(local.clone(), Some(initial)).unwrap(),
                        s.return_statement(Some(e.read(e.local(local).unwrap()).unwrap()))
                            .unwrap(),
                    ],
                )
                .unwrap()
            }
            2 => {
                let (yes, no, nested_yes, nested_no) = branches.unwrap();
                let input = e.parameter(parameters[0].clone()).unwrap();
                let tag_value = e
                    .read(e.member(input.clone(), tag.clone()).unwrap())
                    .unwrap();
                let payload_value = e.read(e.member(input, payload.clone()).unwrap()).unwrap();
                let by_tag = s
                    .if_statement(
                        tag_value.clone(),
                        return_block(nested_yes, payload_value),
                        return_block(nested_no, integer(17)),
                    )
                    .unwrap();
                s.block(
                    scope,
                    vec![
                        s.if_statement(
                            read(1),
                            return_block(
                                yes,
                                e.numeric_conversion(CScalarType::I32, tag_value).unwrap(),
                            ),
                            s.block(no, vec![by_tag]).unwrap(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap()
            }
            3 => {
                let local = local.unwrap();
                let constructed = call(0, vec![read(0), read(1)]);
                // Sequence aggregate calls through materialized local values.
                let initial = e.expression_initializer(constructed).unwrap();
                let copied = call(1, vec![e.read(e.local(local.clone()).unwrap()).unwrap()]);
                let forwarded = forwarded.unwrap();
                let copied_initial = e.expression_initializer(copied).unwrap();
                let value = call(
                    2,
                    vec![
                        e.read(e.local(forwarded.clone()).unwrap()).unwrap(),
                        read(2),
                    ],
                );
                s.block(
                    scope,
                    vec![
                        s.declare(local, Some(initial)).unwrap(),
                        s.declare(forwarded, Some(copied_initial)).unwrap(),
                        s.return_statement(Some(value)).unwrap(),
                    ],
                )
                .unwrap()
            }
            _ => unreachable!(),
        };
        items.push(CFileItem::Definition(
            CDeclarations::new(&r, file.clone())
                .unwrap()
                .function_definition(function.clone(), linkage(index), parameters, body)
                .unwrap(),
        ));
    }
    let source = CDeclarations::new(&r, file)
        .unwrap()
        .source_file(items)
        .unwrap();
    let mut files = vec![];
    if let Some(header) = header {
        files.push(
            CDeclarations::new(&r, header)
                .unwrap()
                .source_file(header_items)
                .unwrap(),
        );
    }
    files.push(source);
    (r.freeze(), files)
}

fn public_origins(public: &impl Fn(usize) -> bool) -> Vec<portable_codegen::RustSourceOrigin> {
    use portable_codegen::*;
    use std::{collections::BTreeMap, sync::Arc};
    let (source, helper) = super::package_source_fixture::origins();
    let mut exports = (*source.crate_exports).clone();
    let mut entries = BTreeMap::new();
    for (index, name) in ["construct", "forward", "inspect", "entry"]
        .iter()
        .enumerate()
    {
        if public(index) {
            entries.insert(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: (*name).into(),
                },
                RustExportTarget::Declaration(RustDeclarationId {
                    crate_id: source.declaration.crate_id,
                    definition_path_hash: 3 + index as u64,
                }),
            );
        }
    }
    exports.modules.insert(exports.root, entries);
    let exports = Arc::new(exports);
    (0..4)
        .map(|index| RustSourceOrigin {
            declaration: RustDeclarationId {
                crate_id: source.declaration.crate_id,
                definition_path_hash: 3 + index,
            },
            externally_reachable: public(index as usize),
            visibility: if public(index as usize) {
                RustVisibility::Public
            } else {
                helper.visibility
            },
            crate_exports: exports.clone(),
            ..source.clone()
        })
        .collect()
}
