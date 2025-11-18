pub mod attribute;
pub mod block;
pub mod element;
pub mod error;
pub mod parse;
pub mod tag;
pub mod text;
mod util;
#[cfg(test)]
pub(crate) use util::test_util;
pub(crate) use util::{nested, quote_nested};

pub mod prelude {
    use super::{attribute, block, element, error, parse, tag, text};
    pub use attribute::Attribute;
    pub use block::Block;
    pub use element::{Element, Node, element};
    pub use error::{ParseError, ParseResult};
    pub use parse::{RSTMLParse, RSTMLParseExt};
    pub use tag::Tag;
    pub use text::Text;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    #[test]
    fn test_large_document() {
        let input = r#"
            div {
               // specific styles can be added like so
               // only one style block per element
               style {
                  .padding = "1rem"
                  .background-color = "blue"
               }

               #main // id shorthand

               .class = "bg-blue"
               .lg // class shorthand

               h1 { "Something" }
               h2 {
              	.class = "bg-red"
                    "Something red"
               }

               a { .href = "https://google.com" "Links should work" }

               br {} // self closing / empty tags

               // comments should work
               /* multi
                  line
                  comments should too */
                  "#;
        assert!(dbg!(Block::parse_ignoring_comments(input)).is_ok());
    }
}
