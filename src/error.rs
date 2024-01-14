use std::{
    error::Error,
    fmt::{Debug, Display},
};

use smallvec::{smallvec, SmallVec};

use crate::{sequence::Sequence, ParserType};

/**
 * Trait for a parser error. This can store information about the type of parser
 * that generated the error and its location. This is implemented by `FabError` and
 * `NoContextFabError`.
 *
 * In order to simplify lifetimes used by the error, the parser error
 * stores a pointer to the location the error occured rather than a reference.
 * This doesn't require unsafe to use properly.
 */
pub trait ParserError {
    fn from_parser_error<T: ?Sized + Sequence>(input: *const T, parser_type: ParserType) -> Self;
    fn from_external_error<T: ?Sized + Sequence, E: Error + Send + Sync + 'static>(
        input: *const T,
        parser_type: ParserType,
        cause: E,
    ) -> Self;
    fn add_context<T: ?Sized + Sequence>(&mut self, _input: *const T, _parser_type: ParserType) {}
    /**
     * Get the location of the error. This is used in combinators to recognize the parser that made
     * the furthest progress.
     */
    fn get_loc(&self) -> Option<usize> {
        None
    }
}

#[derive(Debug, Clone)]
struct LocatedError {
    location: usize,
    parser_type: ParserType,
}
/**
 * This error type has the FabError trait implemented for it,
 * but contains no information about the location of the error or
 * the parser that generated it. Use this error for maximum speed at
 * the cost of debugability. Use FabError for better debugging.
 */
#[derive(Debug, Clone)]
pub struct NoContextFabError;
impl Display for NoContextFabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UnitParserError")
    }
}
impl Error for NoContextFabError {}
/**
 * If you don't care about the errors, and want speed, this type implements ParserError.
 * It contains no information.
 */
impl ParserError for NoContextFabError {
    fn from_parser_error<T: ?Sized + Sequence>(_input: *const T, _parser_type: ParserType) -> Self {
        NoContextFabError
    }

    fn from_external_error<T: ?Sized, E: Error + Send + Sync + 'static>(
        _input: *const T,
        _parser_type: ParserType,
        _cause: E,
    ) -> Self {
        NoContextFabError
    }
}

/**
 * This is the recommended error type for Fabparse. This can be pretty-printed with the method
 * `print_trace(input)`, which will print a stack trace of the parsers with
 * some surrounding context. An example is:
 *
 *
 * Location \[""\]^[\"a1b2c3\"] from parser Repeat
 *
 * Location \["a1b2"\]^\["c3"\] from parser RepeatIter
 *
 * From cause \[TryReducerFailed\]
 *
 * There is also a `print_trace_window(input)` method that lets you customize
 * the context window. The default is currently 10, but Fabparse reserves the
 * right to increase this.
 */
#[derive(Debug)]
pub struct FabError {
    //Use a smallvec for the stack so non-combinator
    //parsers won't need to allocate
    stack: SmallVec<[LocatedError; 1]>,
    cause: Option<Box<dyn Error>>,
}
/**
 * This is the default error for Fabparse.
 * It has a method print_trace(input) which prints a stack trace of
 * the parser error, with context. An example trace on input "a1b2c3" is:
 *
 * Location \[""\]^[\"a1b2c3\"] from parser Repeat
 *
 * Location \["a1b2"\]^\["c3"\] from parser Repeat
 *
 * From cause \[TryReducerFailed\]
 *
 * This method requires that you pass in the input that generated the error.
 * If you don't the method may panic or print an incorrect stack trace.
 *
 * This error type also has a method print_trace_window(input, window_size)
 * which controls how much context is printed. By default, it will be 10 chars or items
 */
impl Display for FabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FabError( Stack: {:?}, Cause: {:?})",
            self.stack, self.cause
        )
    }
}
impl Error for FabError {}

