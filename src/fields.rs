extern crate alloc;
use alloc::collections::BTreeSet as HashSet;
use alloc::string::ToString;


use proc_macro2::{Delimiter, Punct, Span};
use proc_macro2::Spacing::Alone;
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{Error, Ident, LitInt, token, Type, Token};
use syn::parse::discouraged::AnyDelimiter;
use syn::parse::{ParseStream, Parse};
use syn::spanned::Spanned;
use syn::token::{Comma, Impl};

use crate::{consume_delimited, CONFIG_PROP_ERR_MSG};

static FIELD_PROPS: &str = "\"cloned\", \"default\", \"expr\", \"into\", \"iter\"";

/// Represents a configuration on a struct field
///
/// # Example
///
/// ```
/// use derive_ctor::ctor;
///
/// #[derive(ctor)]
/// struct Example {
///     #[ctor(default = [1, 2])]
///    field: i16
/// }
/// ```

#[derive(Clone)]
pub(crate) struct FieldConfig {
    pub(crate) property: FieldConfigProperty,
    pub(crate) applications: HashSet<usize>
}

#[derive(Clone)]
pub(crate) enum FieldConfigProperty {
    Cloned,
    Default,
    Into,
    Iter { iter_type: Type },
    Expression { expression: proc_macro2::TokenStream, input_type: Option<Type>, self_referencing: bool }
}

#[derive(Clone)]
pub(crate) struct ParameterField {
    pub(crate) field_ident: Ident,
    pub(crate) field_type: Type,
    pub(crate) span: Span
}

#[derive(Clone)]
pub(crate) struct GeneratedField {
    pub(crate) field_ident: Ident,
    pub(crate) configuration: FieldConfigProperty,
    #[allow(dead_code /*may be used for future purposes*/)]
    pub(crate) span: Span
}

impl Parse for FieldConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = FieldConfig {
            property: input.parse()?,
            applications: HashSet::default()
        };

        if input.parse::<token::Eq>().is_err() {
            return Ok(config)
        }

        // consume constructor specifier ex: 1, 2, 3
        if let Ok((delim, span, buffer)) = input.parse_any_delimiter() {
            if delim != Delimiter::Bracket {
                return Err(Error::new(span.span(), "Expected enclosing brackets"))
            }
            loop {
                config.applications.insert(buffer.parse::<LitInt>()?.base10_parse()?);
                if buffer.parse::<Comma>().is_err() {
                    break;
                }
            }
        } else {
            config.applications.insert(input.parse::<LitInt>()?.base10_parse()?);
        }

        Ok(config)
    }
}

impl Parse for FieldConfigProperty {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(token) = input.parse::<Impl>() {
            return Err(Error::new(
                token.span(),
                "\"impl\" property has been renamed to \"into\"."
            ));
        }

        let property: Ident = input.parse()?;
        let property_name = property.to_string();
        match property_name.as_str() {
            "cloned" => Ok(FieldConfigProperty::Cloned),
            "default" => Ok(FieldConfigProperty::Default),
            "into" => Ok(FieldConfigProperty::Into),
            "iter" => consume_delimited(input, Delimiter::Parenthesis, |buffer| {
                Ok(FieldConfigProperty::Iter { iter_type: buffer.parse()? })
            }),
            "expr" => {
                let self_referencing = input.parse::<Token![!]>().is_ok();

                consume_delimited(input, Delimiter::Parenthesis, |buffer| {
                    let mut input_type = None;

                    // determine the input_type by looking for the expression: expr(TYPE -> EXPRESSION)
                    if buffer.peek2(Token![->]) {
                        input_type = Some(buffer.parse()?);
                        buffer.parse::<Token![->]>()?;
                    }

                    Ok(FieldConfigProperty::Expression { self_referencing, input_type,
                        expression: proc_macro2::TokenStream::parse(&buffer)
                            .expect("Unable to convert buffer back into TokenStream")
                    })
                })
            },
            "method" => Err(Error::new(
                property.span(),
                "\"method\" property has been removed. Please refer to documentation for a list of valid properties."
            )),
            _ => Err(Error::new(
                property.span(),
                CONFIG_PROP_ERR_MSG.replace("{prop}", &property_name).replace("{values}", FIELD_PROPS)
            ))
        }
    }
}

impl ToTokens for ParameterField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.field_ident.to_tokens(tokens);
        tokens.append(Punct::new(':', Alone));
        self.field_type.to_tokens(tokens);
    }
}

impl ToTokens for GeneratedField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.field_ident;

        let token_stream = quote! {
            let #ident =
        };

        tokens.extend(token_stream);

        tokens.extend(match &self.configuration {
            FieldConfigProperty::Cloned => quote! { #ident.clone() },
            FieldConfigProperty::Default => quote! { Default::default() },
            FieldConfigProperty::Expression { expression, .. } => expression.clone(),
            FieldConfigProperty::Into => quote! { #ident.into() },
            FieldConfigProperty::Iter { .. } => quote! { #ident.into_iter().collect() },
        });

        tokens.append(Punct::new(';', Alone))
    }
}