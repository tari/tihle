//! Parsing of debug expressions.
//!
//! The grammar is defined in `expr.pest`; an expression may contain references
//! to register or memory values, which when evaluated yields a 16-bit value.

use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Expression {
    Literal(u16),
    Register(RegisterName),

    Memory8(Box<Expression>),
    Memory16(Box<Expression>),
    Sum(Box<(Expression, Expression)>),
    Difference(Box<(Expression, Expression)>),
    Product(Box<(Expression, Expression)>),
    Quotient(Box<(Expression, Expression)>),
    Remainder(Box<(Expression, Expression)>),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RegisterName {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    PC,
    SP,
    I,
    R,
    AF,
    BC,
    DE,
    HL,
    IX,
    IY,
    AF_,
    BC_,
    DE_,
    HL_,
}

impl FromStr for RegisterName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const NAMES: &[(&'static str, RegisterName)] = &[
            ("AF", RegisterName::AF),
            ("BC", RegisterName::BC),
            ("DE", RegisterName::DE),
            ("HL", RegisterName::HL),
            ("IX", RegisterName::IX),
            ("IY", RegisterName::IY),
            ("AF'", RegisterName::AF_),
            ("BC'", RegisterName::BC_),
            ("DE'", RegisterName::DE_),
            ("HL'", RegisterName::HL_),
            ("A", RegisterName::A),
            ("F", RegisterName::F),
            ("B", RegisterName::B),
            ("C", RegisterName::C),
            ("D", RegisterName::D),
            ("E", RegisterName::E),
            ("H", RegisterName::H),
            ("L", RegisterName::L),
            ("PC", RegisterName::PC),
            ("SP", RegisterName::SP),
            ("I", RegisterName::I),
            ("R", RegisterName::R),
        ];

        NAMES
            .iter()
            .find(|(n, _)| s.eq_ignore_ascii_case(n))
            .map(|(_, x)| *x)
            .ok_or(())
    }
}

/// The operations required to [evaluate](ExpressionParser::evaluate) expressions.
pub trait CpuContext {
    fn get_register(&mut self, r: RegisterName) -> u16;
    fn read_memory(&mut self, addr: u16, size: u16) -> Vec<u8>;
}

#[derive(Parser)]
#[grammar = "bin/debug/expr.pest"]
pub struct ExpressionParser(PrecClimber<Rule>);

impl ExpressionParser {
    pub fn new() -> Self {
        let climber = PrecClimber::new(vec![
            Operator::new(Rule::sum, Assoc::Left) | Operator::new(Rule::difference, Assoc::Left),
        ]);

        ExpressionParser(climber)
    }

    /// Evaluate the numeric value of an expression provided as a string.
    ///
    /// In case of error, the returned string describes why evaluation failed.
    pub fn evaluate<Ctx: CpuContext>(&self, ctx: &mut Ctx, s: &str) -> Result<u16, String> {
        let ast = self.parse_ast(s)?;
        eprintln!("{:?}", ast);
        Ok(self.evaluate_(ctx, ast))
    }

    fn evaluate_binop<Ctx: CpuContext, F: FnOnce(u16, u16) -> u16>(
        &self,
        ctx: &mut Ctx,
        operation: F,
        pair: Box<(Expression, Expression)>,
    ) -> u16 {
        let (lhs, rhs) = *pair;
        operation(self.evaluate_(ctx, lhs), self.evaluate_(ctx, rhs))
    }

    fn evaluate_<Ctx: CpuContext>(&self, ctx: &mut Ctx, expr: Expression) -> u16 {
        match expr {
            Expression::Literal(x) => x,
            Expression::Sum(pair) => self.evaluate_binop(ctx, u16::wrapping_add, pair),
            Expression::Difference(pair) => self.evaluate_binop(ctx, u16::wrapping_sub, pair),
            Expression::Product(pair) => self.evaluate_binop(ctx, u16::wrapping_mul, pair),
            Expression::Quotient(pair) => self.evaluate_binop(ctx, u16::wrapping_div, pair),
            Expression::Remainder(pair) => self.evaluate_binop(ctx, u16::wrapping_rem, pair),
            Expression::Register(r) => ctx.get_register(r),
            Expression::Memory8(addr) => {
                let addr = self.evaluate_(ctx, *addr);
                let data = ctx.read_memory(addr, 1);
                data[0] as u16
            }
            Expression::Memory16(addr) => {
                let addr = self.evaluate_(ctx, *addr);
                let data = ctx.read_memory(addr, 2);
                data[0] as u16 | ((data[1] as u16) << 8)
            }
        }
    }

    fn parse_ast(&self, s: &str) -> Result<Expression, String> {
        let expr = match Self::parse(Rule::top, s) {
            Ok(x) => x,
            Err(e) => return Err(format!("{}", e)),
        };

        self.parse_tokens(expr)
    }

