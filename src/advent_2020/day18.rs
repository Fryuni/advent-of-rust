use std::fmt::{self, Debug, Display, Formatter, Write};
use std::str::FromStr;

use crate::advent_adapters::AdventState;

type ParserResult<'a, O> = nom::IResult<&'a str, O, nom::error::VerboseError<&'a str>>;

#[derive(Debug, Copy, Clone)]
enum Operation {
    Add,
    Mul,
}

#[derive(Debug, Clone)]
enum Token {
    Lit(usize),
    Operation(Operation),
    Expr(Box<Expr>),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Lit(v) => f.write_fmt(format_args!("{}", v)),
            Token::Operation(Operation::Add) => f.write_char('+'),
            Token::Operation(Operation::Mul) => f.write_char('*'),
            Token::Expr(v) => f.write_fmt(format_args!("({})", v)),
        }
    }
}

#[derive(Debug, Clone)]
struct Expr {
    tokens: Vec<Token>,
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.tokens.iter();

        if let Some(head) = iter.next() {
            Display::fmt(head, f)?;
        }

        for token in iter {
            f.write_char(' ')?;
            Display::fmt(token, f)?;
        }

        Ok(())
    }
}

fn decimal(input: &str) -> ParserResult<usize> {
    nom::combinator::map_res(
        nom::combinator::recognize(nom::multi::many1(nom::sequence::terminated(
            nom::character::complete::one_of("0123456789"),
            nom::multi::many0(nom::character::complete::char('_')),
        ))),
        usize::from_str,
    )(input)
}

impl Expr {
    fn parse(input: &str) -> Self {
        nom::combinator::all_consuming(nom::error::context("root parser", Self::parse_expr))(input)
            .map(|(_, expr)| expr)
            .expect("could not parse expression")
    }

    fn parse_expr(input: &str) -> ParserResult<Self> {
        nom::combinator::map(
            nom::multi::many1(nom::branch::alt((
                Self::parse_operator,
                Self::parse_operand,
            ))),
            |tokens| Self { tokens },
        )(input)
    }

    fn parse_lit(input: &str) -> ParserResult<Token> {
        nom::error::context("parsing literal", nom::combinator::map(decimal, Token::Lit))(input)
    }

    fn parse_operand(input: &str) -> ParserResult<Token> {
        nom::sequence::delimited(
            nom::character::complete::space0,
            nom::branch::alt((
                Self::parse_lit,
                nom::error::context(
                    "expression operand",
                    nom::combinator::map(
                        nom::sequence::delimited(
                            nom::character::complete::char('('),
                            Self::parse_expr,
                            nom::character::complete::char(')'),
                        ),
                        |expr| Token::Expr(Box::new(expr)),
                    ),
                ),
            )),
            nom::character::complete::space0,
        )(input)
    }

    fn parse_operator(input: &str) -> ParserResult<Token> {
        nom::error::context(
            "parsing operator",
            nom::branch::alt((Self::parse_addition, Self::parse_multiplication)),
        )(input)
    }

    fn parse_addition(input: &str) -> ParserResult<Token> {
        nom::combinator::map(nom::character::complete::char('+'), |_| {
            Token::Operation(Operation::Add)
        })(input)
    }

    fn parse_multiplication(input: &str) -> ParserResult<Token> {
        nom::combinator::map(nom::character::complete::char('*'), |_| {
            Token::Operation(Operation::Mul)
        })(input)
    }
}

#[derive(Debug)]
pub struct AdventDay18 {
    content: Vec<Expr>,
}

impl AdventDay18 {
    fn parse(input: String) -> Self {
        Self {
            content: input.trim().split('\n').map(Expr::parse).collect(),
        }
    }

    fn step1(&self) -> usize {
        self.content
            .iter()
            .map(|expr| Self::reduce_expression(expr, Self::step1_evaluator))
            .sum()
    }

    fn step2(&self) -> usize {
        self.content
            .iter()
            .map(|expr| Self::reduce_expression(expr, Self::step2_evaluator))
            .sum()
    }

    fn step1_evaluator(tokens: &[Token]) -> usize {
        let mut value = match tokens.first() {
            Some(&Token::Lit(v)) => v,
            _ => unreachable!("the first token should always be a literal at this point"),
        };

        assert_eq!(tokens.len() % 2, 1, "there must be an odd number of tokens");

        for pair in (&tokens[1..]).chunks_exact(2) {
            match pair {
                [Token::Operation(Operation::Add), Token::Lit(v)] => {
                    value += *v;
                }
                [Token::Operation(Operation::Mul), Token::Lit(v)] => {
                    value *= *v;
                }
                _ => unreachable!(),
            }
        }

        value
    }

    fn step2_evaluator(tokens: &[Token]) -> usize {
        let mut tokens = tokens.to_vec();

        while let Some(pos) = tokens
            .iter()
            .position(|t| matches!(t, Token::Operation(Operation::Add)))
        {
            // Remove the operator
            tokens.remove(pos);

            // Remove the right operand that is now on its place
            let right = match tokens.remove(pos) {
                Token::Lit(v) => v,
                _ => unreachable!(),
            };

            let left_handle = &mut tokens[pos - 1];
            let left = match *left_handle {
                Token::Lit(v) => v,
                _ => unreachable!(),
            };

            *left_handle = Token::Lit(left + right);
        }

        // Only literals and multiplication tokens left, fallback to step 1
        Self::step1_evaluator(&tokens)
    }

    fn reduce_expression(expr: &Expr, f: fn(&[Token]) -> usize) -> usize {
        let reduced_expression: Vec<_> = expr
            .tokens
            .iter()
            .map(|token| match token {
                Token::Expr(inner) => Token::Lit(Self::reduce_expression(&inner, f)),
                &Token::Lit(v) => Token::Lit(v),
                &Token::Operation(op) => Token::Operation(op),
            })
            .collect();

        f(&reduced_expression)
    }
}

impl AdventState for AdventDay18 {
    const INPUT_FILES: &'static [&'static str] = &["test.txt", "input.txt"];

    fn new(_input_file: &'static str, input_content: String) -> Self {
        Self::parse(input_content)
    }

    fn run(self) {
        println!("Answer to step 1: {}", self.step1());
        println!("Answer to step 2: {}", self.step2());
    }
}

#[test]
fn test_parsing() {
    let input = "4 + (5 + 8)";

    let expr = Expr::parse(input);

    println!("result: {}", expr);
    println!("result: {:?}", expr);
}
