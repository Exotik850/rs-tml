use crate::{ParseError, ParseResult, nested};

// Represents a comment within RSTML
//
// Comments can be one-line or multi-line.
//
// One-line comments start with '//' and continue to the end of the line.
// Multi-line comments are enclosed within '/*' and '*/'.
//
// Currently, all comments are ignored during parsing.
// TODO: This going to change in the future to support documentation comments.
#[derive(Debug, PartialEq)]
pub enum Comment<'a> {
    Line(&'a str),
    Block(&'a str),
}

impl<'a> RSTMLParse<'a> for Comment<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        let input = input.trim_start();
        if let Some(rest) = input.strip_prefix("//") {
            if let Some((line, rest)) = rest.split_once('\n') {
                return Ok((rest, Comment::Line(line)));
            }
            return Ok(("", Comment::Line(rest)));
        } else if let Ok((rest, content)) = nested(input, "/*", "*/") {
            return Ok((rest, Comment::Block(content)));
        }
        Err(crate::ParseError::missing_token(
            "// or /*",
            input,
            std::borrow::Cow::Borrowed("Expected '//' for line comment or '/*' for block comment"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{Comment, RSTMLParse};
    use crate::test_util::*;

    #[test]
    fn test_comment_parse() {
        let input = r#"// This is a line comment"#;
        assert_parse_eq(
            Comment::parse_no_whitespace(input),
            Comment::Line(" This is a line comment"),
            "",
        );
    }

    #[test]
    fn test_block_comment_parse() {
        let input = r#"/* This is a block comment */"#;
        assert_parse_eq(
            Comment::parse_no_whitespace(input),
            Comment::Block(" This is a block comment "),
            "",
        );
    }
}

/// Trait for parsing RSTML items from a string input
pub trait RSTMLParse<'a> {
    /// Parses an item from the input, without ignoring leading whitespace
    ///
    /// # Errors
    /// Errors if parsing fails
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self>
    where
        Self: Sized;
}

/// Consumes all leading comments from the input string,
/// as well as any leading whitespace.
pub fn consume_comments(input: &str) -> &str {
    let mut input = input;
    loop {
        let trimmed_input = input.trim_start();
        if let Ok((rest, _comment)) = Comment::parse_no_whitespace(trimmed_input) {
            input = rest;
        } else {
            break;
        }
    }
    input.trim_start()
}

/// Extension trait providing additional parsing methods for RSTML items
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

    /// Parses an item from the input, ignoring comments and leading whitespace
    ///
    /// # Errors
    /// Errors if parsing fails, delegates to `parse_no_whitespace`
    fn parse_ignoring_comments(mut input: &'a str) -> ParseResult<'a, Self>
    where
        Self: Sized,
    {
        input = consume_comments(input);
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

    /// Parses as many items as possible, ignoring comments
    #[must_use]
    fn parse_many_ignoring_comments(mut input: &'a str) -> (&'a str, Vec<Self>)
    where
        Self: Sized,
    {
        let mut items = Vec::new();
        while let Ok((rest, item)) = Self::parse_ignoring_comments(input) {
            items.push(item);
            input = rest;
        }
        (input, items)
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
