extern crate alloc;

use alloc::collections::BTreeSet as HashSet;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use proc_macro::TokenStream;

use proc_macro2::Delimiter;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Generics, Ident, Visibility};
use syn::parse::{Parse, ParseStream};
use syn::token::{Comma, Const};

use crate::{consume_delimited, CtorAttribute, CtorDefinition, try_parse_attributes_with_default};
use crate::constants::{DEFAULT_CTOR_ERR_MSG, ENUM_VARIATION_PROP_NONE as NONE, NESTED_PROP_ALL as ALL, STRUCT_PROP_DEFAULT as DEFAULT, STRUCT_PROP_INTO as INTO};
use crate::fields::generate_ctor_meta;

pub(crate) struct CtorStructConfiguration {
    pub(crate) definitions: Vec<CtorDefinition>,
    pub(crate) is_none: bool,
}

impl Default for CtorStructConfiguration {
    fn default() -> Self {
        Self {
            definitions: vec![CtorDefinition::default()],
            is_none: false,
        }
    }
}

impl Parse for CtorStructConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let mut definitions = Vec::new();

        loop {
            let mut attributes = HashSet::new();
            if input.parse::<Const>().is_ok() {
                attributes.insert(CtorAttribute::Const);
            }

            let definition = if !input.peek(syn::Ident) {
                let visibility = input.parse()?;
                // required to support both: VIS const and const VIS
                if input.parse::<Const>().is_ok() {
                    attributes.insert(CtorAttribute::Const);
                }
                CtorDefinition {
                    visibility,
                    ident: input.parse()?,
                    attrs: attributes,
                }
            } else {
                let ident = input.parse::<Ident>()?;

                match ident.to_string().as_str() {
                    // check for "none" as first parameter, if exists return early (this is only applicable for enums)
                    NONE if definitions.is_empty() => {
                        return Ok(CtorStructConfiguration {
                            definitions: Default::default(),
                            is_none: true,
                        })
                    }
                    DEFAULT => {
                        if let Ok(true) =
                            consume_delimited(input, Delimiter::Parenthesis, |buffer| {
                                Ok(buffer.parse::<Ident>()? == ALL)
                            })
                        {
                            attributes.insert(CtorAttribute::DefaultAll);
                        }
                        attributes.insert(CtorAttribute::Default);
                    }
                    _ => {}
                }
                
                if let Ok(Some(attribute)) = consume_delimited(input, Delimiter::Parenthesis, |buffer| {
                    let ident = buffer.parse::<Ident>()?;
                    if ident == DEFAULT {
                        return Ok(Some(CtorAttribute::DefaultAll))
                    }
                    if ident == INTO {
                        return Ok(Some(CtorAttribute::IntoAll))
                    }
                    Ok(None)
                }) {
                    attributes.insert(attribute);
                }

                CtorDefinition {
                    visibility: Visibility::Inherited,
                    ident,
                    attrs: attributes,
                }
            };

            definitions.push(definition);

            // Consume a comma to continue looking for constructors
            if input.parse::<Comma>().is_err() {
                break;
            }
        }

        Ok(Self {
            definitions,
            is_none: false,
        })
    }
}

pub(crate) fn create_struct_token_stream(derive_input: DeriveInput) -> TokenStream {
    if let Data::Struct(data) = derive_input.data {
        let configuration = match try_parse_attributes_with_default(&derive_input.attrs, || {
            CtorStructConfiguration::default()
        }) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error()),
        };

        return create_ctor_struct_impl(
            derive_input.ident,
            derive_input.generics,
            data.fields,
            configuration,
        );
    }
    panic!("Expected Struct data")
}

fn create_ctor_struct_impl(
    ident: Ident,
    generics: Generics,
    fields: Fields,
    configuration: CtorStructConfiguration,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut methods = Vec::new();
    let mut default_method = None;

    for (i, definition) in configuration.definitions.into_iter().enumerate() {
        let meta = match generate_ctor_meta(&definition.attrs, &fields, i) {
            Ok(meta) => meta,
            Err(err) => return TokenStream::from(err.into_compile_error()),
        };

        let field_idents = meta.field_idents;
        let parameter_fields = meta.parameter_fields;
        let generated_fields = meta.generated_fields;

        let visibility = definition.visibility;
        let mut name = definition.ident;
        let const_tkn = if definition.attrs.contains(&CtorAttribute::Const) {
            quote! { const }
        } else {
            quote! {}
        };

        let is_default = definition.attrs.contains(&CtorAttribute::Default);

        if is_default {
            name = syn::parse_str("default").unwrap();
        }

        let method_token_stream = quote! {
            #visibility #const_tkn fn #name(#(#parameter_fields),*) -> Self {
                #(#generated_fields)*
                Self { #(#field_idents),* }
            }
        };

        if is_default {
            if !parameter_fields.is_empty() {
                let first_error = Error::new(parameter_fields[0].span, DEFAULT_CTOR_ERR_MSG);
                let errors = parameter_fields
                    .into_iter()
                    .skip(1)
                    .fold(first_error, |mut e, f| {
                        e.combine(Error::new(f.span, DEFAULT_CTOR_ERR_MSG));
                        e
                    });
                return TokenStream::from(errors.to_compile_error());
            }
            default_method = Some(method_token_stream);
        } else {
            methods.push(method_token_stream);
        }
    }

    let default_impl = if let Some(def_method) = default_method {
        quote! {
            impl #impl_generics Default for # ident # ty_generics #where_clause {
                #def_method
            }
        }
    } else {
        quote! {}
    };

    TokenStream::from(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }
        #default_impl
    })
}
