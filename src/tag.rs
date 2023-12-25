use std::{error::Error, ops::{Range, RangeBounds}};

use crate::{Parser, ParserError, ParserType};

pub struct CharStrParser;
impl<'a> Parser<'a, str, char, CharStrParser> for char {
    fn parse<E: ParserError>(&self, input: &mut &'a str) -> Result<char, E> {
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
    fn parse<E: ParserError>(&self, input: &mut &'a [T]) -> Result<&'a [T], E> {
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
    fn parse<E: ParserError>(&self, input: &mut &'a [T]) -> Result<T, E> {
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
    fn parse<E: ParserError>(&self, input: &mut &'a [T]) -> Result<T, E> {
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
    fn parse<E: ParserError>(&self, input: &mut &'a str) -> Result<char, E> {
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
    fn parse<E: ParserError>(&self, input: &mut &'a [T]) -> Result<V, E> {
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
    fn parse<E: ParserError>(&self, input: &mut &'a str) -> Result<V, E> {
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
    fn parse<E: ParserError>(&self, input: &mut &'a [T]) -> Result<V, E> {
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
impl<'a, V, U, FErr> Parser<'a, str, V, FnResultStrParser> for U
where
    U: Fn(char) -> Result<V, FErr>,
    FErr: Error + Send + Sync + 'static,
{
    fn parse<E: ParserError>(&self, input: &mut &'a str) -> Result<V, E> {
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
impl<'a> Parser<'a, str, char, CharRangeStrParser> for Range<char> {
    fn parse<E: ParserError>(&self, input: &mut &'a str) -> Result<char, E> {
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
impl<'a, T, U> Parser<'a, [T], T, RangeSliceParser> for U
    where T: PartialOrd + Clone, U : RangeBounds<T> {
    fn parse<E: ParserError>(&self, input: &mut &'a [T]) -> Result<T, E> {
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
impl<'a> Parser<'a, str, &'a str, CharRangeStrParser> for Take {
    fn parse<E: ParserError>(&self, input: &mut &'a str) -> Result<&'a str, E> {
        let mut char_iter = input.chars();
        let mut pos = 0;
        for _ in 0..self.0 {
            if let Some(char) = char_iter.next(){
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
