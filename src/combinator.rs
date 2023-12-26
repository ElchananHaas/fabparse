use std::{error::Error, marker::PhantomData};

use crate::{Parser, ParserError, ParserType};

pub struct ParserMap<P, I: ?Sized, M, O> {
    pub parser: P,
    pub func: fn(M) -> O,
    pub phantom_i: PhantomData<I>,
}

pub struct ParserMapT<PType, M> {
    phantom_ptype: PhantomData<PType>,
    phantom_m: PhantomData<M>,
}
impl<'a, P, M, O, PType, I: ?Sized> Parser<'a, I, O, ParserMapT<PType, M>> for ParserMap<P, I, M, O>
where
    P: Parser<'a, I, M, PType>,
{
    fn parse<E: ParserError>(&self, input: &mut &'a I) -> Result<O, E> {
        match self.parser.parse::<E>(input) {
            Ok(res) => Ok((self.func)(res)),
            Err(err) => Err(err),
        }
    }
}

pub struct Try<P> {
    pub parser: P,
}

pub struct TryParser<T> {
    t: PhantomData<T>,
}

impl<'a, I: ?Sized, O, P, PType> Parser<'a, I, O, TryParser<PType>> for Try<P>
where
    P: Parser<'a, I, Option<O>, PType>,
{
    fn parse<E: ParserError>(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        let parsed = self.parser.parse(input)?;
        parsed.ok_or_else(|| {
            *input = checkpoint;
            E::from_parser_error(*input, ParserType::Try)
        })
    }
}

pub struct TryResultParser<T, Err> {
    t: PhantomData<T>,
    err: PhantomData<Err>,
}
impl<'a, I: ?Sized, O, P, PType, Err> Parser<'a, I, O, TryResultParser<PType, Err>> for Try<P>
where
    P: Parser<'a, I, Result<O, Err>, PType>,
    Err: Error + Send + Sync + 'static,
{
    fn parse<E: ParserError>(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        let parsed = self.parser.parse(input)?;
        parsed.map_err(|err| {
            *input = checkpoint;
            E::from_external_error(*input, ParserType::Try, err)
        })
    }
}

pub struct ParserTryMap<P, I: ?Sized, M, O> {
    pub parser: P,
    pub func: fn(M) -> O,
    pub phantom_i: PhantomData<I>,
}

pub struct ParserMapOptionT<PType, M> {
    phantom_ptype: PhantomData<PType>,
    phantom_m: PhantomData<M>,
}
impl<'a, P, M, O, PType, I: ?Sized> Parser<'a, I, O, ParserMapOptionT<PType, M>>
    for ParserTryMap<P, I, M, Option<O>>
where
    P: Parser<'a, I, M, PType>,
{
    fn parse<E: ParserError>(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        match self.parser.parse::<E>(input) {
            Ok(res) => {
                let func_result = (self.func)(res);
                func_result.ok_or_else(|| {
                    *input = checkpoint;
                    E::from_parser_error(*input, ParserType::TryMap)
                })
            }
            Err(err) => Err(err),
        }
    }
}


impl<'a, P, M, O, PType, I: ?Sized, Err> Parser<'a, I, O, ParserMapOptionT<PType, M>>
    for ParserTryMap<P, I, M, Result<O, Err>>
where
    P: Parser<'a, I, M, PType>,
    Err: Error + Send + Sync + 'static,
{
    fn parse<E: ParserError>(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        match self.parser.parse::<E>(input) {
            Ok(res) => {
                let func_result = (self.func)(res);
                func_result.map_err(|err| {
                    *input = checkpoint;
                    E::from_external_error(*input, ParserType::TryMap, err)
                })
            }
            Err(err) => Err(err),
        }
    }
}
