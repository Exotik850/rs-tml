use crate::error::{ParseError, ParseResult};

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

    let mut search_offset = 0;
    while let Some(next_delim_pos) = input[search_offset..]
        .find(start)
        .map(|p| (p, true))
        .into_iter()
        .chain(input[search_offset..].find(end).map(|p| (p, false)))
        .min_by_key(|&(p, _)| p)
    {
        let (relative_pos, is_start_delim) = next_delim_pos;
        search_offset += relative_pos;

        // Check for escape character. If found, skip this delimiter and continue searching.
        if search_offset > 0 && input.as_bytes()[search_offset - 1] == b'\\' {
            search_offset += 1;
            continue;
        }

        if is_start_delim {
            depth += 1;
            search_offset += start.len();
        } else {
            depth -= 1;
            if depth == 0 {
                let content = &input[..search_offset];
                let rest = &input[search_offset + end.len()..];
                return Ok((rest, content));
            }
            search_offset += end.len();
        }
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

        if rest != expected_rest || value != expected_value {
            panic!(
                "Assertion failed:\nExpected rest: {:#?}\nActual rest:   {:#?}\nExpected value: {:?}\nActual value:   {:?}",
                expected_rest, rest, expected_value, value
            );
        }
    }

    pub fn assert_parse_err<'a, T>(result: ParseResult<'a, T>, expected_error: ParseError<'a>) {
        let err = result.err().unwrap();
        if err != expected_error {
            panic!(
                "Assertion failed:\nExpected error: {:#?}\nActual error:   {:#?}",
                expected_error, err
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::*;
    use crate::error::ParseError;

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
