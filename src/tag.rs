use std::{
    error::Error,
    ops::{Range, RangeBounds},
};

use crate::{sequence::Sequence, Parser, ParserError, ParserType};

pub struct ItemSeqParser;
impl<'a, Item: PartialEq, I, E> Parser<'a, I, Item, E, ItemSeqParser> for Item
where
    I: ?Sized + Sequence<Item = Item>,
    E: ParserError,
{
    fn fab(&self, input: &mut &'a I) -> Result<Item, E> {
        if let Some((start, rest)) = Sequence::try_split_front(input) {
            if start == *self {
                *input = rest;
                Ok(start)
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct SeqSeqParser;

impl<'a, I, E> Parser<'a, I, &'a I, E, SeqSeqParser> for &I
where
    I: ?Sized + Sequence + PartialEq,
    E: ParserError,
{
    fn fab(&self, input: &mut &'a I) -> Result<&'a I, E> {
        if let Some((start, rest)) = input.try_split_at(self.len()) {
            if start == *self {
                *input = rest;
                Ok(start)
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct ConstArrayParser;

impl<'a, E, Item, const N: usize> Parser<'a, [Item], &'a [Item], E, ConstArrayParser> for [Item; N]
where
    E: ParserError,
    Item: Clone + PartialEq,
{
    fn fab(&self, input: &mut &'a [Item]) -> Result<&'a [Item], E> {
        self.as_slice().fab(input)
    }
}
pub struct FnBoolSliceParser;
impl<'a, T: Clone + 'a, U: Fn(T) -> bool, E: ParserError> Parser<'a, [T], T, E, FnBoolSliceParser>
    for U
{
    fn fab(&self, input: &mut &'a [T]) -> Result<T, E> {
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
    fn fab(&self, input: &mut &'a str) -> Result<char, E> {
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
where
    U: Fn(T) -> Option<V>,
{
    fn fab(&self, input: &mut &'a [T]) -> Result<V, E> {
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
where
    U: Fn(char) -> Option<V>,
{
    fn fab(&self, input: &mut &'a str) -> Result<V, E> {
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
    fn fab(&self, input: &mut &'a [T]) -> Result<V, E> {
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
    fn fab(&self, input: &mut &'a str) -> Result<V, E> {
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
    fn fab(&self, input: &mut &'a str) -> Result<char, E> {
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
    fn fab(&self, input: &mut &'a [T]) -> Result<T, E> {
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
impl<'a, I, E: ParserError> Parser<'a, I, &'a I, E, Take> for Take
where
    I: ?Sized + Sequence,
{
    fn fab(&self, input: &mut &'a I) -> Result<&'a I, E> {
        let orig = *input;
        let orig_len: usize = input.len();
        for _ in 0..self.0 {
            if let None = Sequence::try_split_front(input) {
                *input = orig;
                return Err(E::from_parser_error(*input, ParserType::Tag));
            }
        }
        let pos = orig_len - input.len();
        *input = orig;
        let (res, rest) = input
            .try_split_at(pos)
            .expect("Something went wrong in the take parser");
        *input = rest;
        Ok(res)
    }
}

pub struct ParserFunction;

impl<'a, I: ?Sized, O, E: ParserError> Parser<'a, I, O, E, ParserFunction>
    for fn(&mut &'a I) -> Result<O, E>
{
    fn fab(&self, input: &mut &'a I) -> Result<O, E> {
        let checkpoint = *input;
        self(input).map_err(|err| {
            *input = checkpoint;
            err
        })
    }
}
