use std::marker::PhantomData;

use crate::{Parser, ParserError, Sequence};

pub struct Map<P, F, FIn, FOut, I: ?Sized> {
    pub parser: P,
    pub func: F,
    pub pf : PhantomData<FIn>,
    pub po : PhantomData<FOut>,
    pub pi : PhantomData<I>
}

pub struct ParserMap<PType, M> {
    phantom_ptype: PhantomData<PType>,
    phantom_m: PhantomData<M>,
}

impl<'a, P, F, PType, S: ?Sized, O, M> Parser<'a, S, O, ParserMap<PType, M>> for Map<P, F, M, O, S> 
where 
    P: Parser<'a, S, M, PType>,
    S: Sequence {
    fn parse<E: ParserError>(&self, input: &mut &'a S) -> Result<O, E> {
        todo!()
    }
}
/* 
impl<'a, P, F, M, O, PType, I: ?Sized> Parser<'a, I, O, ParserMap<PType, M>> for Map<P, F>
where
    P: Parser<'a, I, M, PType>,
    F: Fn(M) -> O,
{
    fn parse<E: ParserError>(&self, input: &mut &'a I) -> Result<O, E> {
        match self.parser.parse::<E>(input) {
            Ok(res) => Ok((self.func)(res)),
            Err(err) => Err(err),
        }
    }
}*/

pub struct Try<P> {
    pub parser: P,
}

pub struct TryParser<T> {
    t: PhantomData<T>,
}
impl<'a, I, O, P, PType> Parser<'a, I, O, TryParser<PType>> for Try<P>
where
    P: Parser<'a, I, Option<O>, PType>,
{
    fn parse<E: ParserError>(&self, input: &mut &'a I) -> Result<O, E> {
        todo!()
    }
}
