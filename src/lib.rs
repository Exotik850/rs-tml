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
                  comments should too */"#;
        assert!(dbg!(Block::parse_ignoring_comments(input)).is_ok());
    }
}
