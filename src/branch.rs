use std::marker::PhantomData;

use crate::{Parser, ParserError};

pub struct Alt<T>(pub T);
pub struct Permutation<T>(pub T);

macro_rules! alt_impl {
    ( $tstruct:ident $( $parser:ident $parserlower:ident $ptype:ident)+ ) => {
        pub struct $tstruct<$($ptype,)+> {
            $(
                $parserlower : PhantomData<$ptype>,
            )+
        }

        #[allow(unused_assignments)]
        impl<'a, I: ?Sized, O, E: ParserError, $($parser, $ptype,)+> Parser<'a, I, O, E, $tstruct<$($ptype,)+>> for Alt<($($parser,)+)>
            where $(
                $parser: Parser<'a, I, O, E, $ptype>,
            )+{
            fn fab(&self, input: &mut &'a I) -> Result<O, E> {
                let startloc = *input;
                let mut maxloc = None;
                let mut maxlocerr = None;
                let  ($($parserlower,)+) = &self.0;
                $(
                    match $parserlower.fab(input) {
                        Ok(res) => {
                            return Ok(res);
                        }
                        Err(err) => {
                            //If the error type supports location, take the error from the
                            //parser that made the most progress.
                            if let Some(loc) = err.get_loc() {
                                if maxloc.is_none() || maxloc.is_some_and(|val| loc >= val) {
                                    maxloc = Some(loc);
                                    maxlocerr = Some(err);
                                }
                            } else {
                                maxlocerr = Some(err);
                            }
                        }
                    }
                    *input = startloc;
                )+
                //Alt is only implemented for tuples with at least 1 element, so we will always have some error.
                return Err(maxlocerr.expect("Something went wrong in the alt parser."));
            }
        }

    };
}

alt_impl!(Alt1 P1 p1 T1);
alt_impl!(Alt2 P1 p1 T1 P2 p2 T2);
alt_impl!(Alt3 P1 p1 T1 P2 p2 T2 P3 p3 T3);
alt_impl!(Alt4 P1 p1 T1 P2 p2 T2 P3 p3 T3 P4 p4 T4);
alt_impl!(Alt5 P1 p1 T1 P2 p2 T2 P3 p3 T3 P4 p4 T4 P5 p5 T5);
alt_impl!(Alt6 P1 p1 T1 P2 p2 T2 P3 p3 T3 P4 p4 T4 P5 p5 T5 P6 p6 T6);
alt_impl!(Alt7 P1 p1 T1 P2 p2 T2 P3 p3 T3 P4 p4 T4 P5 p5 T5 P6 p6 T6 P7 p7 T7);
alt_impl!(Alt8 P1 p1 T1 P2 p2 T2 P3 p3 T3 P4 p4 T4 P5 p5 T5 P6 p6 T6 P7 p7 T7 P8 p8 T8);
alt_impl!(Alt9 P1 p1 T1 P2 p2 T2 P3 p3 T3 P4 p4 T4 P5 p5 T5 P6 p6 T6 P7 p7 T7 P8 p8 T8 P9 p9 T9);
alt_impl!(Alt10 P1 p1 T1 P2 p2 T2 P3 p3 T3 P4 p4 T4 P5 p5 T5 P6 p6 T6 P7 p7 T7 P8 p8 T8 P9 p9 T9 P10 p10 T10);
alt_impl!(Alt11 P1 p1 T1 P2 p2 T2 P3 p3 T3 P4 p4 T4 P5 p5 T5 P6 p6 T6 P7 p7 T7 P8 p8 T8 P9 p9 T9 P10 p10 T10 P11 p11 T11);

