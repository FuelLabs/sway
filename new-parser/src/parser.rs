use crate::priv_prelude::*;

pub trait Parser {
    type Output: Spanned;
    fn parse(&self, input: &Span) -> Result<Self::Output, ParseError>;
}

impl<P: ?Sized> Parser for Box<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> Result<P::Output, ParseError> {
        (&**self).parse(input)
    }
}

impl<P: ?Sized> Parser for Rc<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> Result<P::Output, ParseError> {
        (&**self).parse(input)
    }
}

impl<'a, P: ?Sized> Parser for &'a P
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> Result<P::Output, ParseError> {
        (&**self).parse(input)
    }
}

pub trait ParserExt: Parser {
    fn map<F>(self, func: F) -> Map<Self, F>
    where
        Self: Sized;

    fn try_map<F>(self, func: F) -> TryMap<Self, F>
    where
        Self: Sized;

    fn then<R>(self, parser: R) -> Then<Self, R>
    where
        Self: Sized;

    fn or<R>(self, parser: R) -> Or<Self, R>
    where
        Self: Sized;

    fn optional(self) -> Optional<Self>
    where
        Self: Sized;

    fn then_whitespace(self) -> ThenWhitespace<Self>
    where
        Self: Sized;

    fn then_optional_whitespace(self) -> ThenOptionalWhitespace<Self>
    where
        Self: Sized;

    fn repeated(self) -> Repeated<Self>
    where
        Self: Sized;
}

impl<P> ParserExt for P
where
    P: Parser,
{
    fn map<F>(self, func: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map {
            parser: self,
            func,
        }
    }

    fn try_map<F>(self, func: F) -> TryMap<Self, F>
    where
        Self: Sized,
    {
        TryMap {
            parser: self,
            func,
        }
    }

    fn then<R>(self, parser: R) -> Then<Self, R>
    where
        Self: Sized,
    {
        Then {
            parser0: self,
            parser1: parser,
        }
    }

    fn or<R>(self, parser: R) -> Or<Self, R>
    where
        Self: Sized,
    {
        Or {
            parser0: self,
            parser1: parser,
        }
    }

    fn optional(self) -> Optional<Self>
    where
        Self: Sized,
    {
        Optional {
            parser: self,
        }
    }

    fn then_whitespace(self) -> ThenWhitespace<Self>
    where
        Self: Sized,
    {
        ThenWhitespace {
            parser: self,
        }
    }

    fn then_optional_whitespace(self) -> ThenOptionalWhitespace<Self>
    where
        Self: Sized,
    {
        ThenOptionalWhitespace {
            parser: self,
        }
    }

    fn repeated(self) -> Repeated<Self>
    where
        Self: Sized,
    {
        Repeated {
            parser: self,
        }
    }
}

