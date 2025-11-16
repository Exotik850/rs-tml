use crate::{Attribute, ParseResult, RSTMLParse, RSTMLParseExt, Tag, parse::consume_comments};

// Represents plain text content within RSTML
//
// Text content is any sequence of characters that is surrounded by quotes
#[derive(Debug, PartialEq)]
pub struct Text<'a> {
    pub content: &'a str,
}

impl std::fmt::Display for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl<'a> RSTMLParse<'a> for Text<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        let (rest, content) = crate::quote_nested(input)?;
        Ok((rest, Text { content }))
    }
}

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Text(Text<'a>),
    Element(Element<'a>),
}

impl<'a> RSTMLParse<'a> for Node<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        if let Ok((rest, text)) = Text::parse_no_whitespace(input) {
            return Ok((rest, Node::Text(text)));
        }
        if let Ok((rest, element)) = Element::parse_no_whitespace(input) {
            return Ok((rest, Node::Element(element)));
        }
        Err(crate::ParseError::invalid_input(
            input,
            std::borrow::Cow::Borrowed("Cannot find Node type"),
        ))
    }
}

// Generic Element struct that can hold different types of children
#[derive(Debug, PartialEq)]
pub struct Element<'a> {
    pub name: Tag<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<Node<'a>>,
}

impl<'a> RSTMLParse<'a> for Element<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        let (rest, name) = Tag::parse_no_whitespace(input)?;
        let (rest_out, content) = crate::nested(rest, "{", Some("}"))?;
        let rest = consume_comments(content);

        let (mut rest, attributes) = Attribute::parse_many_ignoring_comments(rest);
        rest = consume_comments(rest);

        let (rest, children) = Node::parse_many_ignoring_comments(rest);
        if !consume_comments(rest).is_empty() {
            return Err(crate::ParseError::invalid_input(
                rest,
                Some("Unexpected content after element children".into()),
            ));
        }

        Ok((
            rest_out,
            Element {
                name,
                attributes,
                children,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tag;
    use crate::util::test_util::assert_parse_eq;

    #[test]
    fn test_text_parse() {
        let input = r#""Hello, World!""#;
        assert_parse_eq(
            Text::parse_no_whitespace(input),
            Text {
                content: "Hello, World!",
            },
            "",
        );
    }

    #[test]
    fn test_node_text_parse() {
        let input = r#""Sample Text""#;
        assert_parse_eq(
            Node::parse_no_whitespace(input),
            Node::Text(Text {
                content: "Sample Text",
            }),
            "",
        );
    }

    #[test]
    fn test_empty_element_parse() {
        let input = r#"div {
            // No attributes or children
        }"#;
        assert_parse_eq(
            Element::parse_no_whitespace(input),
            Element {
                name: Tag::DIV,
                attributes: vec![],
                children: vec![],
            },
            "",
        );
    }

    #[test]
    fn test_element_parse() {
        let input = r#"div { .class="container" "Hello" }"#;
        assert_parse_eq(
            Element::parse_no_whitespace(input),
            Element {
                name: Tag::DIV,
                attributes: vec![Attribute {
                    key: "class",
                    value: "container",
                }],
                children: vec![Node::Text(Text { content: "Hello" })],
            },
            "",
        );
    }

    #[test]
    fn test_element_with_comment_parse() {
        let input = r#"section {
            // This is a comment
            "Content" // inline comment
            /* Block comment */
        }"#;
        assert_parse_eq(
            Element::parse_no_whitespace(input),
            Element {
                name: Tag::SECTION,
                attributes: vec![],
                children: vec![Node::Text(Text { content: "Content" })],
            },
            "",
        );
    }

    #[test]
    fn test_element_with_no_attributes_parse() {
        let input = r#"span {
            "No attributes here"
        }"#;
        assert_parse_eq(
            Element::parse_no_whitespace(input),
            Element {
                name: Tag::SPAN,
                attributes: vec![],
                children: vec![Node::Text(Text {
                    content: "No attributes here",
                })],
            },
            "",
        );
    }

    #[test]
    fn test_nested_element_parse() {
        let input = r#"div {
            #main
            section {
                // nested element
                "Nested Content"
            }
        }"#;
        assert_parse_eq(
            Element::parse_no_whitespace(input),
            Element {
                name: Tag::DIV,
                attributes: vec![Attribute {
                    key: "id",
                    value: "main",
                }],
                children: vec![Node::Element(Element {
                    name: Tag::SECTION,
                    attributes: vec![],
                    children: vec![Node::Text(Text {
                        content: "Nested Content",
                    })],
                })],
            },
            "",
        );
    }
}
