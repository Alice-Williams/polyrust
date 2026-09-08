//! Initialization joins must preserve derived aggregate coverage on both paths.
use super::contextual_reconstruction::{int, key};
use super::*;

#[derive(Clone, Copy, Debug)]
enum Shape {
    Union,
    Struct,
    ArrayOfUnions,
}

fn fixture() -> (CRegistry, CFileRef, CFunctionRef, CScopeRef, CParameterRef) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: portable_codegen::RelativeOutputPath::new("join.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let function = registry
        .register_function(
            &file,
            key("run"),
            CFunctionType::new(
                CReturnType::Void,
                vec![CParameterType::new(CObjectType::scalar(CScalarType::Bool)).unwrap()],
            ),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let condition = registry
        .register_parameter(&function, 0, key("condition"), CConstness::Unqualified)
        .unwrap();
    (registry, file, function, scope, condition)
}

#[test]
fn aggregate_joins_preserve_common_coverage_without_inventing_unwritten_siblings() {
    for shape in [Shape::Union, Shape::Struct, Shape::ArrayOfUnions] {
        for initialize_else in [true, false] {
            for read_sibling in [false, true] {
                if read_sibling && !matches!(shape, Shape::ArrayOfUnions) {
                    continue;
                }
                let (mut registry, file, function, scope, condition) = fixture();
                let (owner, object) = match shape {
                    Shape::Struct => {
                        let value = registry.declare_struct(&file, key("Record")).unwrap();
                        (
                            CAggregateRef::Struct(value.clone()),
                            CObjectType::structure(value),
                        )
                    }
                    Shape::Union | Shape::ArrayOfUnions => {
                        let value = registry.declare_union(&file, key("Payload")).unwrap();
                        (
                            CAggregateRef::Union(value.clone()),
                            CObjectType::union(value),
                        )
                    }
                };
                let members = ["first", "second"].map(|name| {
                    registry
                        .register_member(&owner, key(name), CObjectType::scalar(CScalarType::I32))
                        .unwrap()
                });
                registry.define_aggregate(&owner, members.to_vec()).unwrap();
                let ty = if matches!(shape, Shape::ArrayOfUnions) {
                    CObjectType::array(object, CArrayLength::new(2).unwrap()).unwrap()
                } else {
                    object
                };
                let local = registry.register_local(&scope, key("storage"), ty).unwrap();
                let branches = ["then_branch", "else_branch"].map(|name| {
                    registry
                        .register_scope(&function, Some(&scope), key(name))
                        .unwrap()
                });
                let values = CExpressions::new(&registry);
                let ast = CStatements::new(&registry, function.clone()).unwrap();
                let selected = |index| {
                    let base = values.local(local.clone()).unwrap();
                    if matches!(shape, Shape::ArrayOfUnions) {
                        values
                            .index(
                                CIndexBase::Array(Box::new(base)),
                                values
                                    .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(index)))
                                    .unwrap(),
                            )
                            .unwrap()
                    } else {
                        base
                    }
                };
                let then_write = ast
                    .assign(
                        values.member(selected(0), members[0].clone()).unwrap(),
                        int(&values),
                    )
                    .unwrap();
                let else_write = ast
                    .assign(
                        values.member(selected(0), members[1].clone()).unwrap(),
                        int(&values),
                    )
                    .unwrap();
                let conditional = ast
                    .if_statement(
                        values
                            .read(values.parameter(condition.clone()).unwrap())
                            .unwrap(),
                        ast.block(branches[0].clone(), vec![then_write]).unwrap(),
                        ast.block(
                            branches[1].clone(),
                            if initialize_else {
                                vec![else_write]
                            } else {
                                vec![]
                            },
                        )
                        .unwrap(),
                    )
                    .unwrap();
                let read = values.read(selected(u64::from(read_sibling))).unwrap();
                let body = ast
                    .block(
                        scope,
                        vec![
                            ast.declare(local, None).unwrap(),
                            conditional,
                            ast.discard(read).unwrap(),
                        ],
                    )
                    .unwrap();
                let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
                let mut source = declarations
                    .source_file(vec![CFileItem::Definition(
                        declarations
                            .function_definition(
                                function,
                                CLinkage::External,
                                vec![condition],
                                body,
                            )
                            .unwrap(),
                    )])
                    .unwrap();
                source.items.push(CFileItem::Declaration(
                    CDeclarations::new(&registry, file)
                        .unwrap()
                        .aggregate(owner)
                        .unwrap(),
                ));
                let expected =
                    if initialize_else && !read_sibling && !matches!(shape, Shape::Struct) {
                        Ok(())
                    } else {
                        Err(CContextError::UninitializedRead)
                    };
                assert_eq!(
                    registry.check_context(&[source]),
                    expected,
                    "{shape:?}/else={initialize_else}/sibling={read_sibling}"
                );
                // Active union-member access remains 02D. Reading/copying the
                // initialized union object must not require one selected member.
            }
        }
    }
}
