use crate::priv_prelude::*;

#[derive(Clone)]
pub struct Map<P, F> {
    parser: P,
    func: F,
}

impl<T, U, P, F> Parser for Map<P, F>
where
    P: Parser<Output = T>,
    F: Fn(T) -> U,
    U: Spanned,
{
    type Output = U;

    fn parse(&self, input: &Span) -> Result<U, ParseError> {
        self.parser.parse(input).map(&self.func)
    }
}

#[derive(Clone)]
pub struct TryMap<P, F> {
    parser: P,
    func: F,
}

impl<P, T, F> Parser for TryMap<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<T, ParseError>,
    T: Spanned,
{
    type Output = T;

    fn parse(&self, input: &Span) -> Result<T, ParseError> {
        let value = self.parser.parse(input)?;
        (self.func)(value)
    }
}

#[derive(Clone)]
pub struct Then<P0, P1> {
    parser0: P0,
    parser1: P1,
}

impl<P0, P1> Parser for Then<P0, P1>
where
    P0: Parser,
    P1: Parser,
{
    type Output = (P0::Output, P1::Output);

    fn parse(&self, input: &Span) -> Result<(P0::Output, P1::Output), ParseError> {
        let value0 = self.parser0.parse(input)?;
        let input = input.with_range(value0.span().end()..);
        let value1 = self.parser1.parse(&input)?;
        Ok((value0, value1))
    }
}

#[derive(Clone)]
pub struct Or<P0, P1> {
    parser0: P0,
    parser1: P1,
}

impl<P0, P1> Parser for Or<P0, P1>
where
    P0: Parser,
    P1: Parser<Output = P0::Output>,
{
    type Output = P0::Output;

    fn parse(&self, input: &Span) -> Result<P0::Output, ParseError> {
        match self.parser0.parse(input) {
            Ok(value) => Ok(value),
            Err(error0) => match self.parser1.parse(input) {
                Ok(value) => Ok(value),
                Err(error1) => {
                    Err(ParseError::Or {
                        error0: Box::new(error0),
                        error1: Box::new(error1),
                    })
                },
            },
        }
    }
}

#[derive(Clone)]
pub struct Optional<P> {
    parser: P,
}

impl<P> Parser for Optional<P>
where
    P: Parser,
{
    type Output = Result<P::Output, Span>;

    fn parse(&self, input: &Span) -> Result<Result<P::Output, Span>, ParseError> {
        match self.parser.parse(input) {
            Ok(value) => Ok(Ok(value)),
            Err(error) => Ok(Err(error.span())),
        }
    }
}

#[derive(Clone)]
pub struct ThenWhitespace<P> {
    parser: P,
}

impl<P> Parser for ThenWhitespace<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> Result<P::Output, ParseError> {
        (&self.parser)
        .then(whitespace())
        .map(|(value, _)| value)
        .parse(input)
    }
}

#[derive(Clone)]
pub struct ThenOptionalWhitespace<P> {
    parser: P,
}

impl<P> Parser for ThenOptionalWhitespace<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> Result<P::Output, ParseError> {
        (&self.parser)
        .then(whitespace().optional())
        .map(|(value, _)| value)
        .parse(input)
    }
}

#[derive(Clone)]
pub struct Repeated<P> {
    parser: P,
}

impl<P> Parser for Repeated<P>
where
    P: Parser,
{
    type Output = WithSpan<Vec<P::Output>>;

    fn parse(&self, input: &Span) -> Result<WithSpan<Vec<P::Output>>, ParseError> {
        let mut values = Vec::new();
        let mut span = input.to_start();
        let mut remaining_input = input.clone();
        loop {
            match self.parser.parse(&remaining_input) {
                Ok(value) => {
                    remaining_input = remaining_input.with_range(value.span().end()..);
                    span = Span::join(span, value.span());
                    values.push(value);
                },
                Err(..) => break,
            }
        }
        Ok(WithSpan { parsed: values, span })
    }
}


