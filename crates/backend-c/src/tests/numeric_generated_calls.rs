//! Generated prototypes do not confer numeric summaries; classifiers are new values.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, *};
use crate::dialect::CKnownCall;

#[test]
fn generated_return_history_is_unproved_until_a_body_summary_exists() {
    for indirect in [false, true] {
        for classifier in 0..3 {
            let mut f = Fixture::new(&[]);
            let size = CObjectType::scalar(CScalarType::Size);
            let callee = f
                .registry
                .register_function(
                    &f.file,
                    key("calculate"),
                    CFunctionType::new(
                        CReturnType::Value(CReturnValue::new(size).unwrap()),
                        vec![],
                    ),
                )
                .unwrap();
            let scope = f
                .registry
                .register_scope(&callee, None, key("callee_root"))
                .unwrap();
            let local = f.local(CScalarType::Size, "result");
            let callable = if indirect {
                f.values()
                    .indirect(
                        f.values().function_address(callee.clone()).unwrap(),
                        callee.clone(),
                    )
                    .unwrap()
            } else {
                f.values().direct(callee.clone()).unwrap()
            };
            let call = f.values().call_value(callable, vec![]).unwrap();
            let value = match classifier {
                0 => f.read(&local),
                1 => f
                    .values()
                    .numeric_conversion(
                        CScalarType::Size,
                        f.compare(CBinaryOperator::Greater, f.read(&local), f.size(0)),
                    )
                    .unwrap(),
                _ => f
                    .values()
                    .numeric_conversion(CScalarType::Size, f.boolean(f.read(&local)))
                    .unwrap(),
            };
            let allocation = f.discard(
                f.values()
                    .call_value(f.values().known(CKnownCall::Allocate), vec![value])
                    .unwrap(),
            );
            let mut source = f.source(vec![f.declare(&local, call), allocation]);
            let ast = CStatements::new(&f.registry, callee.clone()).unwrap();
            let body = ast
                .block(scope, vec![ast.return_statement(Some(f.size(1))).unwrap()])
                .unwrap();
            source.items.insert(
                0,
                CFileItem::Definition(
                    CDeclarations::new(&f.registry, f.file.clone())
                        .unwrap()
                        .function_definition(callee, CLinkage::External, vec![], body)
                        .unwrap(),
                ),
            );
            assert_eq!(
                f.registry.check_numeric_flow(&[source]),
                if classifier == 0 {
                    Err(CSafetyError::UnprovedSizeArithmetic)
                } else {
                    Ok(())
                }
            );
        }
    }
}
