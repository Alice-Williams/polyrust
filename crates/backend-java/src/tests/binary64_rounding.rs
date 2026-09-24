//! Exact catalogue-owned rounding primitives and typed truncation composition.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum Mode {
    Floor,
    Ceil,
    Truncate,
}
impl Mode {
    pub(super) const ALL: [Self; 3] = [Self::Floor, Self::Ceil, Self::Truncate];
    pub(super) fn expected(self, bits: u64) -> u64 {
        let sign = bits & (1 << 63);
        let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
        if exponent >= 52 {
            return bits;
        }
        let truncated = if exponent < 0 {
            sign
        } else {
            bits & !((1_u64 << (52 - exponent)) - 1)
        };
        if truncated == bits || matches!(self, Self::Truncate) {
            return truncated;
        }
        let away =
            matches!(self, Self::Floor) && sign != 0 || matches!(self, Self::Ceil) && sign == 0;
        if !away {
            return truncated;
        }
        if exponent < 0 {
            sign | 0x3ff0_0000_0000_0000
        } else {
            truncated + (1_u64 << (52 - exponent))
        }
    }
}
fn primitive(callable: JavaKnownCallable, operand: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: double(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable,
                signature: callable.signature(),
            },
            receiver: None,
            arguments: vec![operand],
        },
    }
}
pub(super) fn round(mode: Mode, operand: JavaExpr) -> JavaExpr {
    match mode {
        Mode::Floor => primitive(JavaKnownCallable::MathFloor, operand),
        Mode::Ceil => primitive(JavaKnownCallable::MathCeil, operand),
        Mode::Truncate => JavaExpr {
            ty: double(),
            precedence: JavaPrecedence::Conditional,
            kind: JavaExprKind::Conditional {
                condition: Box::new(JavaExpr {
                    ty: JavaType::primitive(JavaPrimitive::Boolean),
                    precedence: JavaPrecedence::Relational,
                    kind: JavaExprKind::Binary {
                        operator: JavaBinaryOperator::Less,
                        left: Box::new(operand.clone()),
                        right: Box::new(JavaExpr::literal(
                            double(),
                            JavaLiteral::F64(
                                portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
                            ),
                        )),
                    },
                }),
                when_true: Box::new(primitive(JavaKnownCallable::MathCeil, operand.clone())),
                when_false: Box::new(primitive(JavaKnownCallable::MathFloor, operand)),
            },
        },
    }
}

pub(super) fn owner(crate_id: u64, dependencies: Option<&JavaDependencyApi>) -> JavaDependencyApi {
    let mut scope = JavaDependencyScope::new();
    let imports: Vec<_> = dependencies
        .into_iter()
        .flat_map(|owner| owner.functions())
        .collect();
    let mut declarations = Vec::new();
    for (index, mode) in Mode::ALL.into_iter().enumerate() {
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
            name: f::name(&format!("round{index}")),
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
                JavaStmt::Return(Some(round(mode, JavaExpr::local(double(), local)))),
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
fn primitive_and_imported_rounding_have_exact_certificates() {
    let first = owner(101, None);
    let second = owner(102, Some(&first));
    for api in [&first, &second] {
        assert_eq!(api.functions().count(), 3);
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
            assert!(text.contains("Math.floor(") && text.contains("Math.ceil("));
            assert!(!text.contains("Runtime") && !text.contains("longBitsToDouble"));
        }
    }
}
#[path = "binary64_rounding_bounds.rs"]
mod bounds;
#[path = "binary64_rounding_contracts.rs"]
mod contracts;
#[path = "binary64_rounding_native.rs"]
mod native;
