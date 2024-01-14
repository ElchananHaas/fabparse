use std::{
    convert::Infallible, 
    error::Error,
    marker::PhantomData,
    ops::{Range, RangeBounds}, fmt::Display,
};

use crate::{sequence::Sequence, Parser, ParserError, ParserType};
/**
 * Repeat parsers can be customized with a custom try reduce function, see the TryReducer trait.
 * This error will be used for reducers that return Option<()> or 
 * bool. 
 * 
 * Reducers that return Result<(),E> will use the E from the result, 
 * and reducers that return () will never fail.
 */
#[derive(Clone, Debug, Copy)]
pub struct TryReducerError;
impl Display for TryReducerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TryReducerFailed")
    }
}

impl Error for TryReducerError {}

/**
 * Repeat parsers by default return a Vec. This behavior can be replaced with 
 * with the method `parser.reduce(acc, fn)`, where accumulator implements
 * TryReducer. This trait is already implemented for all of `[fn(&mut acc)->(),
 * fn(&mut acc)->Option<()>, fn(&mut acc)->bool, fn(&mut acc)->Result<(),E> ]`
 * 
 * `Acc`: The type of the accumulator
 * 
 * `T`: The type of the values that will be accumulated
 * 
 * `FType`: A dummy parameter used for disambiguating trait implementations.
 * You can use any struct defined with your crate.
 * 
 * `FErr`: The error type of the accumutation function.
 */
pub trait TryReducer<'a, Acc, T, FType, FErr, Out, I: ?Sized> {
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), FErr>;
    fn finalize(&self, acc: Acc, orig_input: &'a I, new_input: &'a I) -> Out;
}

pub struct ResultReducer;
impl<'a, Acc, T, F, FErr, I: ?Sized> TryReducer<'a, Acc, T, ResultReducer, FErr, Acc, I> for F
where
    F: Fn(&mut Acc, T) -> Result<(), FErr>,
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), FErr> {
        self(acc, val)
    }
    fn finalize(&self, acc: Acc, _orig_input: &'a I, _new_input: &'a I) -> Acc {
        acc
    }
}
pub struct OptionReducer();
impl<'a, Acc, T, F, I: ?Sized> TryReducer<'a, Acc, T, OptionReducer, TryReducerError, Acc, I> for F
where
    F: Fn(&mut Acc, T) -> Option<()>,
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), TryReducerError> {
        self(acc, val).ok_or(TryReducerError).map(|_| ())
    }
    fn finalize(&self, acc: Acc, _orig_input: &'a I, _new_input: &'a I) -> Acc {
        acc
    }
}
pub struct BoolReducer;
impl<'a, Acc, T, F, I: ?Sized> TryReducer<'a, Acc, T, BoolReducer, TryReducerError, Acc, I> for F
where
    F: Fn(&mut Acc, T) -> bool,
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), TryReducerError> {
        if self(acc, val) {
            Ok(())
        } else {
            Err(TryReducerError)
        }
    }
    fn finalize(&self, acc: Acc, _orig_input: &'a I, _new_input: &'a I) -> Acc {
        acc
    }
}

pub struct InfallibleReducer;
impl<'a, Acc, T, F, I: ?Sized> TryReducer<'a, Acc, T, InfallibleReducer, Infallible, Acc, I> for F
where
    F: Fn(&mut Acc, T) -> (),
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), Infallible> {
        Ok(self(acc, val)).map(|_| ())
    }
    fn finalize(&self, acc: Acc, _orig_input: &'a I, _new_input: &'a I) -> Acc {
        acc
    }
}

pub struct InputSliceReducer;
impl<'a, T, I: ?Sized> TryReducer<'a, (), T, InputSliceReducer, Infallible, &'a I, I> for InputSliceReducer
    where I: Sequence
{
    fn try_reduce(&self, _acc: &mut (), _val: T) -> Result<(), Infallible> {
        Ok(())
    }
    fn finalize(&self, _acc: (), orig_input: &'a I, new_input: &'a I) -> &'a I {
        let size =  orig_input.len() - new_input.len();
        let (first, _rest) = orig_input.try_split_at(size).expect("Valid split boundary");
        first
    }
}

