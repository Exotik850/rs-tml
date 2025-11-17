use syn::{Expr, Pat, parse::Parse};

use crate::RSTMLBlock;

pub struct RSTMLFor {
    pattern: Box<Pat>,
    iterable: Box<Expr>,
    body: RSTMLBlock,
}

impl Parse for RSTMLFor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![for]>()?;
        let pattern = Pat::parse_single(input)?;
        input.parse::<syn::Token![in]>()?;
        let iterable = Expr::parse_without_eager_brace(input)?;
        let body: RSTMLBlock = input.parse()?;
        Ok(RSTMLFor {
            pattern: Box::new(pattern),
            iterable: Box::new(iterable),
            body,
        })
    }
}

impl quote::ToTokens for RSTMLFor {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let pattern = &self.pattern;
        let iterable = &self.iterable;
        let body = &self.body;
        tokens.extend(quote::quote! {
          .with_children((#iterable).into_iter().map(|#pattern| {
              #body
          }))
        });
    }
}
