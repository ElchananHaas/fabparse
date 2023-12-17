//Some code is based on Winnow by Elliot Page + other contributors.

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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParserType {
    Tag,
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
}

pub type PResult<O, E> = Result<O, E>;
pub trait Parser<'a, I: ?Sized, O, ParserType> {
    fn parse<E: ParserError>(&mut self, input: &mut &'a I) -> PResult<O, E>;
}

pub struct CharStrParser;
impl<'a> Parser<'a, str, char, CharStrParser> for char {
    fn parse<E: ParserError>(&mut self, input: &mut &'a str) -> PResult<char, E> {
        if (*input).starts_with(*self) {
            let (_, rest) = input.split_at(self.len_utf8());
            *input = rest;
            Ok(*self)
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct SliceSliceParser;

impl<'a, T: PartialEq, U> Parser<'a, [T], &'a [T], SliceSliceParser> for U
where
    U: AsRef<[T]>,
{
    fn parse<E: ParserError>(&mut self, input: &mut &'a [T]) -> Result<&'a [T], E> {
        let tag: &[T] = self.as_ref();
        if input.len() < tag.len() {
            Err(E::from_parser_error(*input, ParserType::Tag))
        } else {
            let (res, rest) = input.split_at(tag.len());
            if res == tag {
                *input = rest;
                Ok(res)
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        }
    }
}

pub struct ItemSliceParser;
impl<'a, T: PartialEq + Clone> Parser<'a, [T], T, ItemSliceParser> for T {
    fn parse<E: ParserError>(&mut self, input: &mut &'a [T]) -> Result<T, E> {
        if !input.is_empty() && input[0] == *self {
            let res = &input[0];
            *input = &input[1..];
            Ok(res.clone())
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct FnBoolSliceParser;
impl<'a, T: Clone + 'a, U: Fn(T) -> bool> Parser<'a, [T], T, FnBoolSliceParser> for U {
    fn parse<E: ParserError>(&mut self, input: &mut &'a [T]) -> Result<T, E> {
        if input.is_empty() {
            Err(E::from_parser_error(*input, ParserType::Tag))
        } else {
            if self(input[0].clone()) {
                let res = &input[0];
                *input = &input[1..];
                Ok(res.clone())
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        }
    }
}

pub struct FnBoolStrParser;
impl<'a, U: Fn(char) -> bool> Parser<'a, str, char, FnBoolStrParser> for U {
    fn parse<E: ParserError>(&mut self, input: &mut &'a str) -> Result<char, E> {
        let first_char = input.chars().next();
        if let Some(char) = first_char {
            if self(char) {
                *input = &input[char.len_utf8()..];
                Ok(char)
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct FnOptionSliceParser;
impl<'a, T: Clone + 'a, V, U: Fn(T) -> Option<V>> Parser<'a, [T], V, FnOptionSliceParser> for U {
    fn parse<E: ParserError>(&mut self, input: &mut &'a [T]) -> Result<V, E> {
        if input.is_empty() {
            Err(E::from_parser_error(*input, ParserType::Tag))
        } else {
            if let Some(res) = self(input[0].clone()) {
                *input = &input[1..];
                Ok(res)
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        }
    }
}

pub struct FnOptionStrParser;
impl<'a, V, U: Fn(char) -> Option<V>> Parser<'a, str, V, FnOptionStrParser> for U {
    fn parse<E: ParserError>(&mut self, input: &mut &'a str) -> Result<V, E> {
        let first_char = input.chars().next();
        if let Some(char) = first_char {
            if let Some(res) = self(char) {
                *input = &input[char.len_utf8()..];
                Ok(res)
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct FnResultSliceParser;
impl<'a, T: Clone + 'a, V, FErr, U> Parser<'a, [T], V, FnResultSliceParser> for U
where
    U: Fn(T) -> Result<V, FErr>,
    FErr: Error + Send + Sync + 'static,
{
    fn parse<E: ParserError>(&mut self, input: &mut &'a [T]) -> Result<V, E> {
        if input.is_empty() {
            Err(E::from_parser_error(*input, ParserType::Tag))
        } else {
            match self(input[0].clone()) {
                Ok(res) => {
                    *input = &input[1..];
                    Ok(res)
                }
                Err(err) => Err(E::from_external_error(*input, ParserType::Tag, err)),
            }
        }
    }
}
