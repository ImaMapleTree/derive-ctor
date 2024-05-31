// This file is almost a 1-to-1 copy of the enum file because unions work similarly to enums

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, FieldsNamed, Generics, Ident, token, Visibility};
use syn::parse::{Parse, ParseStream};
use syn::token::{Comma, Pub};

use crate::{CtorAttribute, CtorDefinition, try_parse_attributes_with_default};
use crate::constants::{CONFIG_PROP_ERR_MSG, ENUM_PROP_PREFIX as PREFIX, ENUM_PROP_VIS as VIS, ENUM_PROP_VISIBILITY as VISIBILITY};
use crate::structs::CtorStructConfiguration;

const UNION_CTOR_PROPS: &str = "\"prefix\", \"visibility\", \"vis\"";

enum UnionConfigItem {
    Visibility { visibility: Visibility },
    Prefix { prefix: Ident },
}

struct CtorUnionConfiguration {
    prefix: Option<Ident>,
    default_visibility: Visibility,
}

impl Default for CtorUnionConfiguration {
    fn default() -> Self {
        Self {
            prefix: None,
            default_visibility: Visibility::Public(Pub {
                span: Span::mixed_site(),
            }),
        }
    }
}

impl CtorStructConfiguration {
    fn from_union_field(configuration: &CtorUnionConfiguration, field_name: Ident) -> Self {
        Self {
            definitions: vec![CtorDefinition {
                visibility: configuration.default_visibility.clone(),
                ident: match &configuration.prefix {
                    None => field_name,
                    Some(prefix) => {
                        syn::parse_str(&(prefix.to_string() + "_" + &field_name.to_string())).unwrap()
                    }
                },
                attrs: Default::default(),
            }],
            is_none: false,
        }
    }
}

impl Parse for CtorUnionConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut configuration = CtorUnionConfiguration::default();
        loop {
            match input.parse::<UnionConfigItem>()? {
                UnionConfigItem::Visibility { visibility } => {
                    configuration.default_visibility = visibility
                }
                UnionConfigItem::Prefix { prefix } => configuration.prefix = Some(prefix),
            }
            if input.parse::<Comma>().is_err() {
                break;
            }
        }
        Ok(configuration)
    }
}

impl Parse for UnionConfigItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let property = input.parse::<Ident>()?;
        let property_name = property.to_string();

        input.parse::<token::Eq>()?;

        Ok(match property_name.as_str() {
            VIS | VISIBILITY => UnionConfigItem::Visibility { visibility: input.parse()?, },
            PREFIX => UnionConfigItem::Prefix { prefix: input.parse()?, },
            _ => {
                return Err(Error::new(property.span(),
                    CONFIG_PROP_ERR_MSG
                        .replace("{prop}", &property_name)
                        .replace("{values}", UNION_CTOR_PROPS),
                ))
            }
        })
    }
}

pub(crate) fn create_union_token_stream(derive_input: DeriveInput) -> TokenStream {
    if let Data::Union(data) = derive_input.data {
        let configuration = match try_parse_attributes_with_default(&derive_input.attrs, || {
            CtorUnionConfiguration::default()
        }) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error()),
        };

        return create_ctor_union_impl(
            derive_input.ident,
            derive_input.generics,
            data.fields,
            configuration,
        );
    }
    panic!("Expected Union data")
}

fn create_ctor_union_impl(
    ident: Ident,
    generics: Generics,
    fields: FieldsNamed,
    configuration: CtorUnionConfiguration,
) -> TokenStream {
    let mut methods = Vec::new();
    
    for field in fields.named {
        let field_name = field.ident.expect("Field was unnamed");
        let field_type = field.ty;

        let field_config = match try_parse_attributes_with_default(&field.attrs, || {
            CtorStructConfiguration::from_union_field(&configuration, field_name.clone())
        }) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error()),
        };

        // stop generation of method if none
        if field_config.is_none {
            continue;
        }

        for def in field_config.definitions {
            let visibility = def.visibility;
            let name = def.ident;
            let ty = field_type.clone();

            let const_tkn = if def.attrs.contains(&CtorAttribute::Const) {
                quote! { const }
            } else {
                quote! {}
            };
            
            let method_token_stream = quote! {
                #visibility #const_tkn fn #name(#field_name: #ty) -> Self {
                    Self { #field_name }
                }
            };
            
            methods.push(method_token_stream);
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    TokenStream::from(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }
    })
}