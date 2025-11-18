use quote::ToTokens;
use syn::Ident;

use crate::{Attribute, Node};

pub struct Element {
    name: Ident,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}

impl syn::parse::Parse for Element {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        syn::braced!(content in input);
        let mut attributes = Vec::new();
        while let Ok(attr) = content.parse() {
            attributes.push(attr);
        }
        let mut children = Vec::new();
        while let Ok(child) = content.parse() {
            children.push(child);
        }
        Ok(Element {
            name,
            attributes,
            children,
        })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let attrs = self.attributes.iter().map(Attribute::to_child_tokens);
        let children = self.children.iter().map(Node::to_child_tokens);
        tokens.extend(quote::quote! {
            ::rs_tml::element::Element::new(stringify!(#name))
            #(#attrs)*
            #(#children)*
        });
    }
}
