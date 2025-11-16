use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    UnexpectedEndOfInput,
    EmptyInput,
    MissingEndDelimiter {
        expected: Cow<'a, str>,
        found: Cow<'a, str>,
    },
    InvalidInput {
        found: Cow<'a, str>,
        context: Option<Cow<'a, str>>,
    },
    MissingToken {
        expected: Cow<'a, str>,
        found: Cow<'a, str>,
        context: Option<Cow<'a, str>>,
    },
}

impl<'a> ParseError<'a> {
    pub fn invalid_input(
        found: impl Into<Cow<'a, str>>,
        context: impl Into<Option<Cow<'a, str>>>,
    ) -> Self {
        ParseError::InvalidInput {
            found: found.into(),
            context: context.into(),
        }
    }

    pub fn missing_token(
        expected: impl Into<Cow<'a, str>>,
        found: impl Into<Cow<'a, str>>,
        context: impl Into<Option<Cow<'a, str>>>,
    ) -> Self {
        ParseError::MissingToken {
            expected: expected.into(),
            found: found.into(),
            context: context.into(),
        }
    }

    pub fn missing_delimiter(
        expected: impl Into<Cow<'a, str>>,
        found: impl Into<Cow<'a, str>>,
    ) -> Self {
        ParseError::MissingEndDelimiter {
            expected: expected.into(),
            found: found.into(),
        }
    }
}

impl std::fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseError::EmptyInput => write!(f, "Input is empty"),
            ParseError::MissingEndDelimiter { expected, found } => {
                write!(
                    f,
                    "Missing end delimiter: expected '{expected}', found '{found}'"
                )
            }
            ParseError::InvalidInput { found, context } => {
                if let Some(ctx) = context {
                    write!(f, "Invalid input: '{found}'. Context: {ctx}")
                } else {
                    write!(f, "Invalid input: '{found}'")
                }
            }
            ParseError::MissingToken {
                found,
                expected,
                context,
            } => {
                if let Some(ctx) = context {
                    write!(
                        f,
                        "Missing token: expected '{expected}', found '{found}'. Context: {ctx}",
                    )
                } else {
                    write!(f, "Missing token: expected '{expected}', found '{found}'",)
                }
            }
        }
    }
}

pub type ParseResult<'a, T> = Result<(&'a str, T), ParseError<'a>>;
