use std::marker::PhantomData;

use crate::{Parser, ParserError};

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
            fn parse(&self, input: &mut &'a I) -> Result<($($otype,)+), E> {
                let startloc = *input;
                let ($($parserlower,)+) = self;
                let ($($rval,)+);
                $(
                    match $parserlower.parse(input) {
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

sequence_impl!(SeqAlt1 P1 p1 r1 O1 T1);
sequence_impl!(SeqAlt2 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2);
sequence_impl!(SeqAlt3 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3);
sequence_impl!(SeqAlt4 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4);
sequence_impl!(SeqAlt5 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5);
sequence_impl!(SeqAlt6 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6);
sequence_impl!(SeqAlt7 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7);
sequence_impl!(SeqAlt8 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8);
sequence_impl!(SeqAlt9 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9);
sequence_impl!(SeqAlt10 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9 P10 p10 r10 O10 T10);
sequence_impl!(SeqAlt11 P1 p1 r1 O1 T1 P2 p2 r2 O2 T2 P3 p3 r3 O3 T3 P4 p4 r4 O4 T4 P5 p5 r5 O5 T5 P6 p6 r6 O6 T6 P7 p7 r7 O7 T7 P8 p8 r8 O8 T8 P9 p9 r9 O9 T9 P10 p10 r10 O10 T10 P11 p11 r11 O11 T11);
