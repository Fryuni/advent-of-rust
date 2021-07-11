use std::fmt;
use std::iter::FromIterator;

use nom::branch::Alt;
use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError};
use nom::sequence::Tuple;
use nom::{Err, IResult, Parser};

pub struct DynamicAlt<P>(Vec<P>);

impl<P> From<Vec<P>> for DynamicAlt<P> {
    fn from(v: Vec<P>) -> Self {
        Self(v)
    }
}

impl<P> FromIterator<P> for DynamicAlt<P> {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<I: Clone, O, E, P> Alt<I, O, E> for DynamicAlt<P>
where
    P: Parser<I, O, E>,
{
    fn choice(&mut self, input: I) -> IResult<I, O, E> {
        let length = self.0.len();

        for alt in &mut self.0[..length - 1] {
            if let Ok(o) = alt.parse(input.clone()) {
                return Ok(o);
            };
        }

        self.0
            .last_mut()
            .expect("DynamicAlt must include at least one alternative")
            .parse(input)
    }
}

impl<I, O, E, P> Tuple<I, Vec<O>, E> for DynamicAlt<P>
where
    P: Parser<I, O, E>,
{
    fn parse(&mut self, mut input: I) -> IResult<I, Vec<O>, E> {
        let mut results = Vec::with_capacity(self.0.len());

        for parser in &mut self.0 {
            let (next, res) = parser.parse(input)?;

            input = next;
            results.push(res);
        }

        Ok((input, results))
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VerboseError<I> {
    errors: Vec<(I, VerboseErrorKind)>,
}

#[derive(Clone, Debug, PartialEq)]
/// Error context for `VerboseError`
pub enum VerboseErrorKind {
    /// Static string added by the `context` function
    Context(&'static str),
    /// Dynamic string added by the `context` function
    OwnedContext(String),
    /// Indicates which character was expected by the `char` function
    Char(char),
    /// Error kind given by various nom parsers
    Nom(ErrorKind),
}

impl<I> ParseError<I> for VerboseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            errors: vec![(input, VerboseErrorKind::Nom(kind))],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, VerboseErrorKind::Nom(kind)));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        VerboseError {
            errors: vec![(input, VerboseErrorKind::Char(c))],
        }
    }
}

impl<I, E> FromExternalError<I, E> for VerboseError<I> {
    fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
        Self::from_error_kind(input, kind)
    }
}

impl<I: fmt::Display> fmt::Display for VerboseError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Parse error:")?;
        for (input, error) in &self.errors {
            match error {
                VerboseErrorKind::Nom(e) => writeln!(f, "{:?} at: {}", e, input)?,
                VerboseErrorKind::Char(c) => writeln!(f, "expected '{}' at: {}", c, input)?,
                VerboseErrorKind::Context(s) => writeln!(f, "in section '{}', at: {}", s, input)?,
                VerboseErrorKind::OwnedContext(s) => {
                    writeln!(f, "in section '{}', at: {}", s, input)?
                }
            }
        }

        Ok(())
    }
}

impl<I> ContextError<I> for VerboseError<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, VerboseErrorKind::Context(ctx)));
        other
    }
}

impl<I> VerboseError<I> {
    pub fn add_owned_context(input: I, ctx: String, mut other: Self) -> Self {
        other
            .errors
            .push((input, VerboseErrorKind::OwnedContext(ctx)));
        other
    }
}

pub fn owned_context<I: Clone, F, O>(
    context: String,
    mut f: F,
) -> impl FnMut(I) -> IResult<I, O, VerboseError<I>>
where
    F: Parser<I, O, VerboseError<I>>,
{
    move |i: I| match f.parse(i.clone()) {
        Ok(o) => Ok(o),
        Err(Err::Incomplete(i)) => Err(Err::Incomplete(i)),
        Err(Err::Error(e)) => Err(Err::Error(VerboseError::add_owned_context(
            i,
            context.clone(),
            e,
        ))),
        Err(Err::Failure(e)) => Err(Err::Failure(VerboseError::add_owned_context(
            i,
            context.clone(),
            e,
        ))),
    }
}
