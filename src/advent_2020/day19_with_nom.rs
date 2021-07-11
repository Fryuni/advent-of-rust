use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::str::FromStr;

use nom::Finish;
use nom::Parser;

use crate::advent_adapters::AdventState;
use crate::helper;
use crate::helper::nom::VerboseError;
use itertools::Itertools;

type ParsingError<'a> = helper::nom::VerboseError<&'a str>;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Rule {
    Lit(String),
    Ref(usize),
    Sequence(Vec<Rule>),
    Alternative(Vec<Rule>),
}

impl Rule {
    fn parse(input: &str) -> nom::IResult<&str, Self, ParsingError> {
        nom::branch::alt((
            Self::parse_alternative,
            Self::parse_sequence,
            Self::parse_lit,
            Self::parse_ref,
        ))(input)
    }

    fn parse_lit(input: &str) -> nom::IResult<&str, Self, ParsingError> {
        nom::combinator::map(
            nom::sequence::delimited(
                nom::character::complete::char('"'),
                nom::character::complete::alpha1,
                nom::character::complete::char('"'),
            ),
            |l: &str| Self::Lit(l.to_string()),
        )(input)
    }

    fn parse_ref(input: &str) -> nom::IResult<&str, Self, ParsingError> {
        nom::combinator::map(nom::character::complete::digit1, |x: &str| {
            x.parse().map(Self::Ref).unwrap()
        })(input)
    }

    fn parse_sequence(input: &str) -> nom::IResult<&str, Self, ParsingError> {
        nom::combinator::map(
            nom::multi::separated_list1(nom::character::complete::space1, Self::parse_ref),
            Self::Sequence,
        )(input)
    }

    fn parse_alternative(input: &str) -> nom::IResult<&str, Self, ParsingError> {
        nom::combinator::map(
            nom::multi::separated_list1(nom::bytes::complete::tag(" | "), Self::parse_sequence),
            |v| {
                if v.len() == 1 {
                    v.into_iter().next().unwrap()
                } else {
                    Self::Alternative(v)
                }
            },
        )(input)
    }
}

impl<'a> TryFrom<&'a str> for Rule {
    type Error = ParsingError<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::parse(value).map(|(_, r)| r).map_err(|err| match err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
        })
    }
}

#[derive(Debug)]
struct RuleSet {
    rules: BTreeMap<usize, Rule>,
}

impl RuleSet {
    fn parse(input: &str) -> Result<(&str, Self), ParsingError> {
        nom::combinator::map(
            nom::sequence::terminated(
                nom::multi::separated_list1(
                    nom::character::complete::line_ending,
                    nom::sequence::separated_pair(
                        nom::combinator::map_res(nom::character::complete::digit1, usize::from_str),
                        nom::bytes::complete::tag(": "),
                        Rule::parse,
                    ),
                ),
                nom::bytes::complete::tag("\n\n"),
            ),
            |rules| Self {
                rules: rules.into_iter().collect(),
            },
        )(input)
        .finish()
    }