macro_rules! permutation_impl {
    ( $tstruct:ident $($parser:ident $parserlower:ident $rval:ident $otype:ident $ptype:ident)+ ) => {
        pub struct $tstruct<$($ptype,)+> {
            $(
                $parserlower : PhantomData<$ptype>,
            )+
        }

        #[allow(unused_assignments)]
        impl<'a, I: ?Sized, $($otype, )+ E: ParserError, $($parser, $ptype,)+> Parser<'a, I, ($($otype,)+), E, $tstruct<$($ptype,)+>> for Permutation<($($parser,)+)>
            where $(
                $parser: Parser<'a, I, $otype, E, $ptype>,
            )+{
            fn fab(&self, input: &mut &'a I) -> Result<($($otype,)+), E> {
                let outer_startloc = *input;
                let ($($parserlower,)+) = &self.0;
                $(
                    let mut $rval = None;
                )+
                loop {
                    let startloc = *input;
                    let mut done = true;
                    let mut maxloc = None;
                    let mut maxlocerr = None;
                    $(
                        if ($rval.is_none()) {
                            done = false;
                            match $parserlower.fab(input) {
                                Ok(res) => {
                                    $rval = Some(res);
                                    continue;
                                }
                                Err(err) => {
                                    //If the error type supports location, take the error from the
                                    //parser that made the most progress.
                                    if let Some(loc) = err.get_loc() {
                                        if maxloc.is_none() || maxloc.is_some_and(|val| loc >= val) {
                                            maxloc = Some(loc);
                                            maxlocerr = Some(err);
                                        }
                                    } else {
                                        maxlocerr = Some(err);
                                    }
                                    *input = startloc;
                                }
                            }
                        }
                    )+
                    //At this point, none of the parsers have made any progress
                    if (done) {
                        return Ok(($($rval.expect("Something went wrong in the permutation implementation"),)+));
                    } else {
                        *input = outer_startloc;
                        //If no parsers fail, then done will be true and this line won't run.
                        return Err(maxlocerr.expect("Something went wrong in the permutation parser."))
                    }
                }
            }
        }

    };
}

permutation_impl!(SeqAlt1 P1 p1 r1 O1 T1);
permutation_impl!(SeqAlt2 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2);
permutation_impl!(SeqAlt3 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3);
permutation_impl!(SeqAlt4 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4);
permutation_impl!(SeqAlt5 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5);
permutation_impl!(SeqAlt6 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6);
permutation_impl!(SeqAlt7 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7);
permutation_impl!(SeqAlt8 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8);
permutation_impl!(SeqAlt9 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9);
permutation_impl!(SeqAlt10 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9 P10 p10 r10 O10 T10);
permutation_impl!(SeqAlt11 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9 P10 p10 r10 O10 T10 P11 p11 r11 O11 T11);

macro_rules! sequence_impl {
    ( $tstruct:ident $( $parser:ident $parserlower:ident $rval:ident $otype:ident $ptype:ident)+ ) => {
        pub struct $tstruct<$($ptype,)+> {
            $(
                $parserlower : PhantomData<$ptype>,
            )+
        }

        #[allow(unused_assignments)]
        impl<'a, I: ?Sized, $($otype, )+ E: ParserError, $($parser, $ptype,)+> Parser<'a, I, ($($otype,)+), E, $tstruct<$($ptype,)+>> for ($($parser,)+)
            where $(
                $parser: Parser<'a, I, $otype, E, $ptype>,
            )+{
            fn fab(&self, input: &mut &'a I) -> Result<($($otype,)+), E> {
                let startloc = *input;
                let ($($parserlower,)+) = self;
                let ($($rval,)+);
                $(
                    match $parserlower.fab(input) {
                        Ok(res) => {
                            $rval = res;
                        }
                        Err(err) => {
                            *input = startloc;
                            return Err(err);
                        }
                    }
                )+
                Ok(($($rval,)+))
            }
        }

    };
}

sequence_impl!(Seq1 P1 p1 r1 O1 T1);
sequence_impl!(Seq2 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2);
sequence_impl!(Seq3 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3);
sequence_impl!(Seq4 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4);
sequence_impl!(Seq5 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5);
sequence_impl!(Seq6 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6);
sequence_impl!(Seq7 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7);
sequence_impl!(Seq8 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8);
sequence_impl!(Seq9 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9);
sequence_impl!(Seq10 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9 P10 p10 r10 O10 T10);
sequence_impl!(Seq11 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9 P10 p10 r10 O10 T10 P11 p11 r11 O11 T11);
