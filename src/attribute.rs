use std::borrow::Cow;

use crate::prelude::*;
use pastey::paste;

/// Represents an RSTML attribute
///
/// Attributes are key-value pairs associated with RSTML elements.
/// The key is a string representing the attribute name, and the value is a string representing the attribute value.
///
/// Keys start with periods and are typically lowercase words separated by hyphens.
/// Values are usually enclosed in double quotes.
///
/// Keys without values are treated as class attributes with the value of the key name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute<'a> {
    pub key: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

macro_rules! attribute {
    ($($attribute:ident)*) => {
        $(
            paste! {
                pub const fn [<$attribute:lower>](value: &'a str) -> Attribute<'a> {
                    Attribute::new_const(
                      Cow::Borrowed(stringify!([<$attribute:lower>])), 
                      Cow::Borrowed(value)
                    )
                }
            }
        )*
    };
}

impl<'a> Attribute<'a> {
    #[must_use]
    pub const fn new_const(key: Cow<'a, str>, value: Cow<'a, str>) -> Self {
        Attribute { key, value }
    }
    pub fn new(key: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Attribute {
            key: key.into(),
            value: value.into(),
        }
    }

    // TODO : add type attribute, but it's a reserved keyword
    attribute!(id class href src alt title style name value placeholder disabled checked readonly);
}
impl std::fmt::Display for Attribute<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.key, self.value)
    }
}

impl<'a, T, U> From<(T, U)> for Attribute<'a>
where
    T: Into<Cow<'a, str>>,
    U: Into<Cow<'a, str>>,
{
    fn from((key, value): (T, U)) -> Self {
        Attribute::new(key, value)
    }
}

fn get_attribute_key(key: &str) -> ParseResult<'_, &str> {
    if key.is_empty() {
        return Err(ParseError::missing_token(
            ".[name]",
            key,
            Some("Attribute key cannot be empty".into()),
        ));
    }
    if !key.starts_with('.') {
        return Err(ParseError::invalid_input(
            key,
            Some("Attribute key must start with a period or a '#'".into()),
        ));
    }
    let key = &key[1..]; // Remove the leading period
    let Ok((rest, key)) = Tag::parse_no_whitespace(key) else {
        return Err(ParseError::invalid_input(
            key,
            Some("Invalid attribute key format".into()),
        ));
    };
    Ok((rest, key.name))
}

impl<'a> RSTMLParse<'a> for Attribute<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        // Handle #id shorthand syntax
        if let Some(id_value) = input.strip_prefix('#') {
            // Remove the leading #
            let Ok((rest, id)) = Tag::parse_no_whitespace(id_value) else {
                return Err(ParseError::invalid_input(
                    input,
                    Some("Invalid id format".into()),
                ));
            };
            return Ok((rest, Attribute::id(id.name)));
        }

        let Some((key, rest)) = input.split_once('=') else {
            // Handle case where attribute has no value, treat as class with value of key name
            // e.g., .class becomes .class="class"
            return get_attribute_key(input).map(|(rest, key)| (rest, Attribute::class(key)));
        };
        let (_, key) = get_attribute_key(key.trim_end())?;
        let (rest, value) = crate::quote_nested(rest.trim_start())?;
        Ok((rest, Attribute::new(key, value)))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::test_util::*;

    #[test]
    fn test_attribute_parse() {
        let input = r#".class="my-class" .id="element-id""#;
        assert_parse_eq(
            Attribute::parse_no_whitespace(input),
            Attribute::class("my-class"),
            r#" .id="element-id""#,
        );

        assert_parse_eq(
            Attribute::parse(r#" .id="element-id""#),
            Attribute::id("element-id"),
            "",
        );
    }

    #[test]
    fn test_attribute_parse_no_value() {
        let input = r#".disabled .readonly"#;
        assert_parse_eq(
            Attribute::parse_no_whitespace(input),
            Attribute::class("disabled"),
            r#" .readonly"#,
        );
        assert_parse_eq(
            Attribute::parse(r#" .readonly"#),
            Attribute::class("readonly"),
            "",
        );
    }

    #[test]
    fn test_id_parse() {
        let input = r#"#unique-id"#;
        assert_parse_eq(
            Attribute::parse_no_whitespace(input),
            Attribute::id("unique-id"),
            "",
        );
    }

    #[test]
    fn test_attribute_parse_invalid() {
        let input = r#"class=my-class"#;
        assert_parse_err(
            Attribute::parse_no_whitespace(input),
            ParseError::invalid_input(
                "class",
                Some("Attribute key must start with a period or a '#'".into()),
            ),
        );
    }
}
