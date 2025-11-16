use pastey::paste;

use crate::{ParseResult, RSTMLParse};

// Represents an RSTML tag
//
// RSTML tags are structured like 'lower-camel-case' strings.
// They can contain alphanumeric characters and hyphens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag<'a> {
    pub(crate) name: &'a str,
}

macro_rules! tag {
    ($($name:ident)+) => {
        $(
          paste! {
            pub const [<$name:upper>]: Tag<'a> = Tag::new(stringify!([<$name:lower>]));
          }
        )+
    };
}

impl<'a> From<&'a str> for Tag<'a> {
    fn from(name: &'a str) -> Self {
        Tag::new(name)
    }
}

impl<'a> From<Tag<'a>> for &'a str {
    fn from(tag: Tag<'a>) -> Self {
        tag.as_str()
    }
}

impl std::fmt::Display for Tag<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> Tag<'a> {
    pub(crate) const fn new(name: &'a str) -> Self {
        Tag { name }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'a str {
        self.name
    }

    tag!(div span p a img ul li table tr td th header footer nav section article main aside form input button label select option textarea style);
}

fn split_exclusive_once(input: &str, predicate: impl Fn(char) -> bool) -> Option<(&str, &str)> {
    for (idx, ch) in input.char_indices() {
        if predicate(ch) {
            return input.split_at(idx).into();
        }
    }
    None
}

impl<'a> RSTMLParse<'a> for Tag<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        let (name, rest) = split_exclusive_once(input, |c| !(c.is_alphanumeric() || c == '-'))
            .unwrap_or((input, ""));
        if name.is_empty() {
            return Err(crate::ParseError::EmptyInput);
        }
        Ok((rest, Tag::new(name)))
    }
}

#[cfg(test)]
mod tests {
    use super::Tag;
    use crate::test_util::*;
    use crate::{ParseError, RSTMLParse};

    #[test]
    fn test_tag_parse() {
        let input = "div.class#id{content}";
        assert_parse_eq(
            Tag::parse_no_whitespace(input),
            Tag::new("div"),
            ".class#id{content}",
        );
    }

    #[test]
    fn test_empty_tag_parse() {
        let input = ".class#id{content}";
        assert_parse_err(Tag::parse_no_whitespace(input), ParseError::EmptyInput);
    }

    #[test]
    fn test_tag_with_hyphen_parse() {
        let input = "custom-tag.class#id{content}";
        assert_parse_eq(
            Tag::parse_no_whitespace(input),
            Tag::new("custom-tag"),
            ".class#id{content}",
        );
    }
}
