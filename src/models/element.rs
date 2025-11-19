use std::borrow::Cow;

use crate::{parse::consume_comments, prelude::*};

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

    /// Adds a child node to the element.
    ///
    /// If the child is a Block, its children are flattened into the element's children.
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
        let (rest_out, content) = crate::util::nested(rest, "{", "}")?;
        let mut rest = content;

        let mut attributes = Vec::new();
        let mut children = Vec::new();

        // Parse attributes and children in an interleaved manner
        while !rest.is_empty() {
            rest = consume_comments(rest);
            if rest.is_empty() {
                break;
            }

            // Try to parse an attribute first
            if let Ok((new_rest, attr)) = Attribute::parse_ignoring_comments(rest) {
                attributes.push(attr);
                rest = new_rest;
                continue;
            }

            // If attribute parsing fails, try to parse a node (child)
            if let Ok((new_rest, node)) = Node::parse_ignoring_comments(rest) {
                children.push(node);
                rest = new_rest;
                continue;
            }

            // If neither works, we have unexpected content
            return Err(ParseError::invalid_input(
                rest,
                Some("Unexpected content in element".into()),
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
