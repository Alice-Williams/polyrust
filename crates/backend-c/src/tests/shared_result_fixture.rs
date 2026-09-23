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
    let mut r = CRegistry::new();
    let file = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new("result.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let i32_ty = CObjectType::scalar(CScalarType::I32);
    let bool_ty = CObjectType::scalar(CScalarType::Bool);
    let record = r.declare_struct(&file, key("result")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let tag = r
        .register_member(&owner, key("success"), bool_ty.clone())
        .unwrap();
    let payload = r
        .register_member(&owner, key("payload"), i32_ty.clone())
        .unwrap();
    r.define_aggregate(&owner, vec![tag.clone(), payload.clone()])
        .unwrap();
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
    let functions: Vec<_> = signatures
        .iter()
        .map(|(name, result, params)| {
            r.register_function(
                &file,
                key(name),
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
        if index == 3 || matches!(mode, Mutation::Public) {
            CLinkage::External
        } else {
            CLinkage::Internal
        }
    };
    let d = CDeclarations::new(&r, file.clone()).unwrap();
    let aggregate = CFileItem::Declaration(d.aggregate(owner).unwrap());
    let mut items = vec![aggregate.clone()];
    for (index, function) in functions.iter().enumerate() {
        items.push(CFileItem::Declaration(
            d.function_prototype(function.clone(), linkage(index))
                .unwrap(),
        ));
    }
    if matches!(mode, Mutation::LateDeclaration) {
        items.remove(0);
        items.push(aggregate);
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
    (r.freeze(), source)
}
