use std::borrow::Cow;

use crate::prelude::*;

/// Generic Node enum that can represent either a Text, Element, or Block node.
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
    #[must_use]
    pub fn text(value: impl Into<Cow<'a, str>>) -> Self {
        Self::text_const(value.into())
    }

    #[must_use]
    pub fn element(element: impl Into<Element<'a>>) -> Self {
        Self::element_const(element.into())
    }
    #[must_use]
    pub const fn element_const(element: Element<'a>) -> Self {
        Node::Element(element)
    }

    /// Check if the node is empty,
    /// i.e., if it is a Text node with empty content,
    /// an Element node with no attributes and no children,
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Node::Text(text) => text.content.is_empty(),
            Node::Element(element) => element.is_empty(),
        }
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
        if let Ok((rest, text)) = Text::parse_ignoring_comments(input) {
            return Ok((rest, Node::Text(text)));
        }
        if let Ok((rest, element)) = Element::parse_ignoring_comments(input) {
            return Ok((rest, Node::Element(element)));
        }
        Err(ParseError::invalid_input(
            input,
            Some("Expected a Text or Element node".into()),
        ))
    }
}
