use std::{error::Error, marker::PhantomData};

use crate::{sequence::Sequence, Parser, ParserError, ParserType};

pub struct ParserMap<P, I: ?Sized, M, E, F> {
    pub parser: P,
    pub func: F,
    pub phantom_i: PhantomData<I>,
    pub phantom_e: PhantomData<E>,
    pub phantom_m: PhantomData<M>,
}
pub struct ParserMapT<PType, M> {
    phantom_ptype: PhantomData<PType>,
    phantom_m: PhantomData<M>,
}
impl<'a, P, M, I, O, E: ParserError, PType, F> Parser<'a, I, O, E, ParserMapT<PType, M>>
    for ParserMap<P, I, M, E, F>
where
    P: Parser<'a, I, M, E, PType>,
    F: Fn(M) -> O,
    I: ?Sized + Sequence,
{
    fn fab(&self, input: &mut &'a I) -> Result<O, E> {
        match self.parser.fab(input) {
            Ok(res) => Ok((self.func)(res)),
            Err(mut err) => {
                err.add_context(*input, ParserType::Map);
                Err(err)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParserTryMap<P, I: ?Sized, M, E, F> {
    pub parser: P,
    pub func: F,
    pub phantom_i: PhantomData<I>,
    pub phantom_e: PhantomData<E>,
    pub phantom_m: PhantomData<M>,
}

pub struct ParserTryMapOption<PType, M> {
    phantom_ptype: PhantomData<PType>,
    phantom_m: PhantomData<M>,
}
impl<'a, P, M, I: ?Sized + Sequence, O, E: ParserError, PType, F>
    Parser<'a, I, O, E, ParserTryMapOption<PType, M>> for ParserTryMap<P, I, M, E, F>
where
    P: Parser<'a, I, M, E, PType>,
    F: Fn(M) -> Option<O>,
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
            Err(mut err) => {
                err.add_context(checkpoint, ParserType::Map);
                Err(err)
            }
        }
    }
}
pub struct ParserTryMapResult<PType, M, FErr> {
    phantom_ptype: PhantomData<PType>,
    phantom_m: PhantomData<M>,
    phantom_ferr: PhantomData<FErr>,
}
impl<'a, P, M, I, O, E: ParserError, PType, FErr, F>
    Parser<'a, I, O, E, ParserTryMapResult<PType, M, FErr>> for ParserTryMap<P, I, M, E, F>
where
    P: Parser<'a, I, M, E, PType>,
    FErr: Error + Send + Sync + 'static,
    F: Fn(M) -> Result<O, FErr>,
    I: ?Sized + Sequence,
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
            Err(mut err) => {
                err.add_context(checkpoint, ParserType::Map);
                Err(err)
            }
        }
    }
}

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct TakeNot<P> {
    pub parser: P,
}

pub struct TakeNotParser<P, O> {
    pub parser: PhantomData<P>,
    pub out: PhantomData<O>,
}
impl<'a, I, O, E: ParserError, ParType, P, Item> Parser<'a, I, Item, E, TakeNotParser<ParType, O>>
    for TakeNot<P>
where
    P: Parser<'a, I, O, E, ParType>,
    I: ?Sized + Sequence<Item = Item>,
{
    fn fab(&self, input: &mut &'a I) -> Result<Item, E> {
        let checkpoint = *input;
        match self.parser.fab(input) {
            Ok(_) => {
                *input = checkpoint;
                Err(E::from_parser_error(*input, ParserType::TakeNot))
            }
            Err(_) => {
                *input = checkpoint;
                match input.try_split_front() {
                    Some((first, rest)) => {
                        *input = rest;
                        Ok(first)
                    }
                    None => Err(E::from_parser_error(*input, ParserType::TakeNot)),
                }
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct Value<P, V, I: ?Sized, O, E> {
    pub parser: P,
    pub value: V,
    pub phantom_i: PhantomData<I>,
    pub phantom_o: PhantomData<O>,
    pub phantom_e: PhantomData<E>,
}

pub struct ValueParser<P, O> {
    pub parser: PhantomData<P>,
    pub out: PhantomData<O>,
}
impl<'a, I, O, E: ParserError, ParType, P, V> Parser<'a, I, V, E, ValueParser<ParType, O>>
    for Value<P, V, I, O, E>
where
    P: Parser<'a, I, O, E, ParType>,
    I: ?Sized + Sequence,
    V: Clone,
{
    fn fab(&self, input: &mut &'a I) -> Result<V, E> {
        match self.parser.fab(input) {
            Ok(_) => Ok(self.value.clone()),
            Err(mut err) => {
                err.add_context(*input, ParserType::Map);
                Err(err)
            }
        }
    }
}
