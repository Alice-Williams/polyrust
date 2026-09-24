//! Selected-arm execution fixtures use ordinary certified imported result types.
use super::nominal_fixture::{self, Fixture, key};
use crate::{ast::*, dialect::*};
use portable_codegen::RelativeOutputPath;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub(super) enum Mutation {
    None,
    InactiveArm,
    DuplicateScrutinee,
    ArmBeforeScrutinee,
}

pub(super) struct BranchFixture {
    pub input: Fixture,
    pub entry: CFunctionRef,
    pub success: CFunctionRef,
    pub error: CFunctionRef,
}

struct EarlyArm {
    scope: CScopeRef,
    early: CLocalRef,
    selected: CLocalRef,
}

enum MainLocals {
    Normal {
        selected: CLocalRef,
        duplicate: Option<CLocalRef>,
        yes: CScopeRef,
        no: CScopeRef,
    },
    Before {
        yes: EarlyArm,
        no: Box<EarlyArm>,
    },
}

pub(super) fn fixture(producer: &CDependencyApi, mode: Mutation) -> BranchFixture {
    let proof = producer.structs().next().unwrap();
    let constructor = producer
        .functions()
        .find(|f| f.function().key().name.as_str() == "construct")
        .unwrap();
    let mut registry = CRegistry::new();
    registry.import_struct(proof.clone()).unwrap();
    let constructor = registry.import_function(constructor.clone()).unwrap();
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("polyrust_branch.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let source = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("branch.c").unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let integer = CObjectType::scalar(CScalarType::I32);
    let boolean = CObjectType::scalar(CScalarType::Bool);
    let result = CObjectType::structure(proof.record().clone());
    let names: Vec<_> = ["branch_entry", "branch_success", "branch_error"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let origins = nominal_fixture::origins(170, &names);
    let functions: Vec<_> = [
        vec![boolean, integer.clone()],
        vec![integer.clone()],
        vec![],
    ]
    .into_iter()
    .enumerate()
    .map(|(index, params)| {
        let mut key = key(&names[index]);
        key.origin = CGeneratedOrigin::RustSource(Arc::new(origins[index].clone()));
        registry
            .register_function(
                &header,
                key,
                CFunctionType::new(
                    CReturnType::Value(CReturnValue::new(integer.clone()).unwrap()),
                    params
                        .into_iter()
                        .map(|ty| CParameterType::new(ty).unwrap())
                        .collect(),
                ),
            )
            .unwrap()
    })
    .collect();
    let mut declarations = vec![];
    let mut definitions = vec![];
    for (index, function) in functions.iter().enumerate() {
        declarations.push(CFileItem::Declaration(
            CDeclarations::new(&registry, header.clone())
                .unwrap()
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        ));
        let scope = registry
            .register_scope(function, None, key("body"))
            .unwrap();
        let params: Vec<_> = (0..function.signature().parameters().len())
            .map(|position| {
                registry
                    .register_parameter(
                        function,
                        position,
                        key(&format!("input{position}")),
                        CConstness::Unqualified,
                    )
                    .unwrap()
            })
            .collect();
        let main_locals = (index == 0).then(|| {
            let yes = registry
                .register_scope(function, Some(&scope), key("yes"))
                .unwrap();
            let no = registry
                .register_scope(function, Some(&scope), key("no"))
                .unwrap();
            if matches!(mode, Mutation::ArmBeforeScrutinee) {
                let mut arm = |scope: CScopeRef| EarlyArm {
                    early: registry
                        .register_local(&scope, key("early"), integer.clone())
                        .unwrap(),
                    selected: registry
                        .register_local(&scope, key("selected"), result.clone())
                        .unwrap(),
                    scope,
                };
                MainLocals::Before {
                    yes: arm(yes),
                    no: Box::new(arm(no)),
                }
            } else {
                let selected = registry
                    .register_local(&scope, key("selected"), result.clone())
                    .unwrap();
                let duplicate = matches!(mode, Mutation::DuplicateScrutinee).then(|| {
                    registry
                        .register_local(&scope, key("duplicate"), result.clone())
                        .unwrap()
                });
                MainLocals::Normal {
                    selected,
                    duplicate,
                    yes,
                    no,
                }
            }
        });
        let e = CExpressions::new(&registry);
        let s = CStatements::new(&registry, function.clone()).unwrap();
        let read = |position: usize| {
            e.read(e.parameter(params[position].clone()).unwrap())
                .unwrap()
        };
        let literal = |value| {
            e.literal(CLiteral::Signed(CSignedLiteral::I32(value)))
                .unwrap()
        };
        let call = |target: &CFunctionRef, args| {
            e.call_value(e.direct(target.clone()).unwrap(), args)
                .unwrap()
        };
        let body = if index == 0 {
            let member = |local: CLocalRef, position: usize| {
                e.read(
                    e.member(e.local(local).unwrap(), proof.members()[position].clone())
                        .unwrap(),
                )
                .unwrap()
            };
            let declare = |local, value| {
                s.declare(local, Some(e.expression_initializer(value).unwrap()))
                    .unwrap()
            };
            let construct = || call(&constructor, vec![read(0), read(1)]);
            match main_locals.unwrap() {
                MainLocals::Before { yes, no } => {
                    let arm = |arm: EarlyArm, value| {
                        s.block(
                            arm.scope,
                            vec![
                                declare(arm.early.clone(), value),
                                declare(arm.selected.clone(), construct()),
                                s.discard(member(arm.selected, 0)).unwrap(),
                                s.return_statement(Some(
                                    e.read(e.local(arm.early).unwrap()).unwrap(),
                                ))
                                .unwrap(),
                            ],
                        )
                        .unwrap()
                    };
                    s.block(
                        scope,
                        vec![
                            s.if_statement(
                                read(0),
                                arm(yes, call(&functions[1], vec![read(1)])),
                                arm(*no, call(&functions[2], vec![])),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap()
                }
                MainLocals::Normal {
                    selected,
                    duplicate,
                    yes,
                    no,
                } => {
                    let mut statements = vec![declare(selected.clone(), construct())];
                    if let Some(duplicate) = duplicate {
                        statements.push(declare(duplicate.clone(), construct()));
                        statements.push(s.discard(member(duplicate, 0)).unwrap());
                    }
                    let arm = |block, active: usize| {
                        let invoke = |target| {
                            if target == 1 {
                                call(&functions[1], vec![member(selected.clone(), 1)])
                            } else {
                                call(&functions[2], vec![])
                            }
                        };
                        let mut body = vec![];
                        if matches!(mode, Mutation::InactiveArm) {
                            body.push(s.discard(invoke(3 - active)).unwrap());
                        }
                        body.push(s.return_statement(Some(invoke(active))).unwrap());
                        s.block(block, body).unwrap()
                    };
                    statements.push(
                        s.if_statement(member(selected.clone(), 0), arm(yes, 1), arm(no, 2))
                            .unwrap(),
                    );
                    s.block(scope, statements).unwrap()
                }
            }
        } else {
            s.block(
                scope,
                vec![
                    s.return_statement(Some(if index == 1 { read(0) } else { literal(17) }))
                        .unwrap(),
                ],
            )
            .unwrap()
        };
        definitions.push(CFileItem::Definition(
            CDeclarations::new(&registry, source.clone())
                .unwrap()
                .function_definition(function.clone(), CLinkage::External, params, body)
                .unwrap(),
        ));
    }
    let files = vec![
        CDeclarations::new(&registry, header)
            .unwrap()
            .source_file(declarations)
            .unwrap(),
        CDeclarations::new(&registry, source)
            .unwrap()
            .source_file(definitions)
            .unwrap(),
    ];
    BranchFixture {
        entry: functions[0].clone(),
        success: functions[1].clone(),
        error: functions[2].clone(),
        input: Fixture {
            registry: registry.freeze(),
            files,
            functions,
        },
    }
}