    fn parse_with_rule<'a>(&self, rule_idx: usize, input: &'a str) -> Result<(), RuleError<'a>> {
        let rule = self.rules.get(&rule_idx).ok_or(RuleError::RuleNotFound)?;

        nom::combinator::all_consuming(helper::nom::owned_context(
            format!("Rule::Ref({})", rule_idx),
            self.rule_to_parser(rule),
        ))
        .parse(input)
        .finish()
        .map(|(_, o)| o)
        .map_err(RuleError::ParsingError)
    }

    fn rule_to_parser<'a, 'c>(
        &'a self,
        rule: &'a Rule,
        // ) -> impl FnOnce(&'c str) -> nom::IResult<&'c str, (), ParsingError<'c>> + 'a
    ) -> impl Parser<&'c str, (), ParsingError<'c>> + 'a
    where
        'c: 'a,
    {
        move |input| match rule {
            Rule::Lit(lit) => helper::nom::owned_context(
                format!("rule {:?}", rule),
                nom::combinator::value((), nom::bytes::complete::tag(lit.as_str())),
            )(input),
            Rule::Ref(idx) => match self.rules.get(idx) {
                None => Err(nom::Err::Failure(VerboseError::add_owned_context(
                    input,
                    format!("could not find {:?}", rule),
                    Default::default(),
                ))),
                Some(rule) => helper::nom::owned_context(
                    format!("Rule::Ref({})", idx),
                    self.rule_to_parser(rule),
                )
                .parse(input),
            },
            Rule::Sequence(v) => nom::combinator::value(
                (),
                nom::sequence::tuple(
                    v.iter()
                        .enumerate()
                        .map(|(pos, rule)| {
                            helper::nom::owned_context(
                                format!("Rule::Sequence[{}]", pos),
                                self.rule_to_parser(rule),
                            )
                        })
                        .collect::<helper::nom::DynamicAlt<_>>(),
                ),
            )(input),
            Rule::Alternative(v) => nom::branch::alt(
                v.iter()
                    .enumerate()
                    .map(|(pos, rule)| {
                        helper::nom::owned_context(
                            format!("Rule::Alternative[{}]", pos),
                            self.rule_to_parser(rule),
                        )
                    })
                    .collect::<helper::nom::DynamicAlt<_>>(),
            )(input),
        }
    }

    fn merge_rules(&mut self, entries: impl IntoIterator<Item = (usize, Rule)>) {
        self.rules.extend(entries)
    }

    fn simplify(&mut self) {
        loop {
            let mut changes = Vec::new();

            for (&idx, rule) in &self.rules {
                if let Some(simplified) = self.simplify_rule(idx, rule) {
                    changes.push((idx, simplified))
                }
            }

            if changes.is_empty() {
                break;
            }

            for (idx, rule) in changes {
                self.rules.insert(idx, rule);
            }
        }
    }
    fn simplify_rule(&self, current_idx: usize, rule: &Rule) -> Option<Rule> {
        match rule {
            Rule::Ref(idx) if *idx != current_idx => self
                .rules
                .get(idx)
                .filter(|r| matches!(r, Rule::Lit(_)))
                .cloned(),
            Rule::Sequence(v) => {
                let simplifications: Vec<_> = v
                    .iter()
                    .map(|r| self.simplify_rule(current_idx, r))
                    .collect();

                if simplifications.iter().all(Option::is_none) {
                    return None;
                }

                let new_v: Vec<_> = simplifications
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| r.unwrap_or_else(|| v[i].clone()))
                    .collect();

                if new_v.iter().all(|r| matches!(r, Rule::Lit(_))) {
                    Some(Rule::Lit(
                        new_v
                            .into_iter()
                            .map(|r| match r {
                                Rule::Lit(s) => s,
                                _ => unreachable!(),
                            })
                            .collect(),
                    ))
                } else {
                    Some(Rule::Sequence(new_v))
                }
            }
            Rule::Alternative(v) => {
                let simplifications: Vec<_> = v
                    .iter()
                    .map(|r| self.simplify_rule(current_idx, r))
                    .collect();

                if simplifications.iter().all(Option::is_none) {
                    return None;
                }

                let new_v: Vec<_> = simplifications
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| r.unwrap_or_else(|| v[i].clone()))
                    .collect();

                if new_v.iter().all_equal() {
                    return Some(new_v.into_iter().next().unwrap());
                }

                Some(Rule::Alternative(new_v))
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
enum RuleError<'a> {
    RuleNotFound,
    ParsingError(ParsingError<'a>),
}

pub struct AdventDay19 {
    data: String,
}

impl AdventState for AdventDay19 {
    const INPUT_FILES: &'static [&'static str] = &[
        // "test1.txt",
        "test2.txt",
        // "input.txt",
    ];

    fn new(_: &'static str, input_content: String) -> Self {
        Self {
            data: input_content,
        }
    }

    fn run(self) {
        let (data, mut rules) = RuleSet::parse(&self.data).expect("could not parse input");

        let data: Vec<_> = data.split('\n').collect();

        let matching_step_1: Vec<_> = data
            .iter()
            .cloned()
            .filter(|line| rules.parse_with_rule(0, line).is_ok())
            .collect();

        println!("Matches for 1: {:#?}", matching_step_1);
        println!("Step 1: {}", matching_step_1.len());

        rules.merge_rules([
            (
                8,
                Rule::Alternative(vec![
                    Rule::Ref(42),
                    Rule::Sequence(vec![Rule::Ref(42), Rule::Ref(8)]),
                ]),
            ),
            (
                11,
                Rule::Alternative(vec![
                    Rule::Sequence(vec![Rule::Ref(42), Rule::Ref(31)]),
                    Rule::Sequence(vec![Rule::Ref(42), Rule::Ref(11), Rule::Ref(31)]),
                ]),
            ),
        ]);

        rules.rules.iter().for_each(|entry| println!("{:?}", entry));

        rules.simplify();
        rules.rules.iter().for_each(|entry| println!("{:?}", entry));

        let matching_step_2: Vec<_> = data
            .iter()
            .cloned()
            .enumerate()
            .filter(|(pos, line)| match rules.parse_with_rule(0, line) {
                Ok(_) => true,
                Err(RuleError::RuleNotFound) => {
                    eprintln!("Rule not found!");
                    false
                }
                Err(RuleError::ParsingError(err)) => {
                    eprintln!("{}: ({}) -> {}", pos, line, err);
                    false
                }
            })
            .collect();

        println!("Matches for 2: {:#?}", matching_step_2);
        println!("Step 2: {}", matching_step_2.len());
    }
}
