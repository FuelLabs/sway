use crate::priv_prelude::*;

pub trait Parser {
    type Output;
    type Error;
    type FatalError;
    fn parse(&self, input: &Span) -> Result<(Self::Output, usize), Result<Self::Error, Self::FatalError>>;
}

impl<P: ?Sized> Parser for Box<P>
where
    P: Parser,
{
    type Output = P::Output;
    type Error = P::Error;
    type FatalError = P::FatalError;

    fn parse(&self, input: &Span) -> Result<(P::Output, usize), Result<P::Error, P::FatalError>> {
        (&**self).parse(input)
    }
}

impl<P: ?Sized> Parser for Rc<P>
where
    P: Parser,
{
    type Output = P::Output;
    type Error = P::Error;
    type FatalError = P::FatalError;

    fn parse(&self, input: &Span) -> Result<(P::Output, usize), Result<P::Error, P::FatalError>> {
        (&**self).parse(input)
    }
}

impl<'a, P: ?Sized> Parser for &'a P
where
    P: Parser,
{
    type Output = P::Output;
    type Error = P::Error;
    type FatalError = P::FatalError;

    fn parse(&self, input: &Span) -> Result<(P::Output, usize), Result<P::Error, P::FatalError>> {
        (&**self).parse(input)
    }
}

impl<P0, P1> Parser for Either<P0, P1>
where
    P0: Parser,
    P1: Parser<Output = P0::Output, Error = P0::Error, FatalError = P0::FatalError>,
{
    type Output = P0::Output;
    type Error = P0::Error;
    type FatalError = P0::FatalError;

    fn parse(&self, input: &Span) -> Result<(P0::Output, usize), Result<P0::Error, P0::FatalError>> {
        match self {
            Either::Left(parser0) => parser0.parse(input),
            Either::Right(parser1) => parser1.parse(input),
        }
    }
}

pub trait ParserExt: Parser {
    fn map<F, U>(self, func: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U;

    fn map_with_span<F, U>(self, func: F) -> MapWithSpan<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output, Span) -> U;

    fn try_map<F, T>(self, func: F) -> TryMap<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Result<T, Result<Self::Error, Self::FatalError>>;

    fn try_map_with_span<F, T>(self, func: F) -> TryMapWithSpan<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output, Span) -> Result<T, Result<Self::Error, Self::FatalError>>;

    fn then<R>(self, parser: R) -> Then<Self, R>
    where
        Self: Sized,
        R: Parser<Error = Self::Error, FatalError = Self::FatalError>;

    /*
    fn optional<E>(self) -> Optional<Self, E>
    where
        Self: Sized;

    fn then_optional_whitespace(self) -> ThenOptionalWhitespace<Self>
    where
        Self: Sized;

    /*
    fn or<R>(self, parser: R) -> Or<Self, R>
    where
        Self: Sized;
    */

    fn then_whitespace(self) -> ThenWhitespace<Self>
    where
        Self: Sized;

    fn repeated<E>(self) -> Repeated<Self, E>
    where
        Self: Sized;

    fn while_some(self) -> WhileSome<Self>
    where
        Self: Sized;
    */

    fn and_then<F, P1>(self, func: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> P1,
        P1: Parser<Error = Self::Error, FatalError = Self::FatalError>;

    /*
    fn debug(self, text: &'static str) -> Debug<Self>
    where
        Self: Sized;

    fn commit(self) -> Commit<Self>
    where
        Self: Sized;

    fn uncommit(self) -> Uncommit<Self>
    where
        Self: Sized;
    */

    fn map_err<F, E>(self, func: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> E;

    fn map_fatal_err<F, R>(self, func: F) -> MapFatalErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::FatalError) -> R;

    fn map_err_with_span<F, E>(self, func: F) -> MapErrWithSpan<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error, Span) -> E;


    fn map_fatal_err_with_span<F, R>(self, func: F) -> MapFatalErrWithSpan<Self, F>
    where
        Self: Sized,
        F: Fn(Self::FatalError, Span) -> R;

    fn or_else<F, E>(self, func: F) -> OrElse<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error, &Span) -> Result<(Self::Output, usize), Result<E, Self::FatalError>>;

    /*
    fn fatal<E>(self) -> Fatal<Self, E>
    where
        Self: Sized,
        //Self::FatalError: Into<Self::Error>;
        Self: Parser<FatalError = Self::Error>;
    */
}

