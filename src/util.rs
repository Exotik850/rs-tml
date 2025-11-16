use crate::{ParseError, ParseResult};

// Parses nested content within delimiters
//
// Excludes escaped delimiters
//
// If there is no end given, it will look for the next occurrence of the start delimiter
pub fn nested<'a>(
    input: &'a str,
    start: &'a str,
    end: impl Into<Option<&'a str>>,
) -> ParseResult<'a, &'a str> {
    let input = input.trim_start();

    let end = end.into().unwrap_or(start);
    if !input.starts_with(start) {
        return Err(ParseError::invalid_input(
            input.chars().take(start.len()).collect::<String>(),
            Some(format!("Expected start delimiter: {start}").into()),
        ));
    }
    if start == end {
        return delimited(input, start);
    }

    let input = &input[start.len()..];
    let mut depth = 1;
    let mut i = 0;

    while i < input.len() {
        // Check for escape character
        if i > 0 && &input[i - 1..i] == "\\" {
            i += 1;
            continue;
        }

        // Check for start delimiter
        if input[i..].starts_with(start) {
            depth += 1;
            i += start.len();
            continue;
        }

        // Check for end delimiter
        if input[i..].starts_with(end) {
            depth -= 1;
            if depth == 0 {
                let content = &input[..i];
                let rest = &input[i + end.len()..];
                return Ok((rest, content));
            }
            i += end.len();
            continue;
        }

        i += 1;
    }

    Err(ParseError::missing_delimiter(end, input))
}

pub fn delimited<'a>(input: &'a str, delim: &'a str) -> ParseResult<'a, &'a str> {
    let input = input.trim_start();

    // Special case of nested where start and end are the same
    if !input.starts_with(delim) {
        return Err(ParseError::invalid_input(
            input.chars().take(delim.len()).collect::<String>(),
            Some("expected start delimiter".into()),
        ));
    }
    let input = &input[delim.len()..];
    // Look for the next occurrence of delim that is not escaped

    for (i, _) in input.match_indices(delim) {
        if i == 0 || &input[i - 1..i] != "\\" {
            let content = &input[..i];
            let rest = &input[i + delim.len()..];
            return Ok((rest, content));
        }
    }

    Err(ParseError::missing_delimiter(delim, "end of input"))
}

// Parses content nested within double quotes
pub fn quote_nested(input: &str) -> ParseResult<'_, &str> {
    delimited(input, "\"")
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::{ParseError, ParseResult};
    pub fn assert_parse_eq<'a, T: PartialEq + std::fmt::Debug>(
        result: ParseResult<'a, T>,
        expected_value: T,
        expected_rest: &'a str,
    ) {
        let (rest, value) = result.unwrap();
        assert_eq!(rest, expected_rest);
        assert_eq!(value, expected_value);
    }

    pub fn assert_parse_err<'a, T>(result: ParseResult<'a, T>, expected_error: ParseError<'a>) {
        let err = result.err().unwrap();
        assert_eq!(err, expected_error);
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::*;
    use crate::ParseError;

    #[test]
    fn test_nested() {
        let input = "{ level 1 { level 2 } level 1 continued } rest";
        assert_parse_eq(
            super::nested(input, "{", Some("}")),
            " level 1 { level 2 } level 1 continued ",
            " rest",
        );
    }

    #[test]
    fn test_delimited() {
        let input = "\"This is a test\" rest";
        assert_parse_eq(super::delimited(input, "\""), "This is a test", " rest");
    }

    #[test]
    fn test_empty_nested() {
        let input = "{} rest";
        assert_parse_eq(super::nested(input, "{", Some("}")), "", " rest");
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        assert_parse_err(
            super::nested(input, "{", Some("}")),
            ParseError::invalid_input("", Some("Expected start delimiter: {".into())),
        );
    }

    #[test]
    fn test_escaped_delimiter() {
        let input = "{ level 1 \\{ not a delimiter \\} still level 1 } rest";
        assert_parse_eq(
            super::nested(input, "{", Some("}")),
            " level 1 \\{ not a delimiter \\} still level 1 ",
            " rest",
        );
    }

    #[test]
    fn test_missing_end_delimiter() {
        let input = "{ level 1 { level 2 } level 1 continued rest";
        assert_parse_err(
            super::nested(input, "{", Some("}")),
            ParseError::missing_delimiter("}", " level 1 { level 2 } level 1 continued rest"),
        );
    }
}
