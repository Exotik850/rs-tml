use crate::prelude::*;

/// Represents an entire RSTML document.
#[derive(Debug, Clone, PartialEq)]
pub struct Document<'a> {
    pub children: Vec<Node<'a>>,
}

impl Default for Document<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Document<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self { children: vec![] }
    }

    pub fn add_child(&mut self, child: impl Into<Node<'a>>) {
        self.children.push(child.into());
    }
    #[must_use]
    pub fn with_child(mut self, child: impl Into<Node<'a>>) -> Self {
        self.add_child(child);
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
}

impl<'a> RSTMLParse<'a> for Document<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self>
    where
        Self: Sized,
    {
        let (rest, children) = Node::parse_many_ignoring_comments(input);
        Ok((rest, Document { children }))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::test_util::*;

    #[test]
    fn test_document_parse() {
        let input = r#"
            // main content
            div {
                h1 { "Title" }
                p { "This is a paragraph." }
            }"#;
        assert_parse_eq(
            Document::parse_no_whitespace(input),
            Document::new().with_child(
                element("div")
                    .with_child(element("h1").with_child("Title"))
                    .with_child(element("p").with_child("This is a paragraph.")),
            ),
            "",
        );
    }
}
