use std::marker::PhantomData;

use crate::{Parser, ParserError};

pub struct Alt<T>(pub T);

macro_rules! alt_impl {
    ( $tstruct:ident $( $parser:ident $parserlower:ident $ptype:ident)+ ) => {
        pub struct $tstruct<$($ptype,)+> {
            $(
                $parserlower : PhantomData<$ptype>,
            )+
        }

        #[allow(unused_assignments)]
        impl<'a, I: ?Sized, O, $($parser, $ptype,)+> Parser<'a, I, O, $tstruct<$($ptype,)+>> for Alt<($($parser,)+)>
            where $(
                $parser: Parser<'a, I, O, $ptype>,
            )+{
            fn parse<E: ParserError>(&mut self, input: &mut &'a I) -> Result<O, E> {
                let startloc = *input;
                let mut maxloc = None;
                let mut maxlocerr = None;
                let  ($($parserlower,)+) = &mut self.0;
                $(
                    match $parserlower.parse::<E>(input) {
                        Ok(res) => {
                            return Ok(res);
                        }
                        Err(err) => {
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
                return Err(maxlocerr.unwrap());
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
