use crate::{ParseError, ParseResult};

pub trait RSTMLParse<'a> {
    /// Parses an item from the input, without ignoring leading whitespace
    ///
    /// # Errors
    /// Errors if parsing fails
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self>
    where
        Self: Sized;
}

pub trait RSTMLParseExt<'a>: RSTMLParse<'a> {
    /// Parses an item from the input, ignoring leading whitespace
    ///
    /// # Errors
    /// Errors if parsing fails, delegates to `parse_no_whitespace`
    fn parse(input: &'a str) -> ParseResult<'a, Self>
    where
        Self: Sized,
    {
        let input = input.trim_start();
        Self::parse_no_whitespace(input)
    }

    /// Parses as many items as possible from the input
    ///
    /// # Errors
    /// Never errors; stops parsing on first failure
    fn parse_many(mut input: &'a str) -> ParseResult<'a, Vec<Self>>
    where
        Self: Sized,
    {
        let mut items = Vec::new();
        loop {
            let trimmed_input = input.trim_start();
            if trimmed_input.is_empty() {
                break;
            }
            match Self::parse_no_whitespace(trimmed_input) {
                Ok((rest, item)) => {
                    items.push(item);
                    input = rest;
                }
                Err(_) => break,
            }
        }
        Ok((input, items))
    }

    /// Parses exactly `n` items from the input
    ///
    /// # Errors
    /// Errors if fewer than `n` items can be parsed
    fn parse_n(mut input: &'a str, n: usize) -> ParseResult<'a, Vec<Self>>
    where
        Self: Sized,
    {
        let mut items = Vec::new();
        for _ in 0..n {
            let trimmed_input = input.trim_start();
            if trimmed_input.is_empty() {
                return Err(ParseError::UnexpectedEndOfInput);
            }
            match Self::parse_no_whitespace(trimmed_input) {
                Ok((rest, item)) => {
                    items.push(item);
                    input = rest;
                }
                Err(e) => return Err(e),
            }
        }
        Ok((input, items))
    }
}

impl<'a, T: RSTMLParse<'a>> RSTMLParseExt<'a> for T {}
