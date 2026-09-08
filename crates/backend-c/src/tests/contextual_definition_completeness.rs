//! By-value definition and dereference-write completeness through the full entry point.
use super::contextual_reconstruction::key;
use super::*;

#[derive(Clone, Copy, Debug)]
enum Use {
    Object,
    Parameter,
    Return,
    Write,
}

#[test]
fn definitions_and_actual_writes_require_complete_objects_through_aliases() {
    for usage in [Use::Object, Use::Parameter, Use::Return, Use::Write] {
        for complete in [false, true] {
            for aliased in [false, true] {
                let mut registry = CRegistry::new();
                let file = registry
                    .register_file(CFileKey {
                        role: CFileRole::TestSource,
                        path: portable_codegen::RelativeOutputPath::new("complete.c").unwrap(),
                    })
                    .unwrap();
                let record = registry.declare_struct(&file, key("Record")).unwrap();
                let owner = CAggregateRef::Struct(record.clone());
                if complete {
                    let member = registry
                        .register_member(
                            &owner,
                            key("field"),
                            CObjectType::scalar(CScalarType::I32),
                        )
                        .unwrap();
                    registry.define_aggregate(&owner, vec![member]).unwrap();
                }
                let raw = CObjectType::structure(record);
                let alias = aliased.then(|| {
                    registry
                        .register_typedef(&file, key("Alias"), raw.clone())
                        .unwrap()
                });
                let ty = alias
                    .as_ref()
                    .map_or(raw, |alias| CObjectType::typedef(alias.clone()));
                let item = if matches!(usage, Use::Object) {
                    let object = registry
                        .register_object(&file, key("storage"), ty.clone())
                        .unwrap();
                    CFileItem::Definition(
                        CDeclarations::new(&registry, file.clone())
                            .unwrap()
                            .object_definition(
                                object,
                                CLinkage::External,
                                CExpressions::new(&registry).zero_initializer(ty).unwrap(),
                            )
                            .unwrap(),
                    )
                } else {
                    let parameter_type = if matches!(usage, Use::Parameter) {
                        ty.clone()
                    } else {
                        CObjectType::pointer(CPointerTarget::Object(Box::new(ty.clone())))
                    };
                    let result = if matches!(usage, Use::Return) {
                        CReturnType::Value(CReturnValue::new(ty).unwrap())
                    } else {
                        CReturnType::Void
                    };
                    let function = registry
                        .register_function(
                            &file,
                            key("run"),
                            CFunctionType::new(
                                result,
                                vec![CParameterType::new(parameter_type).unwrap()],
                            ),
                        )
                        .unwrap();
                    let parameter = registry
                        .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
                        .unwrap();
                    let scope = registry
                        .register_scope(&function, None, key("root"))
                        .unwrap();
                    let values = CExpressions::new(&registry);
                    let input = values
                        .read(values.parameter(parameter.clone()).unwrap())
                        .unwrap();
                    let statements = CStatements::new(&registry, function.clone()).unwrap();
                    let statement = match usage {
                        Use::Parameter => statements.discard(input).unwrap(),
                        Use::Return => statements
                            .return_statement(Some(
                                values.read(values.dereference(input).unwrap()).unwrap(),
                            ))
                            .unwrap(),
                        Use::Write => {
                            let place = values.dereference(input).unwrap();
                            let assignment =
                                statements.assign(place.clone(), values.read(place).unwrap());
                            if !complete {
                                // Mutability construction already requires the aggregate layout.
                                assert_eq!(assignment, Err(CStatementError::IncompleteAggregate));
                                continue;
                            }
                            assignment.unwrap()
                        }
                        Use::Object => unreachable!(),
                    };
                    let body = statements.block(scope, vec![statement]).unwrap();
                    CFileItem::Definition(
                        CDeclarations::new(&registry, file.clone())
                            .unwrap()
                            .function_definition(
                                function,
                                CLinkage::External,
                                vec![parameter],
                                body,
                            )
                            .unwrap(),
                    )
                };
                let declarations = CDeclarations::new(&registry, file).unwrap();
                let mut items = vec![CFileItem::Declaration(if complete {
                    declarations.aggregate(owner).unwrap()
                } else {
                    declarations.forward_tag(owner).unwrap()
                })];
                if let Some(alias) = alias {
                    items.push(CFileItem::Declaration(declarations.typedef(alias).unwrap()));
                }
                items.push(item);
                let source = declarations.source_file(items).unwrap();
                registry
                    .check_local_structure(std::slice::from_ref(&source))
                    .unwrap();
                assert_eq!(
                    registry.check_context(&[source]),
                    if complete {
                        Ok(())
                    } else {
                        Err(CContextError::IncompleteObject)
                    },
                    "{usage:?}, complete={complete}, alias={aliased}"
                );
            }
        }
    }
}
