use crate::{Attribute, ParseResult, RSTMLParse, RSTMLParseExt, Tag, nested};

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

// Represents a comment within RSTML
//
// Comments can be one-line or multi-line.
//
// One-line comments start with '//' and continue to the end of the line.
// Multi-line comments are enclosed within '/*' and '*/'.
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

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Text(Text<'a>),
    Comment(Comment<'a>),
    Element(Element<'a>),
}

impl<'a> RSTMLParse<'a> for Node<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        if let Ok((rest, text)) = Text::parse_no_whitespace(input) {
            return Ok((rest, Node::Text(text)));
        }
        if let Ok((rest, comment)) = Comment::parse_no_whitespace(input) {
            return Ok((rest, Node::Comment(comment)));
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
        let rest = content.trim_start();

        let (mut rest, attributes) = Attribute::parse_many(rest)?;
        rest = rest.trim_start();

        let (rest, children) = Node::parse_many(rest)?;
        if !rest.trim().is_empty() {
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
        let input = r#"div { }"#;
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
            "Content"
        }"#;
        assert_parse_eq(
            Element::parse_no_whitespace(input),
            Element {
                name: Tag::SECTION,
                attributes: vec![],
                children: vec![
                    Node::Comment(Comment::Line(" This is a comment")),
                    Node::Text(Text { content: "Content" }),
                ],
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
            .id="main"
            section {
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
