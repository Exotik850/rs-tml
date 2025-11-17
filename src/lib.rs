mod attribute;
pub use attribute::Attribute;
mod document;
pub use document::Document;
mod element;
pub use element::{Element, Node, Text};
mod tag;
pub use tag::Tag;
mod error;
pub use error::{ParseError, ParseResult};
mod parse;
pub use parse::{RSTMLParse, RSTMLParseExt};
mod util;
#[cfg(test)]
pub(crate) use util::test_util;
pub(crate) use util::{nested, quote_nested};

#[cfg(test)]
mod tests {
    use crate::RSTMLParseExt;

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
        assert!(crate::Document::parse_ignoring_comments(input).is_ok());
    }
}
