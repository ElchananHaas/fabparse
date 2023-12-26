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

pub struct FnOptionSeqParser;
impl<'a, S, U, Item, FnOut> Parser<'a, S, FnOut, FnOptionSeqParser> for U
where
    Item: Clone,
    U: Fn(Item) -> Option<FnOut>,
    S: ?Sized + Sequence<Item = Item> + PartialEq,
{
    fn parse<E: ParserError>(&self, input: &mut &'a S) -> Result<FnOut, E> {
        if let Some((first, rest)) = Sequence::try_split_front(input) {
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
impl<'a, S, U, Item, FnOut, FnErr> Parser<'a, S, FnOut, FnResultSeqParser> for U
where
    Item: Clone,
    U: Fn(Item) -> Result<FnOut, FnErr>,
    S: ?Sized + Sequence<Item = Item> + PartialEq,
    FnErr: Error + Send + Sync + 'static,
{
    fn parse<E: ParserError>(&self, input: &mut &'a S) -> Result<FnOut, E> {
        if let Some((first, rest)) = Sequence::try_split_front(input) {
            match self(first.clone()) {
                Ok(out) => {
                    *input = rest;
                    Ok(out)
                },
                Err(err) => {
                    Err(E::from_external_error(*input, ParserType::Tag, err))
                }
            }
        } else {
            Err(E::from_parser_error(*input, ParserType::Tag))
        }
    }
}


pub struct RangeSeqParser;
impl<'a, Item, S, U> Parser<'a, S, Item, RangeSeqParser> for U
    where S: ?Sized + Sequence<Item = Item> + PartialEq,
    Item: PartialOrd,
    U: RangeBounds<Item> {
    fn parse<E: ParserError>(&self, input: &mut &'a S) -> Result<Item, E> {
        if let Some((first, rest)) = Sequence::try_split_front(input) {
            if self.contains(&first) {
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

pub struct Take(pub usize);
impl<'a, S> Parser<'a, S, &'a S, Take> for Take 
    where S: ?Sized + Sequence + PartialEq,
    {
    fn parse<E: ParserError>(&self, input: &mut &'a S) -> Result<&'a S, E> {
        let orig_input = *input;
        for _ in 0..self.0 {
            if let Some((_, rest)) = Sequence::try_split_front(input) {
                *input = rest;
            } else {
                *input = orig_input;
                return Err(E::from_parser_error(*input, ParserType::Tag));
            }
        }
        let pos = orig_input.len() - input.len();
        let (res, rest) = orig_input.try_split_at(pos).expect("Something went very wrong in the take parser");
        *input = rest;
        Ok(res)
    }
}