extern crate alloc;

use alloc::collections::BTreeSet as HashSet;
use alloc::string::ToString;
use alloc::vec::Vec;

use proc_macro2::{Delimiter, Punct, Span, TokenTree};
use proc_macro2::Spacing::Alone;
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{Attribute, Error, Fields, Ident, LitInt, parse2, token, Token, Type};
use syn::parse::{Parse, ParseStream};
use syn::parse::discouraged::AnyDelimiter;
use syn::spanned::Spanned;
use syn::token::Comma;

use crate::{consume_delimited, CtorAttribute, is_phantom_data};
use crate::constants::{CONFIG_PROP_ERR_MSG, CTOR_WORD, FIELD_PROP_CLONED as CLONED, FIELD_PROP_DEFAULT as DEFAULT, FIELD_PROP_EXPR as EXPR, FIELD_PROP_INTO as INTO, FIELD_PROP_ITER as ITER};

const FIELD_PROPS: &str = "\"cloned\", \"default\", \"expr\", \"into\", \"iter\"";

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
    pub(crate) applications: HashSet<usize>,
}

#[derive(Clone)]
pub(crate) enum FieldConfigProperty {
    Cloned,
    Default,
    Into,
    Iter {
        iter_type: Type,
    },
    Expression {
        expression: proc_macro2::TokenStream,
        input_type: Option<Type>,
        self_referencing: bool,
    },
}

#[derive(Default)]
pub(crate) struct ConstructorMeta {
    pub(crate) field_idents: Vec<Ident>,
    pub(crate) parameter_fields: Vec<ParameterField>,
    pub(crate) generated_fields: Vec<GeneratedField>,
}

#[derive(Clone)]
pub(crate) struct ParameterField {
    pub(crate) field_ident: Ident,
    pub(crate) field_type: Type,
    pub(crate) span: Span,
}

#[derive(Clone)]
pub(crate) struct GeneratedField {
    pub(crate) field_ident: Ident,
    pub(crate) configuration: FieldConfigProperty,
    #[allow(dead_code /*may be used for future purposes*/)]
    pub(crate) span: Span,
}

impl Parse for FieldConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = FieldConfig {
            property: input.parse()?,
            applications: HashSet::default(),
        };

        if input.parse::<token::Eq>().is_err() {
            return Ok(config);
        }

        // consume constructor specifier ex: 1, 2, 3
        if let Ok((delim, span, buffer)) = input.parse_any_delimiter() {
            if delim != Delimiter::Bracket {
                return Err(Error::new(span.span(), "Expected enclosing brackets"));
            }
            loop {
                config
                    .applications
                    .insert(buffer.parse::<LitInt>()?.base10_parse()?);
                if buffer.parse::<Comma>().is_err() {
                    break;
                }
            }
        } else {
            config
                .applications
                .insert(input.parse::<LitInt>()?.base10_parse()?);
        }

        Ok(config)
    }
}

impl FieldConfigProperty {
    fn is_generated(&self) -> bool {
        match self {
            FieldConfigProperty::Cloned => false,
            FieldConfigProperty::Default => true,
            FieldConfigProperty::Into => false,
            FieldConfigProperty::Iter { .. } => false,
            FieldConfigProperty::Expression { self_referencing, .. } => !self_referencing
        }
    }
}

