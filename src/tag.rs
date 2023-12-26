use std::{
    error::Error,
    ops::{Range, RangeBounds},
};

use crate::{Parser, ParserError, ParserType, Sequence};

pub struct ItemSeqParser;
impl<'a, Item, S> Parser<'a, S, Item, ItemSeqParser> for Item
where
    Item: PartialEq,
    S: ?Sized + Sequence<Item = Item>,
{
    fn parse<E: ParserError>(&self, input: &mut &'a S) -> Result<Item, E> {
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
impl<'a, Item, S> Parser<'a, S, &'a S, SeqSeqParser> for S
where
    S: ?Sized + Sequence<Item = Item> + PartialEq,
{
    fn parse<E: ParserError>(&self, input: &mut &'a S) -> Result<&'a S, E> {
        if let Some((start, rest)) = input.try_split_at(self.len()) {
            if start == self {
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
/**
 * This impl is needed to make constant arrays work as parsers.
 */
pub struct ConstGenericSeqParser;
impl<'a, T, const N: usize> Parser<'a, [T], &'a [T], ConstGenericSeqParser> for [T; N]
where
    T: PartialEq + Clone,
{
    fn parse<E: ParserError>(&self, input: &mut &'a [T]) -> Result<&'a [T], E> {
        self.as_slice().parse(input)
    }
}

pub struct FnBoolSeqParser;
impl<'a, S, U, Item> Parser<'a, S, Item, FnBoolSeqParser> for U
where
    Item: Clone,
    U: Fn(Item) -> bool,
    S: ?Sized + Sequence<Item = Item> + PartialEq,
{
    fn parse<E: ParserError>(&self, input: &mut &'a S) -> Result<Item, E> {
        if let Some((first, rest)) = Sequence::try_split_front(input) {
            if self(first.clone()) {
                *input = rest;
                Ok(first)
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
where
    T: PartialOrd + Clone,
    U: RangeBounds<T>,
{
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
impl<'a> Parser<'a, str, &'a str, Take> for Take {
    fn parse<E: ParserError>(&self, input: &mut &'a str) -> Result<&'a str, E> {
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

impl<'a, T> Parser<'a, [T], &'a [T], Take> for Take {
    fn parse<E: ParserError>(&self, input: &mut &'a [T]) -> Result<&'a [T], E> {
        if input.len() < self.0 {
            Err(E::from_parser_error(*input, ParserType::Tag))
        } else {
            let (res, rest) = input.split_at(self.0);
            *input = rest;
            Ok(res)
        }
    }
}
