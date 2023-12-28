use std::{
    error::Error,
    ops::{Range, RangeBounds},
};

use crate::{Parser, ParserError, ParserType};

pub struct CharStrParser;
impl<'a, E: ParserError> Parser<'a, str, char, E, CharStrParser> for char {
    fn parse(&self, input: &mut &'a str) -> Result<char, E> {
        if (*input).starts_with(*self) {
            let (_, rest) = input.split_at(self.len_utf8());
            *input = rest;
            Ok(*self)
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct ItemSliceParser;
impl<'a, T: PartialEq + Clone, E: ParserError> Parser<'a, [T], T, E, ItemSliceParser> for T {
    fn parse(&self, input: &mut &'a [T]) -> Result<T, E> {
        if !input.is_empty() && input[0] == *self {
            let res = &input[0];
            *input = &input[1..];
            Ok(res.clone())
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct SliceSliceParser;
impl<'a, T: PartialEq, U, E: ParserError> Parser<'a, [T], &'a [T], E, SliceSliceParser> for U
where
    U: AsRef<[T]>,
{
    fn parse(&self, input: &mut &'a [T]) -> Result<&'a [T], E> {
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

pub struct FnBoolSliceParser;
impl<'a, T: Clone + 'a, U: Fn(T) -> bool, E: ParserError> Parser<'a, [T], T, E, FnBoolSliceParser> for U {
    fn parse(&self, input: &mut &'a [T]) -> Result<T, E> {
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
impl<'a, U: Fn(char) -> bool, E: ParserError> Parser<'a, str, char, E, FnBoolStrParser> for U {
    fn parse(&self, input: &mut &'a str) -> Result<char, E> {
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
impl<'a, T: Clone + 'a, V, U, E: ParserError> Parser<'a, [T], V, E, FnOptionSliceParser> for U 
    where U: Fn(T) -> Option<V> {
    fn parse(&self, input: &mut &'a [T]) -> Result<V, E> {
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
impl<'a, V, U, E: ParserError> Parser<'a, str, V, E, FnOptionStrParser> for U
    where U: Fn(char) -> Option<V> {
    fn parse(&self, input: &mut &'a str) -> Result<V, E> {
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
impl<'a, T: Clone + 'a, V, FErr, U, E: ParserError> Parser<'a, [T], V, E, FnResultSliceParser> for U
where
    U: Fn(T) -> Result<V, FErr>,
    FErr: Error + Send + Sync + 'static,
{
    fn parse(&self, input: &mut &'a [T]) -> Result<V, E> {
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

pub struct FnResultStrParser;
impl<'a, V, U, FErr, E: ParserError> Parser<'a, str, V, E, FnResultStrParser> for U
where
    U: Fn(char) -> Result<V, FErr>,
    FErr: Error + Send + Sync + 'static,
{
    fn parse(&self, input: &mut &'a str) -> Result<V, E> {
        let first_char = input.chars().next();
        if let Some(char) = first_char {
            match self(char) {
                Ok(res) => {
                    *input = &input[char.len_utf8()..];
                    Ok(res)
                }
                Err(err) => Err(E::from_external_error(*input, ParserType::Tag, err)),
            }
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct CharRangeStrParser;
impl<'a, E: ParserError> Parser<'a, str, char, E, CharRangeStrParser> for Range<char> {
    fn parse(&self, input: &mut &'a str) -> Result<char, E> {
        let first_char = input.chars().next();
        if let Some(char) = first_char {
            if self.contains(&char) {
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

pub struct RangeSliceParser;
impl<'a, T, U, E: ParserError> Parser<'a, [T], T, E, RangeSliceParser> for U
where
    T: PartialOrd + Clone,
    U: RangeBounds<T>,
{
    fn parse(&self, input: &mut &'a [T]) -> Result<T, E> {
        if input.is_empty() {
            Err(E::from_parser_error(*input, ParserType::Tag))
        } else {
            if self.contains(&input[0]) {
                let res = input[0].clone();
                *input = &input[1..];
                Ok(res)
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        }
    }
}

pub struct Take(pub usize);
impl<'a, E: ParserError> Parser<'a, str, &'a str, E, Take> for Take {
    fn parse(&self, input: &mut &'a str) -> Result<&'a str, E> {
        let mut char_iter = input.chars();
        let mut pos = 0;
        for _ in 0..self.0 {
            if let Some(char) = char_iter.next() {
                pos += char.len_utf8();
            } else {
                return Err(E::from_parser_error(*input, ParserType::Tag));
            }
        }
        let (res, rest) = input.split_at(pos);
        *input = rest;
        Ok(res)
    }
}

impl<'a, T, E: ParserError> Parser<'a, [T], &'a [T], E, Take> for Take {
    fn parse(&self, input: &mut &'a [T]) -> Result<&'a [T], E> {
        if input.len() < self.0 {
            Err(E::from_parser_error(*input, ParserType::Tag))
        } else {
            let (res, rest) = input.split_at(self.0);
            *input = rest;
            Ok(res)
        }
    }
}

pub struct ParserFunction;

impl<'a, I: ?Sized, O, E: ParserError> Parser<'a, I, O, E, ParserFunction> for fn(&mut &'a I) -> Result<O, E> {
    fn parse(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        self(input).map_err(|err| {
            *input = checkpoint;
            err
        })
    }
}