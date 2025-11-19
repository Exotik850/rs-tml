pub mod error;
mod models;
pub use models::*;
pub mod parse;
mod util;
#[cfg(test)]
pub(crate) use util::test_util;

pub mod prelude {
    use super::{error, models, parse};
    pub use error::{ParseError, ParseResult};
    pub use models::prelude::*;
    pub use parse::{RSTMLParse, RSTMLParseExt};
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    #[test]
    fn test_large_document() {
        let input = r#"div {
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
            }"#;
        let block = Block::parse_no_whitespace(input)
            .expect("Block should parse correctly")
            .1;

        // The input is a single div element, so block should have 1 child (the div)
        assert_eq!(block.children.len(), 1);

        // Get the div element
        if let Some(Node::Element(div)) = block.children.first() {
            // The div should have 8 children: style, h1, h2, a, br (5 elements)
            // Plus attributes: #main, .class, .lg (3 attributes)
            // So total children elements should be 5
            assert_eq!(div.children.len(), 5);
            assert_eq!(div.attributes.len(), 3);
        } else {
            panic!("Expected first child to be an element");
        }

        println!("{block}")
    }
}
