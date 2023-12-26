//Some code is based on Winnow by Elliot Page + other contributors.

mod branch;
mod combinator;
mod sequence;
mod tag;

use std::{error::Error, fmt::Debug, marker::PhantomData};

use combinator::{ParserMap, Try, ParserTryMap};

/**
 * Trait for a parser error. Input is the location of the input as a pointer.
 * parser_type is the type of parser.
 *
 * In order to simplify lifetimes used by the parser, the parser error
 * stores a pointer to the location the error occured rather than a reference
 * You can use the pointer to get the surrounding context in the original input
 * to the parser (no unsafe required).
 */
pub trait ParserError {
    fn from_parser_error<T: ?Sized>(input: *const T, parser_type: ParserType) -> Self;
    fn from_external_error<T: ?Sized, E: Error + Send + Sync + 'static>(
        input: *const T,
        parser_type: ParserType,
        cause: E,
    ) -> Self;
    /**
     * Get the location of the error. This is used in combinators to recognize the parser that made
     * the furthest progress.
     */
    fn get_loc(&self) -> Option<usize>;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParserType {
    Tag,
    Alt,
    Try,
    TryMap,
}
#[derive(Debug)]
pub struct ContextError {
    pub parser_type: ParserType,
    pub location: usize,
    pub cause: Option<Box<dyn Error>>,
}

impl ParserError for ContextError {
    fn from_parser_error<T: ?Sized>(input: *const T, parser_type: ParserType) -> Self {
        ContextError {
            parser_type,
            location: input as *const u8 as usize,
            cause: None,
        }
    }
    fn from_external_error<T: ?Sized, E: Error + Send + Sync + 'static>(
        input: *const T,
        parser_type: ParserType,
        cause: E,
    ) -> Self {
        ContextError {
            parser_type,
            location: input as *const u8 as usize,
            cause: Some(Box::new(cause)),
        }
    }
    fn get_loc(&self) -> Option<usize> {
        return Some(self.location);
    }
}

pub trait Parser<'a, I: ?Sized, O, ParserType> {
    fn parse<E: ParserError>(&self, input: &mut &'a I) -> Result<O, E>;

    fn parser_try(self) -> Try<Self>
    where
        Self: Sized,
    {
        Try { parser: self }
    }

    fn parser_map<FOut>(self, func: fn(O) -> FOut) -> ParserMap<Self, I, O, FOut>
    where
        Self: Sized,
    {
        ParserMap {
            parser: self,
            func,
            phantom_i: PhantomData,
        }
    }

    fn try_map<FOut>(self, func: fn(O) -> FOut) -> ParserTryMap<Self, I, O, FOut>
    where Self: Sized {
        ParserTryMap {
            parser: self,
            func,
            phantom_i: PhantomData
        }
    }
}

/**
 * This function takes in a tuple of 1 to 11 parsers, all with the same
 * output type. It returns a parser that succeeds when any of its
 * input parsers succeed, with the output of the parser that succeeded.
 *
 * If none of the parsers succeed, this function will return an error.
 * When using ContextError, the error returned will be the error of the parser that made the
 * furthest progress. This behavior can only be garunteed when correct location hints are provided
 * when constructing ContextErrors. When using a parser that doesn't provide error locations, or in the event
 * of ties, FunnelParse makes no garuntees as to which child parser's error will be returned.
 */
pub fn alt<T>(parsers: T) -> branch::Alt<T> {
    branch::Alt(parsers)
}

/**
 * This function takes in a tuple of 1 to 11 parsers. Returns a parser that
 * succeeds when all of the input parsers have succeeded in any order.
 * Returns a tuple of outputs from the parsers in the same order
 * that they were provided in the input.
 *
 * The parsers will be tried in the order that they are provided in the input.
 * Its runtime is O(N^2) in the number of parsers. For parsing a permutation
 * of more than 11 parsers, consider using a hashmap instead of this combinator.
 */
pub fn permutation<T>(parsers: T) -> branch::Permutation<T> {
    branch::Permutation(parsers)
}

/**
 * Cunstructs a parser that takes that many items. For strings, this
 * will be characters and for arrays it will be elements.
 */
pub fn take(count: usize) -> tag::Take {
    tag::Take(count)
}
