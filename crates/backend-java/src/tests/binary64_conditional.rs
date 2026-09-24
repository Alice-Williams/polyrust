//! Exact Double selection, nested zero handling and original imported authority.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum Selection {
    First,
    Second,
    Negative,
    Absolute,
}
impl Selection {
    pub(super) const ALL: [Self; 4] = [Self::First, Self::Second, Self::Negative, Self::Absolute];
    pub(super) fn expected(self, bits: u64) -> u64 {
        const SIGN: u64 = 1 << 63;
        let magnitude = bits & !SIGN;
        match self {
            Self::First => bits,
            Self::Second => bits ^ SIGN,
            Self::Negative
                if bits & SIGN != 0 && magnitude != 0 && magnitude <= 0x7ff0_0000_0000_0000 =>
            {
                bits
            }
            Self::Negative => bits ^ SIGN,
            Self::Absolute => magnitude,
        }
    }
}

fn zero() -> JavaExpr {
    JavaExpr::literal(
        double(),
        JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(0).unwrap()),
    )
}
fn branch(condition: JavaExpr, when_true: JavaExpr, when_false: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: double(),
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        },
    }
}
pub(super) fn select(mode: Selection, operand: JavaExpr) -> JavaExpr {
    let negative = super::negation::negate(operand.clone());
    let condition = |operator, precedence| JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Boolean),
        precedence,
        kind: JavaExprKind::Binary {
            operator,
            left: Box::new(operand.clone()),
            right: Box::new(zero()),
        },
    };
    match mode {
        Selection::First | Selection::Second => branch(
            JavaExpr::literal(
                JavaType::primitive(JavaPrimitive::Boolean),
                JavaLiteral::Boolean(matches!(mode, Selection::First)),
            ),
            operand,
            negative,
        ),
        Selection::Negative => branch(
            condition(JavaBinaryOperator::Less, JavaPrecedence::Relational),
            operand,
            negative,
        ),
        Selection::Absolute => {
            let magnitude = branch(
                condition(JavaBinaryOperator::Less, JavaPrecedence::Relational),
                negative,
                operand.clone(),
            );
            branch(
                condition(JavaBinaryOperator::Equal, JavaPrecedence::Equality),
                zero(),
                magnitude,
            )
        }
    }
}
pub(super) fn owner(crate_id: u64, dependencies: Option<&JavaDependencyApi>) -> JavaDependencyApi {
    let mut scope = JavaDependencyScope::new();
    let imports: Vec<_> = dependencies
        .into_iter()
        .flat_map(|owner| owner.functions())
        .collect();
    let mut declarations = Vec::new();
    for (index, mode) in Selection::ALL.into_iter().enumerate() {
        let input = JavaExpr::local(double(), f::name("input"));
        let operand = if let Some(function) = imports.get(index) {
            let (next, callable) = scope.import((*function).clone()).unwrap();
            scope = next;
            JavaExpr {
                ty: double(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable: JavaCallableRef::Dependency(callable),
                    receiver: None,
                    arguments: vec![input],
                },
            }
        } else {
            input
        };
        let local = f::name("operand");
        declarations.push(f::Function {
            hash: 10 + index as u64,
            public: true,
            name: f::name(&format!("select{index}")),
            parameters: vec![JavaParameter {
                ty: double(),
                name: f::name("input"),
                final_parameter: true,
            }],
            result: double(),
            body: JavaBlock::new(vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: double(),
                    name: local.clone(),
                    value: Some(operand),
                },
                JavaStmt::Return(Some(select(mode, JavaExpr::local(double(), local)))),
            ]),
        });
    }
    JavaDependencyApi::from_certificate(f::certify(f::package_with_dependencies(
        crate_id,
        declarations,
        scope.finish(),
    )))
    .unwrap()
}
#[test]
fn primitive_and_imported_double_conditionals_have_exact_certificates() {
    let first = owner(98, None);
    let second = owner(99, Some(&first));
    for api in [&first, &second] {
        assert_eq!(api.functions().count(), 4);
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
            assert!(
                !text.contains("Runtime")
                    && !text.contains("longBitsToDouble")
                    && !text.contains("0.0 -")
            );
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum Fault {
    Condition,
    Branch,
    Result,
    Precedence,
}
#[test]
fn counterfeit_double_conditional_types_and_precedence_reject() {
    for fault in [
        Fault::Condition,
        Fault::Branch,
        Fault::Result,
        Fault::Precedence,
    ] {
        let mut declarations = functions(&[]);
        let mut value = select(
            Selection::First,
            JavaExpr::local(double(), f::name("input")),
        );
        match fault {
            Fault::Result => value.ty = JavaType::primitive(JavaPrimitive::Long),
            Fault::Precedence => value.precedence = JavaPrecedence::Primary,
            Fault::Condition | Fault::Branch => {
                let JavaExprKind::Conditional {
                    condition,
                    when_true,
                    ..
                } = &mut value.kind
                else {
                    unreachable!()
                };
                match fault {
                    Fault::Condition => **condition = zero(),
                    Fault::Branch => when_true.ty = JavaType::primitive(JavaPrimitive::Long),
                    _ => unreachable!(),
                }
            }
        }
        declarations[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(value))]);
        match verify_unresolved_package(&JavaDialect, f::package(98, declarations)) {
            Err(_) => {}
            Ok(package) => {
                if let Ok(linked) = TargetLinker::new(JavaDialect).link_ast(&package)
                    && let Ok(certificate) = certify_resolved_package(&JavaDialect, linked)
                {
                    assert!(
                        JavaDependencyApi::from_certificate(certificate).is_err(),
                        "{fault:?}"
                    );
                }
            }
        }
    }
}
#[path = "binary64_conditional_native.rs"]
mod native;
