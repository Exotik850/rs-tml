use std::borrow::Cow;

use crate::{error::ParseResult, parse::RSTMLParse};

// Represents plain text content within RSTML
//
// Text content is any sequence of characters that is surrounded by quotes
#[derive(PartialEq, Clone)]
pub struct Text<'a> {
    pub content: Cow<'a, str>,
}

impl<'a> Text<'a> {
    #[must_use]
    pub const fn new_const(content: Cow<'a, str>) -> Self {
        Text { content }
    }
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self::new_const(content.into())
    }
}

impl From<String> for Text<'_> {
    fn from(value: String) -> Self {
        Text::new_const(value.into())
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(value: &'a str) -> Self {
        Text::new_const(value.into())
    }
}

impl std::fmt::Display for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl std::fmt::Debug for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{:?}\"", self.content)
    }
}

impl<'a> RSTMLParse<'a> for Text<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        let (rest, content) = crate::util::quote_nested(input)?;
        Ok((rest, Text::new(content)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse::RSTMLParse, test_util::assert_parse_eq};

    use super::Text;

    #[test]
    fn test_text_parse() {
        let input = r#""Hello, World!""#;
        assert_parse_eq(
            Text::parse_no_whitespace(input),
            Text::new("Hello, World!"),
            "",
        );
    }

    #[test]
    fn test_text_no_quotes_invalid() {
        let input = r#"Hello, World!"#;
        let result = Text::parse_no_whitespace(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_closing_quote() {
        let input = r#""Hello, World!"#;
        let result = Text::parse_no_whitespace(input);
        assert!(result.is_err());
    }
}
