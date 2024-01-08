//Some code is based on Winnow by Elliot Page + other contributors.

mod branch;
mod combinator;
mod error;
mod repeat;
mod sequence;
mod tag;

use std::{
    fmt::Debug,
    marker::PhantomData,
};

use combinator::{Opt, ParserMap, ParserTryMap, TakeNot, Try, Value};
pub use error::FabError;
pub use error::ParserError;
pub use error::NoContextFabError;
pub use repeat::TryReducer;
pub use repeat::TryReducerError;
use repeat::{Reducer, Repeat};
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
    fn fab_value<V>(self, value: V) -> combinator::Value<Self, V, I, O, E>
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
     * This creates a Try parser that unwraps the output of the underlying parser
     * if it returns Result::Ok or Option::Some.
     * If it returns None or Err, then the Try parser will fail.
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
     * This parser composes the behavior of the Try and Map parsers. It
     * first maps the input, and if the result is Option::Some or Result::Ok,
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
     * parser will accept any number of repetitions, inlcuding 0.
     *
     * The repeat method has some methods to modify its behavior, which can be composed with each other. They are:
     *
     * min(usize). Sets an inclusive lower bound on the number of repetitions for the parser to succeed.
     *
     * max(usize). Sets an exclusive upper bound of the maximum number of repetitions of the parser for it to succeed.
     * If it would exceed this number, it fails.
     *
     * `bound(impl RangeBounds<usize>)`. Takes in any range type. repeat.bound(min..max) is equivilent to calling repeat.min(min).max(max)
     * This method can also accept min..=max, min.., ..=max, and ..
     *
     * reduce(acc, fn(&mut acc, O) -> ()) where O is the output type of the underlying parser.
     * reduce(acc, fn(&mut acc, O) -> Result<(), ErrorType>))
     * reduce(acc, fn(&mut acc, O) -> Option<()>)
     * reduce(acc, fn(&mut acc, O) -> bool)
     * retuce(acc, custom type implementing TryReducer)
     *
     * Replaces the Vec output of the parser. Every iteration of the repeat,
     * the repeat parser will call the reduction function with the current accumulator and the output of
     * the underlying parser. This can be used to create HashMaps or other data structures from the repeat parser.
     *
     * The accumulator must be Clone. Caution: The output of the reduction
     * function MUST be a unit type. HashMap::insert returns an option, not the unit type.
     * In some cases it might be easier to implement the TryReducer trait rather
     * than using a huge lambda.
     *
     * The reduce method also works with a function returning Result<(), ErrorType> where ErrorType is any type
     * that is 'static + Send + Sync + Error. In this case, the repeat parser will act as a try reduce, failing
     * when the reduction function returns an error. For the option and boolean cases, it will fail when the
     * function returns None or false, respectively.
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
                reduce_fn: |vec: &mut Vec<O>, val| vec.push(val),
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
 * will be characters and for arrays it will be elements. This parser always outputs a slice.
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