impl ParserError for FabError {
    fn from_parser_error<T: ?Sized>(input: *const T, parser_type: ParserType) -> Self {
        FabError {
            stack: smallvec![LocatedError {
                parser_type,
                location: input as *const u8 as usize
            }],
            cause: None,
        }
    }
    fn from_external_error<T: ?Sized, E: Error + Send + Sync + 'static>(
        input: *const T,
        parser_type: ParserType,
        cause: E,
    ) -> Self {
        FabError {
            stack: smallvec![LocatedError {
                parser_type,
                location: input as *const u8 as usize
            }],
            cause: Some(Box::new(cause)),
        }
    }
    fn get_loc(&self) -> Option<usize> {
        return Some(self.stack[0].location);
    }
    fn add_context<T: ?Sized + Sequence>(&mut self, input: *const T, parser_type: ParserType) {
        self.stack.push(LocatedError {
            location: input as *const u8 as usize,
            parser_type,
        })
    }
}
/**
 * Gets a slice of window elements from the start of the sequence
 */
fn get_from_start<I: ?Sized + Sequence>(input: &I, window: usize) -> &I {
    let mut current_start = input;
    for _ in 0..window {
        if let Some((_, rest)) = current_start.try_split_front() {
            current_start = rest;
        } else {
            break;
        }
    }
    let index_diff = input.len() - current_start.len();
    input
        .try_split_at(index_diff)
        .expect("Get from start found a valid split.")
        .0
}
/**
 * Gets a slice of window elements from the end of the sequence.
 */
fn get_from_end<I: ?Sized + Sequence>(input: &I, window: usize) -> &I {
    let mut end_index = input.len();
    let mut count = 0;
    while count < window && end_index > 0 {
        end_index -= 1;
        if let Some(_) = input.try_split_at(end_index) {
            count += 1;
        }
    }
    input
        .try_split_at(end_index)
        .expect("Get from end found a valid split.")
        .1
}
/**
 * Gets window elements of the surrounding context, both forwards and backwards.
 * We need to use try split to handle strings correctly, which can only be split at char boundries.
 *
 * Requires place to be value of a valid pointer to within the sequence as usize, otherwise will panic.
 */
fn get_surrounding_context<I: ?Sized + Sequence>(
    input: &I,
    place: usize,
    window: usize,
) -> (&I, &I) {
    let start = input as *const I as *const u8 as usize;
    assert!(place >= start);
    let index = place - start;
    let (before, after) = input
        .try_split_at(index)
        .expect("Place is a valid split boundary");
    (get_from_end(before, window), get_from_start(after, window))
}

impl FabError {
    pub fn print_trace<I: ?Sized + Sequence + Debug>(&self, parser_input: &I) {
        self.print_trace_window(parser_input, 10);
    }
    pub fn print_trace_window<I: ?Sized + Sequence + Debug>(
        &self,
        parser_input: &I,
        window: usize,
    ) {
        for item in self.stack.iter().rev() {
            let (before, after) = get_surrounding_context(parser_input, item.location, window);
            println!(
                "Location [{:?}]^[{:?}] from parser {:?}",
                before, after, item.parser_type
            )
        }
        if let Some(cause) = &self.cause {
            println!("From cause [{}]", cause);
        }
    }
}

mod test {
    #[allow(unused_imports)]
    use crate::error::*;

    #[test]
    fn test_get_surrounding_context_success() {
        let input = "abcdefgh";
        let place = (input as *const str as *const u8 as usize) + 4;
        let (start, rest) = get_surrounding_context(input, place, 3);
        assert_eq!(start, "bcd");
        assert_eq!(rest, "efg");
    }

    #[test]
    fn test_get_surrounding_context_trimmed() {
        let input = "abcd";
        let place = (input as *const str as *const u8 as usize) + 2;
        let (start, rest) = get_surrounding_context(input, place, 3);
        assert_eq!(start, "ab");
        assert_eq!(rest, "cd");
    }
}
