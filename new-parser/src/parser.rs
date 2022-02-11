use crate::priv_prelude::*;

pub trait Parser {
    type Output;
    fn parse(&self, input: &Span) -> (bool, Result<(Self::Output, usize), ParseError>);
}

impl<P: ?Sized> Parser for Box<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), ParseError>) {
        (&**self).parse(input)
    }
}

impl<P: ?Sized> Parser for Rc<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), ParseError>) {
        (&**self).parse(input)
    }
}

impl<'a, P: ?Sized> Parser for &'a P
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), ParseError>) {
        (&**self).parse(input)
    }
}

impl<P0, P1> Parser for Either<P0, P1>
where
    P0: Parser,
    P1: Parser<Output = P0::Output>,
{
    type Output = P0::Output;

    fn parse(&self, input: &Span) -> (bool, Result<(P0::Output, usize), ParseError>) {
        match self {
            Either::Left(parser0) => parser0.parse(input),
            Either::Right(parser1) => parser1.parse(input),
        }
    }
}

pub trait ParserExt: Parser {
    fn map<F>(self, func: F) -> Map<Self, F>
    where
        Self: Sized;

    fn map_with_span<F>(self, func: F) -> MapWithSpan<Self, F>
    where
        Self: Sized;

    fn try_map<F>(self, func: F) -> TryMap<Self, F>
    where
        Self: Sized;

    fn try_map_with_span<F>(self, func: F) -> TryMapWithSpan<Self, F>
    where
        Self: Sized;

    fn then<R>(self, parser: R) -> Then<Self, R>
    where
        Self: Sized;

    fn optional(self) -> Optional<Self>
    where
        Self: Sized;

    fn then_optional_whitespace(self) -> ThenOptionalWhitespace<Self>
    where
        Self: Sized;

    fn or<R>(self, parser: R) -> Or<Self, R>
    where
        Self: Sized;

    fn then_whitespace(self) -> ThenWhitespace<Self>
    where
        Self: Sized;

    fn repeated(self) -> Repeated<Self>
    where
        Self: Sized;

    fn while_some(self) -> WhileSome<Self>
    where
        Self: Sized;

    fn and_then<F>(self, func: F) -> AndThen<Self, F>
    where
        Self: Sized;

    fn debug(self, text: &'static str) -> Debug<Self>
    where
        Self: Sized;

    fn commit(self) -> Commit<Self>
    where
        Self: Sized;

    fn uncommit(self) -> Uncommit<Self>
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
        Map::new(self, func)
    }

    fn map_with_span<F>(self, func: F) -> MapWithSpan<Self, F>
    where
        Self: Sized,
    {
        MapWithSpan::new(self, func)
    }

    fn try_map<F>(self, func: F) -> TryMap<Self, F>
    where
        Self: Sized,
    {
        TryMap::new(self, func)
    }

    fn try_map_with_span<F>(self, func: F) -> TryMapWithSpan<Self, F>
    where
        Self: Sized,
    {
        TryMapWithSpan::new(self, func)
    }

    fn then<R>(self, parser: R) -> Then<Self, R>
    where
        Self: Sized,
    {
        Then::new(self, parser)
    }

    fn optional(self) -> Optional<Self>
    where
        Self: Sized,
    {
        Optional::new(self)
    }

    fn then_optional_whitespace(self) -> ThenOptionalWhitespace<Self>
    where
        Self: Sized,
    {
        ThenOptionalWhitespace::new(self)
    }

    fn or<R>(self, parser: R) -> Or<Self, R>
    where
        Self: Sized,
    {
        Or::new(self, parser)
    }

    fn then_whitespace(self) -> ThenWhitespace<Self>
    where
        Self: Sized,
    {
        ThenWhitespace::new(self)
    }

    fn repeated(self) -> Repeated<Self>
    where
        Self: Sized,
    {
        Repeated::new(self)
    }

    fn while_some(self) -> WhileSome<Self>
    where
        Self: Sized,
    {
        WhileSome::new(self)
    }

    fn and_then<F>(self, func: F) -> AndThen<Self, F>
    where
        Self: Sized,
    {
        AndThen::new(self, func)
    }

    fn debug(self, text: &'static str) -> Debug<Self>
    where
        Self: Sized,
    {
        Debug::new(self, text)
    }

    fn commit(self) -> Commit<Self>
    where
        Self: Sized,
    {
        Commit::new(self)
    }

    fn uncommit(self) -> Uncommit<Self>
    where
        Self: Sized,
    {
        Uncommit::new(self)
    }
}