impl<P> ParserExt for P
where
    P: Parser,
{
    fn map<F, U>(self, func: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        Map::new(self, func)
    }

    fn map_with_span<F, U>(self, func: F) -> MapWithSpan<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output, Span) -> U,
    {
        MapWithSpan::new(self, func)
    }

    fn try_map<F, T>(self, func: F) -> TryMap<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Result<T, Result<Self::Error, Self::FatalError>>,
    {
        TryMap::new(self, func)
    }

    fn try_map_with_span<F, T>(self, func: F) -> TryMapWithSpan<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output, Span) -> Result<T, Result<Self::Error, Self::FatalError>>,
    {
        TryMapWithSpan::new(self, func)
    }

    fn then<R>(self, parser: R) -> Then<Self, R>
    where
        Self: Sized,
        R: Parser<Error = Self::Error, FatalError = Self::FatalError>,
    {
        Then::new(self, parser)
    }

    /*
    fn optional<E>(self) -> Optional<Self, E>
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

    /*
    fn or<R>(self, parser: R) -> Or<Self, R>
    where
        Self: Sized,
    {
        Or::new(self, parser)
    }
    */

    fn then_whitespace(self) -> ThenWhitespace<Self>
    where
        Self: Sized,
    {
        ThenWhitespace::new(self)
    }

    fn repeated<E>(self) -> Repeated<Self, E>
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
    */

    fn and_then<F, P1>(self, func: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> P1,
        P1: Parser<Error = Self::Error, FatalError = Self::FatalError>,
    {
        AndThen::new(self, func)
    }

    /*
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
    */

    fn map_err<F, E>(self, func: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> E,
    {
        MapErr::new(self, func)
    }

    fn map_fatal_err<F, R>(self, func: F) -> MapFatalErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::FatalError) -> R,
    {
        MapFatalErr::new(self, func)
    }

    fn map_err_with_span<F, E>(self, func: F) -> MapErrWithSpan<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error, Span) -> E,
    {
        MapErrWithSpan::new(self, func)
    }

    fn map_fatal_err_with_span<F, R>(self, func: F) -> MapFatalErrWithSpan<Self, F>
    where
        Self: Sized,
        F: Fn(Self::FatalError, Span) -> R,
    {
        MapFatalErrWithSpan::new(self, func)
    }

    fn or_else<F, E>(self, func: F) -> OrElse<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error, &Span) -> Result<(Self::Output, usize), Result<E, Self::FatalError>>,
    {
        OrElse::new(self, func)
    }

    /*
    fn fatal<E>(self) -> Fatal<Self, E>
    where
        Self: Sized,
        Self: Parser<FatalError = Self::Error>,
        //Self::FatalError: Into<Self::Error>,
    {
        Fatal::new(self)
    }
    */
}

pub trait ParserFatalExt: Parser {
    fn fatal<E>(self) -> Fatal<Self, E>
    where
        Self: Sized;
}

impl<P, R> ParserFatalExt for P
where
    P: Parser<Error = R, FatalError = R>,
{
    fn fatal<E>(self) -> Fatal<Self, E>
    where
        Self: Sized,
    {
        Fatal::new(self)
    }
}

pub trait ParserRecoverExt: Parser {
    fn optional<E>(self) -> Optional<Self, E>
    where
        Self: Sized;

    fn repeated<E>(self) -> Repeated<Self, E>
    where
        Self: Sized;
}

impl<P> ParserRecoverExt for P
where
    P: Parser<Error = ()>,
{
    fn optional<E>(self) -> Optional<Self, E>
    where
        Self: Sized,
    {
        Optional::new(self)
    }

    fn repeated<E>(self) -> Repeated<Self, E>
    where
        Self: Sized,
    {
        Repeated::new(self)
    }
}

