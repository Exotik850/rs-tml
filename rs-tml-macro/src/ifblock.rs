use crate::RSTMLBlock;
use quote::ToTokens;
use syn::{Expr, Pat, Token, parse::Parse};

pub struct LetGaurd {
    let_token: Token![let],
    pattern: Box<Pat>,
    equal_token: Token![=],
    expr: Box<Expr>,
}

impl Parse for LetGaurd {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let let_token: Token![let] = input.parse()?;
        let pattern: Pat = Pat::parse_single(input)?;
        let equal_token: Token![=] = input.parse()?;
        let expr: Expr = Expr::parse_without_eager_brace(input)?;
        Ok(LetGaurd {
            let_token,
            pattern: Box::new(pattern),
            equal_token,
            expr: Box::new(expr),
        })
    }
}

impl ToTokens for LetGaurd {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let let_token = &self.let_token;
        let pattern = &self.pattern;
        let equal_token = &self.equal_token;
        let expr = &self.expr;
        tokens.extend(quote::quote! {
            #let_token #pattern #equal_token #expr
        });
    }
}

pub enum IfCond {
    Condition(Box<Expr>),
    Gaurd(LetGaurd),
}

impl Parse for IfCond {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![let]) {
            let expr_let = input.parse()?;
            Ok(IfCond::Gaurd(expr_let))
        } else {
            let expr: Expr = Expr::parse_without_eager_brace(input)?;
            Ok(IfCond::Condition(Box::new(expr)))
        }
    }
}

impl ToTokens for IfCond {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            IfCond::Condition(expr) => {
                expr.to_tokens(tokens);
            }
            IfCond::Gaurd(expr_let) => {
                expr_let.to_tokens(tokens);
            }
        }
    }
}

pub struct RSTMLIf {
    if_token: Token![if],
    condition: IfCond,
    then: RSTMLBlock,
    else_if: Option<Box<RSTMLIf>>,
    else_block: Option<(Token![else], RSTMLBlock)>,
}

impl RSTMLIf {
    fn build_chain(&self, out: &mut proc_macro2::TokenStream) {
        let RSTMLIf {
            if_token,
            condition,
            then,
            else_if,
            else_block,
            ..
        } = self;

        // Generate an expression that evaluates to Option<Node> so we can
        // conditionally add a child without requiring an else branch from the user.
        out.extend(quote::quote! {
            #if_token #condition {
                Some(Node::from(#then))
            }
        });

        if let Some(next_if) = else_if.as_deref() {
            out.extend(quote::quote! { else });
            next_if.build_chain(out);
        } else if let Some((else_token, else_blk)) = else_block {
            // If an explicit else block exists, return Some(node) for it.
            out.extend(quote::quote! {
                 #else_token {
                    Some(Node::from(#else_blk))
                }
            });
        } else {
            // No else provided by the user; default to None so the Option compiles.
            out.extend(quote::quote! {
                 else { None }
            });
        }
    }
}

impl Parse for RSTMLIf {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let if_token: Token![if] = input.parse()?;
        let condition = input.parse()?;
        // Parse the then-block as an RSTMLBlock, letting it handle its own braces
        let then: RSTMLBlock = input.parse()?;

        let mut else_if = None;
        let mut else_block = None;

        if input.peek(Token![else]) {
            let else_token: Token![else] = input.parse()?;
            if input.peek(Token![if]) {
                let next_if: RSTMLIf = input.parse()?;
                else_if = Some(Box::new(next_if));
            } else {
                let else_blk: RSTMLBlock = input.parse()?;
                else_block = Some((else_token, else_blk));
            }
        }

        Ok(RSTMLIf {
            if_token,
            condition,
            then,
            else_if,
            else_block,
        })
    }
}

impl quote::ToTokens for RSTMLIf {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.build_chain(tokens);
    }
}
