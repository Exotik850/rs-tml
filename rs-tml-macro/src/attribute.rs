use syn::{Expr, Ident, LitStr, Token, parse::Parse};

pub enum AttributeKey {
    Static(String),
    Dynamic(Expr),
    StaticId(String),
    DynamicId(Expr),
}

fn parse_hyphenated_ident(input: syn::parse::ParseStream) -> syn::Result<String> {
    let first: Ident = input.parse()?;
    let mut out = first.to_string();
    // Consume sequences of -ident to allow hyphenated names like data-id or class-name
    while input.peek(Token![-]) && input.peek2(Ident) {
        let _dash: Token![-] = input.parse()?;
        let next: Ident = input.parse()?;
        out.push('-');
        out.push_str(&next.to_string());
    }
    Ok(out)
}

impl Parse for AttributeKey {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if !(input.peek(Token![.]) || input.peek(Token![#])) {
            return Err(input.error("Expected '.' or '#' at the start of an attribute"));
        }
        let is_class = input.parse::<Token![.]>().is_ok();
        if !is_class {
            input.parse::<Token![#]>()?;
        }
        let key = if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            // Support both .*(expr) and .*{expr}
            let expr = if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                Expr::parse_without_eager_brace(&content)?
            } else if input.peek(syn::token::Brace) {
                let content;
                syn::braced!(content in input);
                Expr::parse_without_eager_brace(&content)?
            } else {
                return Err(input.error("Expected '(' or '{' after '.*' attribute shorthand"));
            };
            if is_class {
                AttributeKey::Dynamic(expr)
            } else {
                AttributeKey::DynamicId(expr)
            }
        } else {
            let name = parse_hyphenated_ident(input)?;
            if is_class {
                AttributeKey::Static(name)
            } else {
                AttributeKey::StaticId(name)
            }
        };
        Ok(key)
    }
}

pub enum AttributeValue {
    Static(LitStr),
    Dynamic(Expr),
}

impl Parse for AttributeValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            Ok(AttributeValue::Static(lit))
        } else {
            let expr = Expr::parse_without_eager_brace(input)?;
            Ok(AttributeValue::Dynamic(expr))
        }
    }
}

/// Attribute for RSTML Elements
///
/// Examples:
/// .title = "Hello World"          // `KeyValue` with static key and static value
/// .data-id = `some_variable`        // `KeyValue` with static key and dynamic value
/// .*`dynamic_key` = "Static Value"  // `KeyValue` with dynamic key and static value
/// .*`dynamic_key` = `dynamic_value`   // `KeyValue` with dynamic key and dynamic value
/// .*(expr) = expr                 // `KeyValue` with dynamic key and dynamic value
/// #id                             // `KeyOnly` with static key (id shorthand)
/// #*(expr)                        // `KeyOnly` with dynamic key (id shorthand)
/// .disabled                       // `KeyOnly` with static key (class shorthand)
/// .*`dynamic_key`                   // `KeyOnly` with dynamic key (class shorthand)
/// ..*attrs                        // `KeySpread` with dynamic key
pub enum Attribute {
    KeyValue {
        key: AttributeKey,
        value: AttributeValue,
    },
    KeyOnly {
        key: AttributeKey,
    },
    KeySpread {
        key: Expr,
    },
}

impl Parse for Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // if there are two consecutive dots, it's a key spread
        if input.peek(Token![.]) && input.peek2(Token![.]) {
            input.parse::<Token![.]>()?;
            input.parse::<Token![.]>()?;
            let key = Expr::parse_without_eager_brace(input)?;
            return Ok(Attribute::KeySpread { key });
        }

        let key = input.parse()?;
        if !input.peek(Token![=]) {
            return Ok(Attribute::KeyOnly { key });
        }
        if matches!(key, AttributeKey::StaticId(_) | AttributeKey::DynamicId(_)) {
            return Err(input.error("ID shorthand cannot be used with key-value attributes"));
        }
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Attribute::KeyValue { key, value })
    }
}

impl quote::ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Attribute::KeyValue { key, value } => {
                let key_tokens = match key {
                    AttributeKey::Static(name) => quote::quote! { #name },
                    AttributeKey::Dynamic(expr) => quote::quote! { (#expr) },
                    _ => unreachable!("ID shorthand cannot be used with key-value attributes"),
                };
                let value_tokens = match value {
                    AttributeValue::Static(lit) => quote::quote! { #lit },
                    AttributeValue::Dynamic(expr) => quote::quote! { #expr },
                };
                tokens.extend(quote::quote! {
                    .with_key_value(#key_tokens, #value_tokens)
                });
            }
            Attribute::KeyOnly { key } => {
                // static => with_key_value("class", key)
                // dynamic => with_key_value("class", (key).to_string())
                // static id => with_key_value("id", key)
                // dynamic id => with_key_value("id", (key).to_string())
                match key {
                    AttributeKey::Static(name) => {
                        tokens.extend(quote::quote! {
                            .with_key_value("class", #name)
                        });
                    }
                    AttributeKey::Dynamic(expr) => {
                        tokens.extend(quote::quote! {
                            .with_key_value("class", (#expr))
                        });
                    }
                    AttributeKey::StaticId(name) => {
                        tokens.extend(quote::quote! {
                            .with_key_value("id", #name)
                        });
                    }
                    AttributeKey::DynamicId(expr) => {
                        tokens.extend(quote::quote! {
                            .with_key_value("id", (#expr))
                        });
                    }
                }
            }
            Attribute::KeySpread { key } => {
                tokens.extend(quote::quote! {
                  .with_key_values({#key}.into_iter())
                });
            }
        }
    }
}