impl Parse for FieldConfigProperty {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let property: Ident = input.parse()?;
        let property_name = property.to_string();
        match property_name.as_str() {
            CLONED => Ok(FieldConfigProperty::Cloned),
            DEFAULT => Ok(FieldConfigProperty::Default),
            INTO => Ok(FieldConfigProperty::Into),
            ITER => consume_delimited(input, Delimiter::Parenthesis, |buffer| {
                Ok(FieldConfigProperty::Iter { iter_type: buffer.parse()? })
            }),
            EXPR => {
                let self_referencing = input.parse::<Token![!]>().is_ok();

                consume_delimited(input, Delimiter::Parenthesis, |buffer| {
                    let mut input_type = None;

                    // determine the input_type by looking for the expression: expr(TYPE -> EXPRESSION)
                    if buffer.peek2(Token![->]) {
                        input_type = Some(buffer.parse()?);
                        buffer.parse::<Token![->]>()?;
                    }

                    Ok(FieldConfigProperty::Expression { self_referencing, input_type,
                        expression: proc_macro2::TokenStream::parse(buffer)
                            .expect("Unable to convert buffer back into TokenStream")
                    })
                })
            }
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

fn try_parse_field_attributes(attributes: &[Attribute]) -> Result<Option<FieldConfig>, Error> {
    for attribute in attributes {
        let attr_path = attribute.path();
        if attr_path.is_ident(CTOR_WORD) {
            return attribute.parse_args().map(Some);
        }
        let attribute_token_stream = attribute.to_token_stream();
        if let Some(TokenTree::Group(group)) = attribute_token_stream.into_iter().skip(1).next() {
            if let Ok(property) = parse2::<FieldConfigProperty>(group.stream()) {
                return Ok(Some(FieldConfig { property, applications: Default::default() }))
            }
        }
    }
    Ok(None)
}

pub(crate) fn generate_ctor_meta(
    ctor_attributes: &HashSet<CtorAttribute>,
    fields: &Fields,
    ctor_index: usize,
) -> Result<ConstructorMeta, Error> {
    let mut meta = ConstructorMeta::default();

    for (field_index, field) in fields.iter().enumerate() {
        let configuration = try_parse_field_attributes(&field.attrs)?;

        let span = field.span();

        let field_ident = field.ident.clone().unwrap_or_else(|| {
            Ident::new(
                &("arg".to_string() + &field_index.to_string()),
                Span::mixed_site(),
            )
        });

        meta.field_idents.push(field_ident.clone());

        let field_ident = field_ident.clone();
        let ft = &field.ty;

        let mut req_field_type = None;
        let mut gen_configuration = None;
        let is_default_all = ctor_attributes.contains(&CtorAttribute::DefaultAll);
        let is_into_all = ctor_attributes.contains(&CtorAttribute::IntoAll);

        match &configuration {
            None if is_default_all => {
                gen_configuration = Some(FieldConfigProperty::Default)
            }
            None if is_into_all => {
                req_field_type = Some(parse2(quote! { impl Into<#ft> }).expect("Could not parse `Into` type"));
                gen_configuration = Some(FieldConfigProperty::Into)
            }
            None if is_phantom_data(&field.ty) => {
                gen_configuration = Some(FieldConfigProperty::Default)
            }
            None => req_field_type = Some(field.ty.clone()),
            // default(all) should generate a property if the property is a non-generated one
            Some(configuration) if !configuration.property.is_generated() && is_default_all => {
                gen_configuration = Some(FieldConfigProperty::Default)
            }
            Some(configuration) => {
                let applications = &configuration.applications;
                gen_configuration = Some(configuration.property.clone());

                if applications.is_empty() || applications.contains(&ctor_index) {
                    // create a required field type if the configuration requires an additional input parameter
                    req_field_type = match &configuration.property {
                        FieldConfigProperty::Cloned => Some(parse2(quote! { &#ft }).expect("Could not parse ref type")),
                        FieldConfigProperty::Into => {
                            Some(parse2(quote! { impl Into<#ft> }).expect("Could not parse `Into` type"))
                        }
                        FieldConfigProperty::Iter { iter_type } => {
                            Some(parse2(quote! { impl IntoIterator<Item=#iter_type> }).expect("Could not parse `IntoIterator` type"))
                        }
                        FieldConfigProperty::Expression { input_type, .. }
                            if input_type.is_some() =>
                        {
                            input_type.clone()
                        }
                        FieldConfigProperty::Expression {
                            self_referencing, ..
                        } if *self_referencing => Some(field.ty.clone()),
                        _ => None,
                    }
                } else if is_phantom_data(&field.ty) {
                    gen_configuration = Some(FieldConfigProperty::Default);
                } else {
                    gen_configuration = None;
                    req_field_type = Some(field.ty.clone());
                }
            }
        }

        if let Some(cfg) = gen_configuration {
            meta.generated_fields.push(GeneratedField {
                field_ident: field_ident.clone(),
                configuration: cfg,
                span,
            })
        }
        if let Some(field_type) = req_field_type {
            meta.parameter_fields.push(ParameterField {
                field_ident,
                field_type,
                span,
            })
        }
    }
    Ok(meta)
}
