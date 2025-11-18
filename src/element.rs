use std::borrow::Cow;

use crate::{parse::consume_comments, prelude::*};

/// Generic Node enum that can represent either Text or Element
#[derive(PartialEq, Clone)]
pub enum Node<'a> {
    Text(Text<'a>),
    Element(Element<'a>),
}

impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Text(text) => write!(f, "{text:?}"),
            Node::Element(element) => write!(f, "{element:?}"),
        }
    }
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
    pub const fn text_const(value: Cow<'a, str>) -> Self {
        Node::Text(Text::new_const(value))
    }
    pub fn text(value: impl Into<Cow<'a, str>>) -> Self {
        Self::text_const(value.into())
    }

    pub fn element(element: impl Into<Element<'a>>) -> Self {
        Self::element_const(element.into())
    }
    #[must_use]
    pub const fn element_const(element: Element<'a>) -> Self {
        Node::Element(element)
    }

    #[must_use]
    pub fn into_node(self) -> Self {
        self
    }
}

impl From<String> for Node<'_> {
    fn from(value: String) -> Self {
        Node::Text(Text::new(value))
    }
}

impl<'a> From<&'a str> for Node<'a> {
    fn from(value: &'a str) -> Self {
        Node::Text(Text::new(value))
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
        Err(ParseError::invalid_input(
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

    pub fn add_children<I>(&mut self, children: I)
    where
        I: IntoIterator<Item: Into<Node<'a>>>,
    {
        for child in children {
            self.add_child(child);
        }
    }
    #[must_use]
    pub fn with_children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item: Into<Node<'a>>>,
    {
        self.add_children(children);
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
    pub fn add_attributes<I>(&mut self, attributes: I)
    where
        I: IntoIterator<Item = Attribute<'a>>,
    {
        for attribute in attributes {
            self.add_attribute(attribute);
        }
    }
    #[must_use]
    pub fn with_attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = Attribute<'a>>,
    {
        self.add_attributes(attributes);
        self
    }

    pub fn add_key_value(&mut self, key: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) {
        self.add_attribute(Attribute::new(key, value));
    }
    #[must_use]
    pub fn with_key_value(
        mut self,
        key: impl Into<Cow<'a, str>>,
        value: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.add_key_value(key, value);
        self
    }

    pub fn add_key_values<I, K, V>(&mut self, key_values: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        for (key, value) in key_values {
            self.add_key_value(key, value);
        }
    }
    #[must_use]
    pub fn with_key_values<I, K, V>(mut self, key_values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.add_key_values(key_values);
        self
    }

    #[must_use]
    pub fn into_node(self) -> Node<'a> {
        Node::Element(self)
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
            return Err(ParseError::invalid_input(
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
    use crate::prelude::*;
    use crate::util::test_util::assert_parse_eq;

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
