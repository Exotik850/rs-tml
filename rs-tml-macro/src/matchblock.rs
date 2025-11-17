use syn::{Expr, Pat, Token, parse::Parse, token::Brace};

use crate::Node;

pub struct RSTMLMatchArm {
    pattern: Box<Pat>,
    guard: Option<(Token![if], Box<Expr>)>,
    body: Vec<Node>,
}

impl Parse for RSTMLMatchArm {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // let pattern = Expr::parse_without_eager_brace(input)?;
        let pattern: Pat = Pat::parse_multi(input)?;
        let guard = if input.peek(syn::Token![if]) {
            let if_token = input.parse::<syn::Token![if]>()?;
            let expr = Expr::parse_without_eager_brace(input)?;
            Some((if_token, Box::new(expr)))
        } else {
            None
        };
        input.parse::<syn::Token![=>]>()?;

        let mut body = Vec::new();
        if input.peek(Brace) {
            let content;
            syn::braced!(content in input);
            while !content.is_empty() {
                let node: Node = content.parse()?;
                // Allow optional commas between nodes inside the arm block
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
                body.push(node);
            }
        } else {
            let node: Node = input.parse()?;
            body.push(node);
        }

        Ok(RSTMLMatchArm {
            pattern: Box::new(pattern),
            guard,
            body,
        })
    }
}

pub struct RSTMLMatch {
    match_token: Token![match],
    expression: Box<Expr>,
    arms: Vec<RSTMLMatchArm>,
}

impl Parse for RSTMLMatch {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let match_token = input.parse::<syn::Token![match]>()?;
        let expression: Expr = Expr::parse_without_eager_brace(input)?;
        let content;
        syn::braced!(content in input);
        let mut arms = Vec::new();
        while !content.is_empty() {
            arms.push(content.parse()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        Ok(RSTMLMatch {
            match_token,
            expression: Box::new(expression),
            arms,
        })
    }
}

impl quote::ToTokens for RSTMLMatch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let match_token = &self.match_token;
        let expr = &self.expression;
        let arms = self.arms.iter().map(|arm| {
            let pattern = &arm.pattern;
            let guard = arm.guard.as_ref().map(|(i, g)| quote::quote! { #i #g });
            let body = arm.body.iter();
            quote::quote! {
                #pattern #guard => {
                    #(#body)*
                }
            }
        });
        tokens.extend(quote::quote! {
            .with_child(#match_token #expr {
                #(#arms),*
            })
        });
    }
}
