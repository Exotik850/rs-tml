use crate::prelude::*;

/// Represents an entire RSTML document.
#[derive(Debug, Clone, PartialEq)]
pub struct Block<'a> {
    pub children: Vec<Node<'a>>,
}

impl Default for Block<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Block<'a> {
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

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    #[must_use]
    pub fn as_nodes(&self) -> &[Node<'a>] {
        &self.children
    }

    #[must_use]
    pub fn into_nodes(self) -> Vec<Node<'a>> {
        self.children
    }
}

impl<'a> RSTMLParse<'a> for Block<'a> {
    fn parse_no_whitespace(input: &'a str) -> ParseResult<'a, Self>
    where
        Self: Sized,
    {
        let (rest, children) = Node::parse_many_ignoring_comments(input);
        Ok((rest, Block { children }))
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
            Block::parse_no_whitespace(input),
            Block::new().with_child(
                element("div")
                    .with_child(element("h1").with_child("Title"))
                    .with_child(element("p").with_child("This is a paragraph.")),
            ),
            "",
        );
    }
}
