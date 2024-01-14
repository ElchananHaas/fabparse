use crate::{opt, sequence::Sequence, Parser, ParserError};

/**
 * This string can be parsed as an unsigned number
 */
pub fn num_unsigned_str<'a, E: ParserError>(input: &mut &'a str) -> Result<&'a str, E>
where
{
    char::is_ascii_digit
        .fab_repeat()
        .min(1)
        .as_input_slice()
        .fab(input)
}
/**
 * This string can be parsed as a signed number
 */
pub fn num_signed_str<'a, E: ParserError>(input: &mut &'a str) -> Result<&'a str, E>
where
{
    let orig_input = *input;
    let _sign = opt('-').fab(input)?;
    let _rest = char::is_ascii_digit
        .fab_repeat()
        .min(1)
        .as_input_slice()
        .fab(input)?;
    Ok(orig_input.subtract(*input))
}
/**
 * This string can be parsed as a float or double
 */
pub fn float_str<'a, E: ParserError>(input: &mut &'a str) -> Result<&'a str, E>
where
{
    let orig_input = *input;
    let _sign = opt('-').fab(input)?;
    let _whole_part = char::is_ascii_digit
        .fab_repeat()
        .min(1)
        .as_input_slice()
        .fab(input)?;
    let _decimal_part = opt((
        '.',
        char::is_ascii_digit
        .fab_repeat()
        .min(1)
        .as_input_slice()
    )).fab(input)?;
    Ok(orig_input.subtract(*input))
}