pub struct Reducer<Reduce, Acc: Clone> {
    pub acc: Acc,
    pub reduce_operator: Reduce,
}
/**
 * This struct can be constructed through the method `fab_repeat` on any parser. 
 * It can be customized with a min/max number of repititions, or a custom 
 * try reduce.
 */
pub struct Repeat<P, ParI: ?Sized, ParO, ParE, F, Acc: Clone> {
    parser: P,
    reducer: Reducer<F, Acc>,
    bounds: Range<usize>,
    phantom_i: PhantomData<ParI>,
    phantom_o: PhantomData<ParO>,
    phantom_e: PhantomData<ParE>,
}

impl<P, ParI: ?Sized, ParO, ParE, F, Acc: Clone> Repeat<P, ParI, ParO, ParE, F, Acc> {
    /**
     * Constructs a new repeat parser. Prefer to use the method `fab_repeat` in the parser trait.
     */
    pub fn new(parser: P, reducer: Reducer<F, Acc>, bounds: Range<usize>) -> Self {
        Repeat {
            parser,
            reducer,
            bounds,
            phantom_i: PhantomData,
            phantom_o: PhantomData,
            phantom_e: PhantomData,
        }
    }
}

pub struct RepeatParser<PType, ReducerOut, FErr> {
    ptype: PhantomData<PType>,
    reducer_out: PhantomData<ReducerOut>,
    ferr: PhantomData<FErr>,
}

fn loc<I: ?Sized>(seq: &I) -> usize {
    seq as *const I as *const u8 as usize
}

/**
 * This function is generics soup, but the goal is this:
 * It repeats applying the parser until it fails or exceeds the maximum bound.
 * It accumulates the output of the parser into Acc using F. If F returns an error
 * the parser also fails with that error. In iterator language, this is a TryReduce operator.
 */
impl<'a, P, I, O, E, PType, F, Acc, FErr, ReducerOut, AccOut>
    Parser<'a, I, AccOut, E, RepeatParser<PType, ReducerOut, FErr>> for Repeat<P, I, O, E, F, Acc>
where
    E: ParserError,
    I: ?Sized + Sequence,
    P: Parser<'a, I, O, E, PType>,
    Acc: Clone,
    FErr: 'static + Send + Sync + Error,
    F: TryReducer<'a, Acc, O, ReducerOut, FErr, AccOut, I>,
{
    fn fab(&self, input: &mut &'a I) -> Result<AccOut, E> {
        let mut res = self.reducer.acc.clone();
        let mut repetitions: usize = 0;
        let mut last_location = *input;
        let orig_input = *input;
        if self.bounds.is_empty() {
            return Err(E::from_parser_error(*input, ParserType::Repeat));
        }
        loop {
            // Break out of the loop early if we hit the repetition limit.
            if repetitions == self.bounds.end - 1 {
                return Ok(self.reducer.reduce_operator.finalize(res, orig_input, input));
            }
            //This will be used if the try reduce fails to get a 
            //correct location of where the parser started.
            let loc_before_iteration = *input;
            match self.parser.fab(input) {
                //The parser succeeded, accumulate its output and continue parsing
                Ok(val) => {
                    //We made no progress, so return an error rather than looping indefinitely
                    if loc(*input) == loc(last_location) {
                        let mut err = E::from_parser_error(loc_before_iteration, ParserType::RepeatIter);
                        *input = orig_input;
                        err.add_context(orig_input, ParserType::Repeat);
                        return Err(err)
                    }
                    last_location = *input;
                    //The reduce operation can fail, so we need an if let for that case. It accumuates
                    //results by mutable reference, so there is no need for anything in the Ok case.
                    if let Err(err) = self.reducer.reduce_operator.try_reduce(&mut res, val) {
                        let mut err = E::from_external_error(loc_before_iteration, ParserType::RepeatIter, err);
                        *input = orig_input;
                        //Since the repeat error can occur anywhere in the sequence, add the
                        //start of the repeat to the context.
                        err.add_context(orig_input, ParserType::Repeat);
                        return Err(err);
                    }
                }
                Err(_) => {
                    //The underlying parser failed, so return the results up to here.
                    if self.bounds.contains(&repetitions) {
                        return Ok(self.reducer.reduce_operator.finalize(res, orig_input, input));
                    } else {
                        *input = orig_input;
                        return Err(E::from_parser_error(*input, ParserType::Repeat));
                    }
                }
            }
            repetitions += 1;
        }
    }
}

