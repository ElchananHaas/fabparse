//Some code is based on Winnow by Elliot Page + other contributors.

mod tag;
mod branch;

use std::error::Error;

/**
 * Trait for a parser error. Input is the location of the input as a pointer.
 * parser_type is the type of parser.
 *
 * In order to simplify lifetimes used by the parser, the parser error
 * stores a pointer to the location the error occured rather than a reference
 * You can use the pointer to get the surrounding context in the original input
 * to the parser. This doesn't require unsafe, but if you drop the data
 * you are parsing before performing this lookup, you will not get relevant results.
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
     * the most progress, returning more helpful errors. 
     */
    fn get_loc(&self) -> Option<usize>;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParserType {
    Tag,
    Alt
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
        return Some(self.location)
    }
}

pub trait Parser<'a, I: ?Sized, O, ParserType> {
    fn parse<E: ParserError>(&mut self, input: &mut &'a I) -> Result<O, E>;
}

pub fn alt<T>(val: T) -> branch::Alt<T>{
    branch::Alt(val)
}
