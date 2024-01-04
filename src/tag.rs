use std::{error::Error, ops::RangeBounds};

use crate::{sequence::Sequence, Parser, ParserError, ParserType};

pub struct ItemSeqParser;
impl<'a, Item: PartialEq, I, E> Parser<'a, I, Item, E, ItemSeqParser> for Item
where
    I: ?Sized + Sequence<Item = Item>,
    E: ParserError,
{
    fn fab(&self, input: &mut &'a I) -> Result<Item, E> {
        if let Some((start, rest)) = input.try_split_front() {
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

pub struct FnBoolSeqParser;
impl<'a, I, F, E, Item> Parser<'a, I, Item, E, FnBoolSeqParser> for F
where
    I: ?Sized + Sequence<Item = Item>,
    F: Fn(Item) -> bool,
    E: ParserError,
    Item: Clone,
{
    fn fab(&self, input: &mut &'a I) -> Result<Item, E> {
        if let Some((first, rest)) = input.try_split_front() {
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

pub struct FnOptionSeqParser;
impl<'a, I, F, E, Item, FnOut> Parser<'a, I, FnOut, E, FnOptionSeqParser> for F
where
    I: ?Sized + Sequence<Item = Item>,
    F: Fn(Item) -> Option<FnOut>,
    E: ParserError,
    Item: Clone,
{
    fn fab(&self, input: &mut &'a I) -> Result<FnOut, E> {
        if let Some((first, rest)) = input.try_split_front() {
            if let Some(out) = self(first.clone()) {
                *input = rest;
                Ok(out)
            } else {
                Err(E::from_parser_error(*input, ParserType::Tag))
            }
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct FnResultSeqParser;
impl<'a, I, F, E, Item, FnOut, FnErr> Parser<'a, I, FnOut, E, FnResultSeqParser> for F
where
    I: ?Sized + Sequence<Item = Item>,
    F: Fn(Item) -> Result<FnOut, FnErr>,
    E: ParserError,
    Item: Clone,
    FnErr: 'static + Error + Send + Sync,
{
    fn fab(&self, input: &mut &'a I) -> Result<FnOut, E> {
        if let Some((first, rest)) = input.try_split_front() {
            match self(first.clone()) {
                Ok(out) => {
                    *input = rest;
                    Ok(out)
                }
                Err(err) => Err(E::from_external_error(*input, ParserType::Tag, err)),
            }
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}

pub struct RangeSeqParser;
impl<'a, Item, I, E, R> Parser<'a, I, Item, E, RangeSeqParser> for R
where
    I: ?Sized + Sequence<Item = Item>,
    E: ParserError,
    R: RangeBounds<Item>,
    Item: PartialOrd,
{
    fn fab(&self, input: &mut &'a I) -> Result<Item, E> {
        if let Some((start, rest)) = input.try_split_front() {
            if self.contains(&start) {
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

pub struct Take(pub usize);
impl<'a, I, E: ParserError> Parser<'a, I, &'a I, E, Take> for Take
where
    I: ?Sized + Sequence,
{
    fn fab(&self, input: &mut &'a I) -> Result<&'a I, E> {
        let orig = *input;
        let orig_len: usize = input.len();
        for _ in 0..self.0 {
            if let Some((_first, rest)) = input.try_split_front() {
                *input = rest;
            } else {
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

impl<'c, I: ?Sized, O, E: ParserError, F> Parser<'c, I, O, E, ParserFunction> for F
where
    F: Fn(&mut &'c I) -> Result<O, E>,
{
    fn fab(&self, input: &mut &'c I) -> Result<O, E> {
        let checkpoint = *input;
        self(input).map_err(|err| {
            *input = checkpoint;
            err
        })
    }
}
