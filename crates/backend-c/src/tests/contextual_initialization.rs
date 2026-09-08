//! Reachable predecessor intersections and exact storage initialization.

use super::contextual_reconstruction::{fixture, fixture_return, int, key, package};
use super::*;

fn scalar() -> CObjectType {
    CObjectType::scalar(CScalarType::I32)
}

#[test]
fn self_initialization_is_rejected_but_an_address_is_not_a_value_read() {
    let (mut registry, file, function, scope) = fixture();
    let local = registry
        .register_local(&scope, key("slot"), scalar())
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let place = values.local(local.clone()).unwrap();
    let address = ast
        .discard(values.address_of(place.clone()).unwrap())
        .unwrap();
    let source = |statements| {
        package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            statements,
        )
    };
    registry
        .check_context(&[source(vec![
            ast.declare(local.clone(), None).unwrap(),
            address,
        ])])
        .unwrap();
    let declaration = ast
        .declare(
            local,
            Some(
                values
                    .expression_initializer(values.read(place).unwrap())
                    .unwrap(),
            ),
        )
        .unwrap();
    assert_eq!(
        registry.check_context(&[source(vec![declaration])]),
        Err(CContextError::UninitializedRead)
    );
}

#[test]
fn every_continuing_branch_must_initialize_but_returned_branches_do_not_join() {
    for (initialize_else, return_else) in [(true, false), (false, false), (false, true)] {
        let (mut registry, file, function, scope) = fixture();
        let yes = registry
            .register_scope(&function, Some(&scope), key("yes"))
            .unwrap();
        let no = registry
            .register_scope(&function, Some(&scope), key("no"))
            .unwrap();
        let local = registry
            .register_local(&scope, key("slot"), scalar())
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let place = values.local(local.clone()).unwrap();
        let assignment = ast.assign(place.clone(), int(&values)).unwrap();
        let else_statements = if return_else {
            vec![ast.return_statement(None).unwrap()]
        } else if initialize_else {
            vec![assignment.clone()]
        } else {
            vec![]
        };
        let branch = ast
            .if_statement(
                values.literal(CLiteral::Bool(true)).unwrap(),
                ast.block(yes, vec![assignment]).unwrap(),
                ast.block(no, else_statements).unwrap(),
            )
            .unwrap();
        let body = vec![
            ast.declare(local, None).unwrap(),
            branch,
            ast.discard(values.read(place).unwrap()).unwrap(),
        ];
        let result = registry.check_context(&[package(&registry, file, function, scope, body)]);
        if initialize_else || return_else {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::UninitializedRead));
        }
    }
}

#[test]
fn unreachable_reads_do_not_poison_flow_but_nonvoid_fallthrough_is_rejected() {
    let (mut registry, file, function, scope) = fixture();
    let local = registry
        .register_local(&scope, key("slot"), scalar())
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let body = vec![
        ast.declare(local.clone(), None).unwrap(),
        ast.return_statement(None).unwrap(),
        ast.discard(values.read(values.local(local).unwrap()).unwrap())
            .unwrap(),
    ];
    registry
        .check_context(&[package(&registry, file, function, scope, body)])
        .unwrap();

    let (registry, file, function, scope) =
        fixture_return(CReturnType::Value(CReturnValue::new(scalar()).unwrap()));
    assert_eq!(
        registry.check_context(&[package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            vec![]
        )]),
        Err(CContextError::MissingReturn)
    );
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    registry
        .check_context(&[package(
            &registry,
            file,
            function,
            scope,
            vec![ast.return_statement(Some(int(&values))).unwrap()],
        )])
        .unwrap();
}

#[test]
fn switch_requires_explicit_exits_from_every_reachable_arm() {
    for explicit_exit in [true, false] {
        let (mut registry, file, function, scope) = fixture();
        let arm_scope = registry
            .register_scope(&function, Some(&scope), key("arm"))
            .unwrap();
        let default_scope = registry
            .register_scope(&function, Some(&scope), key("otherwise"))
            .unwrap();
        let switch = registry.register_switch(&scope, key("choice")).unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let exit = ast
            .break_statement(CBreakTarget::Switch(switch.clone()))
            .unwrap();
        let arm = ast
            .switch_arm(
                vec![CCaseConstant::Signed(CSignedLiteral::Int(0))],
                ast.block(
                    arm_scope,
                    if explicit_exit {
                        vec![exit.clone()]
                    } else {
                        vec![]
                    },
                )
                .unwrap(),
            )
            .unwrap();
        let selection = ast
            .switch_statement(
                switch,
                int(&values),
                vec![arm],
                ast.block(default_scope, vec![exit]).unwrap(),
            )
            .unwrap();
        let result =
            registry.check_context(&[package(&registry, file, function, scope, vec![selection])]);
        if explicit_exit {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::SwitchFallthrough));
        }
    }
}

#[test]
fn field_coverage_joins_whole_assignments_without_initializing_unwritten_siblings() {
    for (write_second, whole_on_else) in [(false, false), (true, false), (true, true)] {
        let (mut registry, file, function, scope) = fixture();
        let record = registry.declare_struct(&file, key("Pair")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let first = registry
            .register_member(&owner, key("first"), scalar())
            .unwrap();
        let second = registry
            .register_member(&owner, key("second"), scalar())
            .unwrap();
        registry
            .define_aggregate(&owner, vec![first.clone(), second.clone()])
            .unwrap();
        let ty = CObjectType::structure(record);
        let local = registry
            .register_local(&scope, key("pair"), ty.clone())
            .unwrap();
        let source_local = whole_on_else.then(|| {
            registry
                .register_local(&scope, key("source"), ty.clone())
                .unwrap()
        });
        let yes = registry
            .register_scope(&function, Some(&scope), key("yes"))
            .unwrap();
        let no = registry
            .register_scope(&function, Some(&scope), key("no"))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let place = values.local(local.clone()).unwrap();
        let write_first = ast
            .assign(values.member(place.clone(), first).unwrap(), int(&values))
            .unwrap();
        let write_second_stmt = ast
            .assign(values.member(place.clone(), second).unwrap(), int(&values))
            .unwrap();
        let mut writes = vec![write_first];
        if write_second {
            writes.push(write_second_stmt);
        }
        let mut declarations = vec![ast.declare(local, None).unwrap()];
        let else_body = if let Some(source) = source_local {
            declarations.push(
                ast.declare(source.clone(), Some(values.zero_initializer(ty).unwrap()))
                    .unwrap(),
            );
            vec![
                ast.assign(
                    place.clone(),
                    values.read(values.local(source).unwrap()).unwrap(),
                )
                .unwrap(),
            ]
        } else {
            writes.clone()
        };
        let branch = ast
            .if_statement(
                values.literal(CLiteral::Bool(true)).unwrap(),
                ast.block(yes, writes).unwrap(),
                ast.block(no, else_body).unwrap(),
            )
            .unwrap();
        declarations.push(branch);
        declarations.push(ast.discard(values.read(place).unwrap()).unwrap());
        let mut source = package(&registry, file.clone(), function, scope, declarations);
        source.items.insert(
            0,
            CFileItem::Declaration(
                CDeclarations::new(&registry, file)
                    .unwrap()
                    .aggregate(owner)
                    .unwrap(),
            ),
        );
        let result = registry.check_context(&[source]);
        if write_second {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::UninitializedRead));
        }
    }
}
