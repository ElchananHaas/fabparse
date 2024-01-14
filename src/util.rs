use crate::{opt, sequence::Sequence, Parser, ParserError};

pub fn num_unsigned<'a, E: ParserError>(input: &mut &'a str) -> Result<&'a str, E>
where
{
    char::is_ascii_digit
        .fab_repeat()
        .as_input_slice()
        .fab(input)
}
pub fn num_signed<'a, E: ParserError>(input: &mut &'a str) -> Result<&'a str, E>
where
{
    let orig_input = *input;
    let _sign = opt('-').fab(input)?;
    let _rest = (|c: char| c.is_ascii_digit())
        .fab_repeat()
        .as_input_slice()
        .fab(input)?;
    Ok(orig_input.subtract(*input))
}