    fn parse_tokens(&self, t: Pairs<Rule>) -> Result<Expression, String> {
        // Handle standalone tokens
        let parse_primary = |pair: Pair<Rule>| -> Result<Expression, String> {
            match pair.as_rule() {
                Rule::literal => match parse_literal(pair.as_str()) {
                    None => Err(format!("Literal value \"{}\" too large", pair.as_str())),
                    Some(x) => Ok(Expression::Literal(x)),
                },
                Rule::register => Ok(Expression::Register(
                    RegisterName::from_str(pair.as_str()).unwrap(),
                )),
                Rule::mem8 => {
                    let contained = self.parse_tokens(pair.into_inner());
                    contained.map(|e| Expression::Memory8(Box::new(e)))
                }
                Rule::mem16 => {
                    let contained = self.parse_tokens(pair.into_inner());
                    contained.map(|e| Expression::Memory16(Box::new(e)))
                }
                x => unreachable!("Rule {:?} should not be reachable", x),
            }
        };

        // Apply an infix operator to two arguments
        fn parse_infix(
            lhs: Result<Expression, String>,
            op: Pair<Rule>,
            rhs: Result<Expression, String>,
        ) -> Result<Expression, String> {
            let (lhs, rhs) = match (lhs, rhs) {
                (Err(e), _) | (Ok(_), Err(e)) => return Err(e),
                (Ok(l), Ok(r)) => (l, r),
            };

            eprintln!("Parse infix {:?}", op);
            Ok(match op.as_rule() {
                Rule::sum => Expression::Sum(Box::new((lhs, rhs))),
                Rule::difference => Expression::Difference(Box::new((lhs, rhs))),
                Rule::product => Expression::Product(Box::new((lhs, rhs))),
                Rule::quotient => Expression::Quotient(Box::new((lhs, rhs))),
                Rule::remainder => Expression::Remainder(Box::new((lhs, rhs))),
                _ => unreachable!(),
            })
        }

        self.0.climb(t, parse_primary, parse_infix)
    }
}

fn parse_literal(s: &str) -> Option<u16> {
    let (s, base) = if s.starts_with("$") {
        (&s[1..], 16)
    } else if s.starts_with("0x") || s.starts_with("0X") {
        (&s[2..], 16)
    } else if s.ends_with("h") {
        (&s[..s.len() - 1], 16)
    } else if s.starts_with("0b") || s.starts_with("0B") {
        (&s[2..], 2)
    } else if s.starts_with("%") {
        (&s[1..], 2)
    } else {
        (&s[..], 10)
    };

    match u16::from_str_radix(s, base) {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Expression, ExpressionParser, RegisterName};

    fn test_parse(s: &str) -> Expression {
        let parser = ExpressionParser::new();
        parser.parse_ast(s).expect("Failed to parse")
    }

    fn test_parse_error(s: &str) -> String {
        let parser = ExpressionParser::new();
        parser.parse_ast(s).expect_err("Didn't fail to parse")
    }

    #[test]
    fn parse_register_sum() {
        assert_eq!(
            test_parse("HL+de"),
            Expression::Sum(Box::new((
                Expression::Register(RegisterName::HL),
                Expression::Register(RegisterName::DE)
            )))
        );
    }

    #[test]
    fn parse_decimal_literal() {
        assert_eq!(test_parse("1234"), Expression::Literal(1234))
    }

    #[test]
    fn parse_hex_literal() {
        assert_eq!(test_parse("0x1234"), Expression::Literal(0x1234));
        assert_eq!(test_parse("$FEDC"), Expression::Literal(0xFEDC));
        assert_eq!(test_parse("C0deh"), Expression::Literal(0xC0DE));
        assert_eq!(
            test_parse_error("0x12345"),
            "Literal value \"0x12345\" too large".to_string()
        );
    }

    #[test]
    fn parse_binary_literal() {
        assert_eq!(test_parse("0b1011001"), Expression::Literal(0b1011001));
        assert_eq!(test_parse("%11110000"), Expression::Literal(0b11110000));
        assert_eq!(
            test_parse_error("0b10000000000000000"),
            "Literal value \"0b10000000000000000\" too large".to_string()
        );
    }

    #[test]
    fn parse_memory_ref() {
        assert_eq!(
            test_parse("( de + 2 )"),
            Expression::Memory8(Box::new(Expression::Sum(Box::new((
                Expression::Register(RegisterName::DE),
                Expression::Literal(2)
            )))))
        );

        assert_eq!(
            test_parse("w(3)"),
            Expression::Memory16(Box::new(Expression::Literal(3)))
        );
    }
}
