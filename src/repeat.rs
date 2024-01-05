use std::{
    convert::Infallible,
    error::Error,
    marker::PhantomData,
    ops::{Range, RangeBounds},
};

use crate::{sequence::Sequence, Parser, ParserError, ParserType, UnitError};

/**
 * Trait for any function which can act as a try reducer.
 * Acc: The type of the accumulator
 * T: The type of the values that will be accumulated
 * FType: A dummy parameter used for disambiguating trait implementations.
 * You can use any struct defined with your crate.
 * FErr: The error type of the accumutation function.
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
impl<Acc, T, F> TryReducer<Acc, T, OptionReducer, UnitError> for F
where
    F: Fn(&mut Acc, T) -> Option<()>,
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), UnitError> {
        self(acc, val).ok_or(UnitError).map(|_| ())
    }
}
pub struct BoolReducer;
impl<Acc, T, F> TryReducer<Acc, T, BoolReducer, UnitError> for F
where
    F: Fn(&mut Acc, T) -> bool,
{
    fn try_reduce(&self, acc: &mut Acc, val: T) -> Result<(), UnitError> {
        if self(acc, val) {
            Ok(())
        } else {
            Err(UnitError)
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
pub struct Repeat<P, ParI: ?Sized, ParO, ParE, F, Acc: Clone> {
    pub parser: P,
    pub reducer: Reducer<F, Acc>,
    pub bounds: Range<usize>,
    pub phantom_i: PhantomData<ParI>,
    pub phantom_o: PhantomData<ParO>,
    pub phantom_e: PhantomData<ParE>,
}

impl<P, ParI: ?Sized, ParO, ParE, F, Acc: Clone> Repeat<P, ParI, ParO, ParE, F, Acc> {
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
    pub fn min(self, min: usize) -> Self {
        Repeat::new(self.parser, self.reducer, min..self.bounds.end)
    }

    pub fn max(self, max: usize) -> Self {
        Repeat::new(self.parser, self.reducer, self.bounds.start..max)
    }

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

    pub fn reduce<NewAcc: Clone, NewF>(
        self,
        acc: NewAcc,
        reduce_fn: NewF,
    ) -> Repeat<P, ParI, ParO, ParE, NewF, NewAcc> {
        Repeat::new(self.parser, Reducer { acc, reduce_fn }, self.bounds)
    }
}
