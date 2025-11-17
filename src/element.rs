use crate::{Attribute, ParseResult, RSTMLParse, RSTMLParseExt, Tag, parse::consume_comments};

// Represents plain text content within RSTML
//
// Text content is any sequence of characters that is surrounded by quotes
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Text<'a> {
    pub content: &'a str,
}

impl<'a> Text<'a> {
    #[must_use]
    pub const fn new_const(content: &'a str) -> Self {
        Text { content }
    }
    pub fn new(content: impl Into<&'a str>) -> Self {
        Self::new_const(content.into())
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(value: &'a str) -> Self {
        Text::new_const(value)
    }
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

/// Generic Node enum that can represent either Text or Element
#[derive(Debug, PartialEq, Clone)]
pub enum Node<'a> {
    Text(Text<'a>),
    Element(Element<'a>),
}

impl<'a> Node<'a> {
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }

    #[must_use]
    pub const fn is_element(&self) -> bool {
        matches!(self, Node::Element(_))
    }

    #[must_use]
    pub const fn text_const(value: &'a str) -> Self {
        Node::Text(Text::new_const(value))
    }
    pub fn text(value: impl Into<&'a str>) -> Self {
        Self::text_const(value.into())
    }

    pub fn element(element: impl Into<Element<'a>>) -> Self {
        Self::element_const(element.into())
    }
    #[must_use]
    pub const fn element_const(element: Element<'a>) -> Self {
        Node::Element(element)
    }
}

impl<'a> From<&'a str> for Node<'a> {
    fn from(value: &'a str) -> Self {
        Node::Text(Text::new_const(value))
    }
}

impl<'a> From<Text<'a>> for Node<'a> {
    fn from(value: Text<'a>) -> Self {
        Node::Text(value)
    }
}

impl<'a> From<Element<'a>> for Node<'a> {
    fn from(value: Element<'a>) -> Self {
        Node::Element(value)
    }
}

impl<'a> RSTMLParse<'a> for Node<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        if let Ok((rest, element)) = Element::parse_no_whitespace(input) {
            return Ok((rest, Node::Element(element)));
        }
        if let Ok((rest, text)) = Text::parse_no_whitespace(input) {
            return Ok((rest, Node::Text(text)));
        }
        Err(crate::ParseError::invalid_input(
            input,
            std::borrow::Cow::Borrowed("Cannot find Node type"),
        ))
    }
}

// Generic Element struct that can hold different types of children
#[derive(Debug, PartialEq, Clone)]
pub struct Element<'a> {
    pub name: Tag<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<Node<'a>>,
}

impl<'a> Element<'a> {
    pub const EMPTY: Self = Self::empty();
    #[must_use]
    pub const fn empty() -> Element<'a> {
        Element {
            name: Tag::DIV,
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.attributes.is_empty() && self.children.is_empty()
    }
    #[must_use]
    pub const fn new_const(name: Tag<'a>) -> Self {
        Element {
            name,
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }
    pub fn new(name: impl Into<Tag<'a>>) -> Self {
        Self::new_const(name.into())
    }

    pub fn add_child(&mut self, child: impl Into<Node<'a>>) {
        self.children.push(child.into());
    }
    #[must_use]
    pub fn with_child(mut self, child: impl Into<Node<'a>>) -> Self {
        self.add_child(child.into());
        self
    }

    pub fn add_attribute(&mut self, attribute: Attribute<'a>) {
        self.attributes.push(attribute);
    }
    #[must_use]
    pub fn with_attribute(mut self, attribute: Attribute<'a>) -> Self {
        self.add_attribute(attribute);
        self
    }

    pub fn add_key_value(&mut self, key: impl Into<&'a str>, value: impl Into<&'a str>) {
        self.add_attribute(Attribute::new(key, value));
    }
    #[must_use]
    pub fn with_key_value(mut self, key: impl Into<&'a str>, value: impl Into<&'a str>) -> Self {
        self.add_key_value(key, value);
        self
    }
}

impl<'a> RSTMLParse<'a> for Element<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self> {
        let (rest, name) = Tag::parse_no_whitespace(input)?;
        let rest = consume_comments(rest);
        let (rest_out, content) = crate::nested(rest, "{", "}")?;
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

pub fn element<'a>(name: impl Into<Tag<'a>>) -> Element<'a> {
    Element::new(name)
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
            Text::new("Hello, World!"),
            "",
        );
    }

    #[test]
    fn test_node_text_parse() {
        let input = r#""Sample Text""#;
        assert_parse_eq(
            Node::parse_no_whitespace(input),
            Node::text("Sample Text"),
            "",
        );
    }

    #[test]
    fn test_empty_element_parse() {
        let input = r#"div {
            // No attributes or children
        }"#;
        assert_parse_eq(Element::parse_no_whitespace(input), Element::EMPTY, "");
    }

    #[test]
    fn test_element_parse() {
        let input = r#"div { .class="container" "Hello" }"#;
        assert_parse_eq(
            Element::parse_no_whitespace(input),
            element(Tag::DIV)
                .with_key_value("class", "container")
                .with_child("Hello"),
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
            element(Tag::SECTION).with_child("Content"),
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
            element(Tag::SPAN).with_child("No attributes here"),
            "",
        );
    }

    #[test]
    fn test_nested_element_parse() {
        let input = r#"div
        // Main container
        {
            #main
            section {
                // nested element
                "Nested Content"
            }
        }"#;
        assert_parse_eq(
            Element::parse_no_whitespace(input),
            element(Tag::DIV)
                .with_key_value("id", "main")
                .with_child(element(Tag::SECTION).with_child("Nested Content")),
            "",
        );
    }
}