impl<P, ParI: ?Sized, ParO, ParE, F, Acc: Clone> Repeat<P, ParI, ParO, ParE, F, Acc> {
    /**
     * Sets an inclusive minimum number of repititions for this parser to succeed.
     */
    pub fn min(self, min: usize) -> Self {
        Repeat::new(self.parser, self.reducer, min..self.bounds.end)
    }
    /**
     * Sets as exclusive maximum limit of the the number of repititions of this parser.
     * When it hits the limit, it succeeds with its current output.
     */
    pub fn max(self, max: usize) -> Self {
        Repeat::new(self.parser, self.reducer, self.bounds.start..max)
    }
    /**
     * Sets both a minimum and maximum number of repitions for this parser to succeed.
     */
    pub fn bound<B: RangeBounds<usize>>(self, bounds: B) -> Self {
        let lower = match bounds.start_bound() {
            std::ops::Bound::Included(val) => *val,
            //The lower bound for a range shound never be usize::MAX.
            std::ops::Bound::Excluded(val) => {
                if *val == usize::MAX {
                    panic!("The lower bound for the range shouldn't be usize::MAX")
                } else {
                    *val + 1
                }
            }
            std::ops::Bound::Unbounded => 0,
        };
        let upper = match bounds.end_bound() {
            //An inclusive bound of usize::MAX makes sense, don't wrap it to 0.
            std::ops::Bound::Included(val) => {
                if *val == usize::MAX {
                    *val
                } else {
                    *val + 1
                }
            }
            std::ops::Bound::Excluded(val) => *val,
            std::ops::Bound::Unbounded => usize::MAX,
        };
        Repeat::new(self.parser, self.reducer, lower..upper)
    }
    /**
     * Returns the slice of the input that this parser matched. &str when parsing &str, &\[T\] when parsing  &\[T\]
     */
    pub fn as_input_slice(self) -> Repeat<P, ParI, ParO, ParE, InputSliceReducer, ()> {
        Repeat::new(self.parser, Reducer { acc: (), reduce_operator: InputSliceReducer }, self.bounds)
    }
    /**
     * By default this parser will output a vec. This method allows that to be replaced
     * with a custom type to costruct HashMaps or other custom output types. 
     * This method takes in `acc`, the accumulator base case and `reduce_fn` the 
     * custom try reduce function. `reduce_fn` can be of the form
     * `fn(&mut acc)->()` if it always succeeds. If if can fail, it can 
     * be of the forms `[fn(&mut acc)->Option<()>, fn(&mut acc)->bool, fn(&mut acc)->Result<(),E> ]`
     * It can also be a custom struct that implements the TryReducer trait.
     */
    pub fn reduce<NewAcc: Clone, NewF>(
        self,
        acc: NewAcc,
        reduce_fn: NewF,
    ) -> Repeat<P, ParI, ParO, ParE, NewF, NewAcc> {
        Repeat::new(self.parser, Reducer { acc, reduce_operator: reduce_fn }, self.bounds)
    }
}
