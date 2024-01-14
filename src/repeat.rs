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
pub trait TryReducer<Acc, T, FType, FErr> {
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), FErr>;
}

pub struct ResultReducer;
impl<Acc, T, F, FErr> TryReducer<Acc, T, ResultReducer, FErr> for F
where
    F: Fn(&mut Acc, T) -> Result<(), FErr>,
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), FErr> {
        self(acc, val).map(|_| ())
    }
}
pub struct OptionReducer();
impl<Acc, T, F> TryReducer<Acc, T, OptionReducer, TryReducerError> for F
where
    F: Fn(&mut Acc, T) -> Option<()>,
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), TryReducerError> {
        self(acc, val).ok_or(TryReducerError).map(|_| ())
    }
}
pub struct BoolReducer;
impl<Acc, T, F> TryReducer<Acc, T, BoolReducer, TryReducerError> for F
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
}

pub struct InfallibleReducer;
impl<Acc, T, F> TryReducer<Acc, T, InfallibleReducer, Infallible> for F
where
    F: Fn(&mut Acc, T) -> (),
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), Infallible> {
        Ok(self(acc, val)).map(|_| ())
    }
}
pub struct Reducer<F, Acc: Clone> {
    pub acc: Acc,
    pub reduce_fn: F,
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

//Checks that the repetition count is within the allowed repetitions. If it isn't, it resets the input to the original input
//and returns an error.
fn check_bounds<'a, I: ?Sized + Sequence, O, E: ParserError>(
    input: &mut &'a I,
    orig_input: &'a I,
    repetitions: usize,
    bounds: Range<usize>,
    res: O,
) -> Result<O, E> {
    if bounds.contains(&repetitions) {
        Ok(res)
    } else {
        *input = orig_input;
        Err(E::from_parser_error(*input, ParserType::Repeat))
    }
}

/**
 * This function is generics soup, but the goal is this:
 * It repeats applying the parser until it fails or exceeds the maximum bound.
 * It accumulates the output of the parser into Acc using F. If F returns an error
 * the parser also fails with that error. In iterator language, this is a TryReduce operator.
 */
impl<'a, P, I, O, E, PType, F, Acc, FErr, ReducerOut>
    Parser<'a, I, Acc, E, RepeatParser<PType, ReducerOut, FErr>> for Repeat<P, I, O, E, F, Acc>
where
    E: ParserError,
    I: ?Sized + Sequence,
    P: Parser<'a, I, O, E, PType>,
    Acc: Clone,
    FErr: 'static + Send + Sync + Error,
    F: TryReducer<Acc, O, ReducerOut, FErr>,
{
    fn fab(&self, input: &mut &'a I) -> Result<Acc, E> {
        let mut res = self.reducer.acc.clone();
        let mut repetitions: usize = 0;
        let mut last_location = *input;
        let orig_input = *input;
        loop {
            // Break out of the loop early if the parser exceeds the upper bound on repetitions.
            if repetitions >= self.bounds.end {
                *input = orig_input;
                return Err(E::from_parser_error(*input, ParserType::Repeat));
            }
            //This will be used if the try reduce fails to get a 
            //correct location of where the parser started.
            let loc_before_iteration = *input;
            match self.parser.fab(input) {
                //The parser succeeded, accumulate its output and continue parsing
                Ok(val) => {
                    //We made no progress, so break out of the loop
                    if loc(*input) == loc(last_location) {
                        return check_bounds(
                            input,
                            orig_input,
                            repetitions,
                            self.bounds.clone(),
                            res,
                        );
                    }
                    last_location = *input;
                    //The reduce operation can fail, so we need an if let for that case. It accumuates
                    //results by mutable reference, so there is no need for anything in the Ok case.
                    if let Err(err) = self.reducer.reduce_fn.try_reduce(&mut res, val) {
                        let mut err = E::from_external_error(loc_before_iteration, ParserType::Repeat, err);
                        *input = orig_input;
                        //Since the repeat error can occur anywhere in the sequence, add the
                        //start of the repeat to the context.
                        err.add_context(orig_input, ParserType::Repeat);
                        return Err(err);
                    }
                }
                Err(_) => {
                    //The underlying parser failed, so return the results up to here.
                    return check_bounds(input, orig_input, repetitions, self.bounds.clone(), res);
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
     * If it would exceed it, it fails.
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
        Repeat::new(self.parser, Reducer { acc, reduce_fn }, self.bounds)
    }
}
