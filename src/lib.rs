//! > Fabparse. A minimized parser combinator library.
//!  
//! Fabparse is a minimized trait based parser combinator library. Most of the 
//! functionality is within the [`Parser`] trait. Fabparse implements the parser trait
//! for a wide variety of types. It restricts itself to a few powerful and composable operations.
//! 
//! All of these are parsers.
//! 
//!| Parser | Input | Parsing |Output | Input after parsing|
//!| - | - | - | - | - |
//!| `'a'` | `let mut input = "abc"` | `'a'.fab(&mut input)` | `'a'` | `"bc"`|
//!| `'a'` | `let mut input = "def"` | `'a'.fab(&mut input)` | `FabError(...)` | `"def"`|
//!| `"abc"` | `let mut input = "abcdef"` | `"abc".fab(&mut input)` | `"abc"` | `"def"`|
//!| `[1, 2]` | `let mut input =  "[1, 2, 3].as_slice()"` | `[1, 2].fab(&mut input)` | `[1, 2]` | `[3]`|
//!| `('a'..='z')` | `let mut input = "zyx"` | `('a'..='z').fab(&mut input)` | `z` | `"yx"`|
//!| [`take`]`(2)` | `let mut input =  "abc"` | `take(2).fab(&mut input)` | `"ab"` | `"c"`|
//!| `❘c❘ c=='m'` | `let mut input = "moo"` | `(❘c❘ c=='m').fab(&mut input)` | `'m'` | `"oo"`|
//!| [`char::is_ascii_digit`] | `let mut input = "123"` | `char::is_ascii_digit.fab(&mut input)` | `'1'` | `"23"`|
//!| `let parser = ❘c: char❘ if c=='m' {Some(5)} else {None}` | `let mut input = "moo"` | `parser.fab(&mut input)` | `5` | `"oo"`|
//!| `let parser = ❘c: char❘ if c=='m' {Ok(5)} else {Err(ErrType)}` | `let mut input = "moo"` | `parser.fab(&mut input)` | `5` | `"oo"`|
//! 
//! 
//! 
//! Custom functions can also be parsers.
//! 
//! `
//! fn is_it_a(input: &mut &str) -> Result<char, FabError> {
//!     'a'.fab(input)
//! }
//! `
//! 
//!| Parser | Input | Parsing |Output | Input after parsing|
//!| - | - | - | - | - |
//!| `is_it_a` | `let mut input = "abc"` | `is_it_a.fab(&mut input)` | `'a'` | `"bc"`|
//! 
//! These parsers can be modified through the methods available 
//! in the [`Parser`] trait. 
//! 
//!| Parser | Input | Parsing |Output | Input after parsing|
//!| - | - | - | - | - |
//!| `let parser = 'a'.fab_value(5)` | `let mut input = "abc"` | `parser.fab(&mut input)` | `5` | `"bc"`|
//!| `let parser = 'a'.fab_map(`[`char::to_ascii_uppercase`]`)` | `let mut input = "abc"` | `parser.fab(&mut input)` | `A` | `"bc"`|
//!| `let parser = '1'.fab_try_map(❘c❘ c.to_digit(10))` | `let mut input = "123"` | `parser.fab(&mut input)` | `1` | `"23"`|
//!| `let parser = 'a'.fab_try_map(❘c❘ c.to_digit(10))` | `let mut input = "abc"` | `parser.fab(&mut input)` | `FabError(...)` | `"abc"`|
//!| `let parser = 'a'.fab_repeat()` | `let mut input = "aabb"` | `parser.fab(&mut input)` | `vec['a','a']` | `"bb"`|
//!| `let parser = 'a'.fab_repeat()` | `let mut input = "bbbb"` | `parser.fab(&mut input)` | `vec[]` | `"bbbb"`|
//!| `let parser = 'a'.fab_repeat().as_input_slice()` | `let mut input = "aabb"` | `parser.fab(&mut input)` | `"aa"` | `"bb"`|
//!| `let parser = 'a'.fab_repeat().min(1)` | `let mut input = "bbbb"` | `parser.fab(&mut input)` | `FabError(...)` | `"bbbb"`|
//! 
//! fab_try_map works both with functions that return Results and ones that return Options.
//! 
//! The [`Repeat`] struct has additional method for customization trait. These include setting a maximum
//! number of items to parse, or outputting a custom data structure.
//! 
//! These parsers can be combined with these methods. 
//! 
//!| Parser | Input | Parsing |Output | Input after parsing|
//!| - | - | - | - | - |
//!| `alt('a','b')` | `let mut input = "abc"` | `alt('a','b').fab(&mut input)` | `a` | `"bc"`|
//!| `alt('a','b')` | `let mut input = "bca"` | `alt('a','b').fab(&mut input)` | `b` | `"ca"`|
//!| `alt('a','b')` | `let mut input = "cab"` | `alt('a','b').fab(&mut input)` | `FabError(...)` | `"cab"`|
//!| `opt('a')` | `let mut input = "abc"` | `opt('a').fab(&mut input)` | `Some('a')` | `"bc"`|
//!| `opt('a')` | `let mut input = "cab"` | `opt('a').fab(&mut input)` | `None` | `"cab"`|
//!| `take_not('a')` | `let mut input = "cab"` | `take_not('a').fab(&mut input)` | `'c'` | `"ab"`|
//!| `take_not('a')` | `let mut input = "abc"` | `take_not('a').fab(&mut input)` | `FabError(...)` | `"abc"`|
//! 
//! Some code is inspired by Winnow by Elliot Page + other contributors.

