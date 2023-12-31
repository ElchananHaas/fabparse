//Some code is based on Winnow by Elliot Page + other contributors.

mod branch;
mod combinator;
mod sequence;
mod tag;

use std::{error::Error, fmt::Debug, marker::PhantomData};

use combinator::{ParserMap, Try, ParserTryMap, Opt};

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
    CustomFn
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

pub trait Parser<'a, I: ?Sized, O, E: ParserError, ParserType> {
    /**
     * Parses the input. This method advances the input reference to any remaining
     * unparsed input.
     */
    fn fab(&self, input: &mut &'a I) -> Result<O, E>;
    /**
     * This creates a Try parser that if the underlying parser returns Result::Ok or Option::Some,
     * unwraps the value. If it returns None or Err, then the Try parser will fail. 
     * 
     * Only use this method on parsers that return an Option or Result. 
     * Otherwise, the returned parser won't implemented the Parser trait.
     */
    fn fab_try(self) -> Try<Self>
    where
        Self: Sized,
    {
        Try { parser: self }
    }
    /**
     * This creates a Map parser that applies the function to the 
     * output of the underlying parser.
     */
    fn fab_map<FOut>(self, func: fn(O) -> FOut) -> ParserMap<Self, I, O, FOut, E>
    where
        Self: Sized,
    {
        ParserMap {
            parser: self,
            func,
            phantom_i: PhantomData,
            phantom_e: PhantomData
        }
    }
    /**
     * This parser composes the behavior of the Try and Map parsers. It 
     * first maps the input, and if the result is Option::Some or Result::Ok, 
     * it unwraps the input. Othewise, the parser fails. 
     * 
     */
    fn fab_try_map<FOut>(self, func: fn(O) -> FOut) -> ParserTryMap<Self, I, O, FOut, E>
    where Self: Sized {
        ParserTryMap {
            parser: self,
            func,
            phantom_i: PhantomData,
            phantom_e: PhantomData
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
 * This function takes in a tuple of 1 to 11 parsers. It returns a parser that
 * succeeds when all of the input parsers have succeeded in any order.
 * Returns a tuple of outputs from the parsers in the same order
 * that they were provided in the input.
 *
 * The parsers will be tried in the order that they are provided in the input.
 */
pub fn permutation<T>(parsers: T) -> branch::Permutation<T> {
    branch::Permutation(parsers)
}

/**
 * Constructs a parser that takes that many items. For strings, this
 * will be characters and for arrays it will be elements.
 */
pub fn take(count: usize) -> tag::Take {
    tag::Take(count)
}

/**
 * Makes the underlying parser optional. If the underlying parser succeeds with result out, 
 * this parser returns Some(out). Otherwise, this parser succeeds with None and 
 * consumes no input.
 */
pub fn opt<T>(parser: T) -> combinator::Opt<T> {
    Opt {
        parser
    }
}