use std::{error::Error, marker::PhantomData};

use crate::{Parser, ParserError, ParserType};

pub struct ParserMap<P, I: ?Sized, M, O, E> {
    pub parser: P,
    pub func: fn(M) -> O,
    pub phantom_i: PhantomData<I>,
    pub phantom_e: PhantomData<E>,
}

pub struct ParserMapT<PType, M> {
    phantom_ptype: PhantomData<PType>,
    phantom_m: PhantomData<M>,
}
impl<'a, P, M, I: ?Sized, O, E: ParserError, PType> Parser<'a, I, O, E, ParserMapT<PType, M>>
    for ParserMap<P, I, M, O, E>
where
    P: Parser<'a, I, M, E, PType>,
{
    fn fab(&self, input: &mut &'a I) -> Result<O, E> {
        match self.parser.fab(input) {
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

impl<'a, I: ?Sized, O, E: ParserError, P, PType> Parser<'a, I, O, E, TryParser<PType>> for Try<P>
where
    P: Parser<'a, I, Option<O>, E, PType>,
{
    fn fab(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        let parsed = self.parser.fab(input)?;
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
impl<'a, I: ?Sized, O, E: ParserError, P, PType, Err>
    Parser<'a, I, O, E, TryResultParser<PType, Err>> for Try<P>
where
    P: Parser<'a, I, Result<O, Err>, E, PType>,
    Err: Error + Send + Sync + 'static,
{
    fn fab(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        let parsed = self.parser.fab(input)?;
        parsed.map_err(|err| {
            *input = checkpoint;
            E::from_external_error(*input, ParserType::Try, err)
        })
    }
}

pub struct ParserTryMap<P, I: ?Sized, M, O, E> {
    pub parser: P,
    pub func: fn(M) -> O,
    pub phantom_i: PhantomData<I>,
    pub phantom_e: PhantomData<E>,
}

pub struct ParserMapOptionT<PType, M> {
    phantom_ptype: PhantomData<PType>,
    phantom_m: PhantomData<M>,
}
impl<'a, P, M, I: ?Sized, O, E: ParserError, PType> Parser<'a, I, O, E, ParserMapOptionT<PType, M>>
    for ParserTryMap<P, I, M, Option<O>, E>
where
    P: Parser<'a, I, M, E, PType>,
{
    fn fab(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        match self.parser.fab(input) {
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

impl<'a, P, M, I: ?Sized, O, E: ParserError, PType, Err>
    Parser<'a, I, O, E, ParserMapOptionT<PType, M>> for ParserTryMap<P, I, M, Result<O, Err>, E>
where
    P: Parser<'a, I, M, E, PType>,
    Err: Error + Send + Sync + 'static,
{
    fn fab(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        match self.parser.fab(input) {
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

pub struct Opt<P> {
    pub parser: P,
}

impl<'a, I: ?Sized, O, E: ParserError, ParserType, P> Parser<'a, I, Option<O>, E, Opt<ParserType>>
    for Opt<P>
where
    P: Parser<'a, I, O, E, ParserType>,
{
    fn fab(&self, input: &mut &'a I) -> Result<Option<O>, E> {
        match self.parser.fab(input) {
            Ok(out) => Ok(Some(out)),
            Err(_) => Ok(None),
        }
    }
}
