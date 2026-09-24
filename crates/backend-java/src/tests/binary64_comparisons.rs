//! Double comparisons preserve numeric equality, including signed zero and NaN.
use super::*;

pub(super) fn comparisons() -> JavaDependencyApi {
    let operators = [
        JavaBinaryOperator::Equal,
        JavaBinaryOperator::NotEqual,
        JavaBinaryOperator::Less,
        JavaBinaryOperator::LessEqual,
        JavaBinaryOperator::Greater,
        JavaBinaryOperator::GreaterEqual,
    ];
    let declarations = operators
        .into_iter()
        .enumerate()
        .map(|(index, operator)| {
            let input = JavaExpr::local(double(), f::name("input"));
            let zero = JavaExpr::literal(
                double(),
                JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(0).unwrap()),
            );
            f::Function {
                hash: 10 + index as u64,
                public: true,
                name: f::name(&format!("compare{index}")),
                parameters: vec![JavaParameter {
                    ty: double(),
                    name: f::name("input"),
                    final_parameter: true,
                }],
                result: f::boolean(),
                body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                    ty: f::boolean(),
                    precedence: if index < 2 {
                        JavaPrecedence::Equality
                    } else {
                        JavaPrecedence::Relational
                    },
                    kind: JavaExprKind::Binary {
                        operator,
                        left: Box::new(input),
                        right: Box::new(zero),
                    },
                }))]),
            }
        })
        .collect();
    JavaDependencyApi::from_certificate(f::certify(f::package(93, declarations))).unwrap()
}

/// Integer category oracle; intentionally does not execute Rust float comparisons.
pub(super) fn cases() -> Vec<(u64, [bool; 6])> {
    let mut bits = values();
    bits.extend([
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_0001,
        0xfff8_0000_0000_0100,
        0x7ff0_0000_0000_0001,
        0xfff0_0000_0000_0001,
    ]);
    bits.into_iter()
        .map(|bits| {
            let magnitude = bits & 0x7fff_ffff_ffff_ffff;
            let nan = magnitude > 0x7ff0_0000_0000_0000;
            let equal = magnitude == 0;
            let less = bits >> 63 == 1 && !equal && !nan;
            let greater = bits >> 63 == 0 && !equal && !nan;
            (
                bits,
                [
                    equal,
                    !equal,
                    less,
                    less || equal,
                    greater,
                    greater || equal,
                ],
            )
        })
        .collect()
}

#[test]
fn comparison_signatures_remain_exact_and_boolean() {
    let owner = comparisons();
    assert_eq!(owner.functions().count(), 6);
    for function in owner.functions() {
        assert_eq!(function.declaration_signature().parameters, [double()]);
        assert_eq!(function.declaration_signature().result, f::boolean());
    }
}
