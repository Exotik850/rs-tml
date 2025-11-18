use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Expr, Ident, LitStr, Token, parse::Parse, token::Paren};

mod attribute;
use attribute::Attribute;
mod element;
use element::Element;

use crate::{forblock::RSTMLFor, ifblock::RSTMLIf, matchblock::RSTMLMatch};
mod forblock;
mod ifblock;
mod matchblock;

struct Document {
    children: Vec<Node>,
}

impl Parse for Document {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut children = Vec::new();
        while !input.is_empty() {
            let node: Node = input.parse()?;
            children.push(node);
        }
        Ok(Document { children })
    }
}

impl quote::ToTokens for Document {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote::quote! {
            ::rs_tml::block::Block::new()
        });
        for child in &self.children {
            match child {
                Node::Element(element) => {
                    tokens.extend(quote::quote! {
                        .with_child({#element}.into_node())
                    });
                }
                other => {
                    tokens.extend(quote::quote! {
                        #other
                    });
                }
            }
        }
    }
}

// Block of nodes delimited by braces
struct RSTMLBlock {
    children: Vec<Node>,
}

impl Parse for RSTMLBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::braced!(content in input);
        let mut children = Vec::new();
        while !content.is_empty() {
            let node: Node = content.parse()?;
            children.push(node);
        }
        Ok(RSTMLBlock { children })
    }
}

impl quote::ToTokens for RSTMLBlock {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for child in &self.children {
            child.to_tokens(tokens);
        }
    }
}

enum TextNode {
    Literal(LitStr),
    Dynamic(LitStr), // Contains format! style placeholders
}

fn is_fmt_string(input: &str) -> bool {
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            if let Some(&next_c) = chars.peek() {
                if next_c == '{' {
                    chars.next(); // Skip escaped '{'
                    continue;
                }
                return true; // Found unescaped '{'
            }
            return true; // Found unescaped '{' at end
        } else if c == '}' {
            if let Some(&next_c) = chars.peek() {
                if next_c == '}' {
                    chars.next(); // Skip escaped '}'
                    continue;
                }
                return true; // Found unescaped '}'
            }
            return true; // Found unescaped '}' at end
        }
    }
    false
}

impl Parse for TextNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit: LitStr = input.parse()?;

        // if lit contains unescaped '{' or '}', treat as Dynamic
        if is_fmt_string(&lit.value()) {
            return Ok(TextNode::Dynamic(lit));
        }
        Ok(TextNode::Literal(lit))
    }
}

impl quote::ToTokens for TextNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TextNode::Literal(lit) => {
                tokens.extend(quote::quote! {
                    ::rs_tml::node::Node::text(#lit)
                });
            }
            TextNode::Dynamic(lit) => {
                tokens.extend(quote::quote! {
                    ::rs_tml::node::Node::text(format!(#lit))
                });
            }
        }
    }
}

enum Node {
    Text(TextNode),
    Element(Element),
    If(RSTMLIf),
    For(RSTMLFor),
    Match(RSTMLMatch),
    Expand(Expr),
}

impl Parse for Node {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(if_block) = input.parse::<RSTMLIf>() {
            return Ok(Node::If(if_block));
        }
        if let Ok(for_block) = input.parse::<RSTMLFor>() {
            return Ok(Node::For(for_block));
        }
        if let Ok(match_block) = input.parse::<RSTMLMatch>() {
            return Ok(Node::Match(match_block));
        }
        if let Ok(text) = input.parse() {
            return Ok(Node::Text(text));
        }
        if let Ok(element) = input.parse::<Element>() {
            return Ok(Node::Element(element));
        }
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            if !input.peek(Paren) {
                let ident: Ident = input.parse()?;
                return Ok(Node::Expand(Expr::Verbatim(ident.into_token_stream())));
            }
            let content;
            syn::parenthesized!(content in input);
            let expr = content.parse()?;
            return Ok(Node::Expand(expr));
        }
        Err(input.error("Expected a valid RSTML node"))
    }
}

impl quote::ToTokens for Node {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Node::Text(lit) => tokens.extend(quote::quote! {
                #lit
            }),
            Node::Element(element) => {
                element.to_tokens(tokens);
            }
            Node::If(if_block) => {
                if_block.to_tokens(tokens);
            }
            Node::For(for_block) => {
                for_block.to_tokens(tokens);
            }
            Node::Match(match_block) => {
                match_block.to_tokens(tokens);
            }
            Node::Expand(expr) => tokens.extend(quote::quote! {
                ::rs_tml::node::Node::from(#expr)
            }),
        }
    }
}

/// A procedural macro that returns an RSTML document
#[proc_macro]
pub fn rstml(input: TokenStream) -> TokenStream {
    let document = syn::parse_macro_input!(input as Document);
    document.into_token_stream().into()
}

// // these all expand to valid code
// // attributes
// .attr = if expr { // match as well
//    "something"
//  } [else { "something else" }]

// // variable for attr name
// .*name = "Tony"
// .*(expr) = "Runtime name {expr}" // format!
// // expand (T: Display, U: Display) iterator
// ..*attrs
// ..*(expr)

// // match statement
// match expr {
//    Some(field) [if expr] => { // [expr] means optional
//       p { "{field}" }
//    }
//    None => p { "nothing" }
// }

// // conditional
// if condition {
//    h1 { "True!" }
// } [else if other_condition {
//    h1 { "Else if!" }
// }] [else {
//    h1 { "False!" }
// }]

// // iterators
// for (i, name) in names.iter().enumerate() {
//    p { "{i}: {name}" }
// }

// // expand another call
// *child