#[doc(hidden)]
pub mod branch;
#[doc(hidden)]
pub mod combinator;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod repeat;
#[doc(hidden)]
pub mod sequence;
#[doc(hidden)]
pub mod tag;
pub mod util;

use std::{
    fmt::Debug,
    marker::PhantomData,
};

use combinator::{Opt, ParserMap, ParserTryMap, TakeNot, Value};
pub use error::FabError;
pub use error::ParserError;
pub use error::NoContextFabError;
pub use repeat::TryReducer;
pub use repeat::TryReducerError;
pub use repeat::Repeat;
use repeat::Reducer;
/**
 * This enum represents the kinds of parsers in Fabparse. This is used in errors to 
 * identify the parser that failed.
 */
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ParserType {
    Tag,
    Alt,
    Try,
    Map,
    TryMap,
    Function,
    TakeNot,
    Repeat,
    RepeatIter,
    Sequence,
    Permutation,
}


pub trait Parser<'a, I: ?Sized, O, E: ParserError, ParserType> {
    /**
     * Parses the input. This method advances the input reference to the remaining
     * unparsed input. The method is named "fab" instead of "parse" to avoid conflicts
     * with the "parse" method of &str.
     */
    fn fab(&self, input: &mut &'a I) -> Result<O, E>;
    /**
     * Returns a parser that replaces the output of the underlying parser with V.
     */
    fn fab_value<V: Clone>(self, value: V) -> combinator::Value<Self, V, I, O, E>
    where
        Self: Sized,
    {
        Value {
            parser: self,
            value,
            phantom_i: PhantomData,
            phantom_o: PhantomData,
            phantom_e: PhantomData,
        }
    }
    /**
     * This creates a Map parser that applies the function to the
     * output of the underlying parser.
     *
     */
    fn fab_map<F>(self, func: F) -> ParserMap<Self, I, O, E, F>
    where
        Self: Sized,
    {
        ParserMap {
            parser: self,
            func,
            phantom_i: PhantomData,
            phantom_e: PhantomData,
            phantom_m: PhantomData,
        }
    }
    /**
     * This parser first maps the input, and if the result is Option::Some or Result::Ok,
     * it unwraps the input. Othewise, the parser fails. 
     *
     */
    fn fab_try_map<F>(self, func: F) -> ParserTryMap<Self, I, O, E, F>
    where
        Self: Sized,
    {
        ParserTryMap {
            parser: self,
            func,
            phantom_i: PhantomData,
            phantom_e: PhantomData,
            phantom_m: PhantomData,
        }
    }
    /**
     * Repeats the underlying parser, returning the results in a Vec. This
     * parser will accept any number of repetitions, including 0.
     *
     * The repeat method has some methods to modify its behavior, which can be composed with each other. See the
     * [`Repeat`] struct for these emthods. 
     *
     */
    fn fab_repeat(self) -> Repeat<Self, I, O, E, fn(&mut Vec<O>, O) -> (), Vec<O>>
    where
        Self: Sized,
        O: Clone,
    {
        Repeat::new(
            self,
            Reducer {
                acc: Vec::new(),
                reduce_operator: |vec: &mut Vec<O>, val| vec.push(val),
            },
            0..usize::MAX,
        )
    }
}

/**
 * This function takes in a tuple of 1 to 11 parsers, all with the same
 * output type. It returns a parser that succeeds when any of its
 * input parsers succeed, with the output of the parser that succeeded.
 *
 * If none of the parsers succeed, this function will return an error.
 * When using `FabError`, the error returned will be the error of the parser that made the
 * furthest progress. When using a parser that doesn't provide error locations, or in the event
 * of ties, FunnelParse makes no garuntees as to which child parser's error will be returned.
 */
pub fn alt<T>(parsers: T) -> branch::Alt<T> {
    branch::Alt(parsers)
}

/**
 * This function takes in a tuple of 1 to 11 parsers. It returns a parser that
 * succeeds when all of the input parsers have succeeded in any order.
 * `permutation((parser_1,parser_2))` will return `(output_parser_1, output_parser_2)`,
 * regardless of the order they succeed in.
 *
 * If none of the parsers succeed, this function will return an error.
 * When using `FabError`, the error returned will be the error of the parser that made the
 * furthest progress. When using a parser that doesn't provide error locations, or in the event
 * of ties, FunnelParse makes no garuntees as to which child parser's error will be returned.
 */
pub fn permutation<T>(parsers: T) -> branch::Permutation<T> {
    branch::Permutation(parsers)
}

/**
 * `take(x: usize) `Constructs a parser that takes `x` items. For strings, this
 * will be characters and for arrays it will be elements. This parser outputs a &str for an input of &str
 * and a &\[T\] for an input of &\[T\]
 */
pub fn take(count: usize) -> tag::Take {
    tag::Take(count)
}

/**
 * This function makes the underlying parser optional. If the underlying parser succeeds with Ok(out),
 * this parser returns Some(out). Otherwise, this parser succeeds with None and
 * consumes no input.
 */
pub fn opt<T>(parser: T) -> combinator::Opt<T> {
    Opt { parser }
}
/**
 * Creates a parser that takes a single item if the underlying parser fails. If the
 * underlying parser succeeds, this parser fails. For strings, on success this will take a char
 * and for arrays it will take a single item. The result will be the char/item.
 *
 * An example usage of this is take_not('a'), which will recognize any single char except for 'a'.
 */
pub fn take_not<T>(parser: T) -> combinator::TakeNot<T> {
    TakeNot { parser }
}
