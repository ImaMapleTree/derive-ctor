#[cfg(feature = "no-std")]
extern crate alloc;
#[cfg(feature = "no-std")]
use alloc::vec;
#[cfg(feature = "no-std")]
use alloc::vec::Vec;
#[cfg(feature = "no-std")]

#[cfg(not(feature = "no-std"))]
use std::vec;
#[cfg(not(feature = "no-std"))]
use std::vec::Vec;

use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::quote;
use syn::{parse2, Data, DeriveInput, Error, Fields, Generics, Ident, Visibility};
use syn::parse::{ParseStream, Parse};
use syn::token::{Comma, Const, Pub};

use crate::fields::{FieldConfig, FieldConfigProperty, GeneratedField, ParameterField};
use crate::{is_phantom_data, try_parse_attributes, try_parse_attributes_with_default};

pub(crate) struct CtorDefinition {
    pub(crate) visibility: Visibility,
    pub(crate) ident: Ident,
    pub(crate) is_const: bool
}

impl Default for CtorDefinition {
    fn default() -> Self {
        Self {
            visibility: Visibility::Public(Pub { span: Span::call_site() }),
            ident: Ident::new("new", Span::mixed_site()),
            is_const: false
        }
    }
}

pub(crate) struct CtorStructConfiguration {
    pub(crate) definitions: Vec<CtorDefinition>
}

impl Default for CtorStructConfiguration {
    fn default() -> Self {
        Self { definitions: vec![CtorDefinition::default()] }
    }
}

impl Parse for CtorStructConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default())
        }

        let mut definitions = Vec::new();

        loop {
            let mut is_const = input.parse::<Const>().is_ok();

            let definition = if !input.peek(syn::Ident) {
                let visibility = input.parse()?;
                is_const = input.parse::<Const>().is_ok() || is_const; // required to support both: VIS const and const VIS
                CtorDefinition {
                    visibility,
                    ident: input.parse()?,
                    is_const
                }
            } else {
                CtorDefinition {
                    visibility: Visibility::Inherited,
                    ident: input.parse()?,
                    is_const
                }
            };

            definitions.push(definition);

            // Consume a comma to continue looking for constructors
            if input.parse::<Comma>().is_err() {
                break;
            }
        }

        Ok(Self { definitions })
    }
}

pub(crate) fn create_struct_token_stream(derive_input: DeriveInput) -> TokenStream {
    if let Data::Struct(data) = derive_input.data {
        let configuration = match try_parse_attributes_with_default(&derive_input.attrs, || CtorStructConfiguration::default()) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error())
        };

        return create_ctor_struct_impl(
            derive_input.ident,
            derive_input.generics,
            data.fields,
            configuration
        )
    }
    panic!("Expected Struct data")
}

fn create_ctor_struct_impl(
    ident: Ident,
    generics: Generics,
    fields: Fields,
    configuration: CtorStructConfiguration
) -> TokenStream {
    let meta = match generate_ctor_meta_from_fields(fields, configuration.definitions.len()) {
        Ok(meta) => meta,
        Err(err) => return TokenStream::from(err.into_compile_error()),
    };

    let field_idents = meta.field_idents;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut methods = Vec::new();
    for (i, definition) in configuration.definitions.into_iter().enumerate() {
        let method_req_fields = &meta.parameter_fields[i];
        let method_gen_fields = &meta.generated_fields[i];

        let visibility = definition.visibility;
        let name = definition.ident;
        let const_tkn = if definition.is_const { quote! { const } } else { quote!{} };

        methods.push(quote! {
            #visibility #const_tkn fn #name(#(#method_req_fields),*) -> Self {
                #(#method_gen_fields)*
                Self { #(#field_idents),* }
            }
        })
    }


    TokenStream::from(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }
    })
}

pub(crate) struct ConstructorMeta {
    pub(crate) parameter_fields: Vec<Vec<ParameterField>>,
    pub(crate) generated_fields: Vec<Vec<GeneratedField>>,
    pub(crate) field_idents: Vec<Ident>
}

impl ConstructorMeta {
    pub(crate) fn new(method_count: usize) -> ConstructorMeta {
        ConstructorMeta {
            parameter_fields: vec![Default::default(); method_count],
            generated_fields: vec![Default::default(); method_count],
            field_idents: Default::default()
        }
    }
}


pub(crate) fn generate_ctor_meta_from_fields(fields: Fields, method_count: usize) -> Result<ConstructorMeta, Error> {
    let mut meta = ConstructorMeta::new(method_count);
    for (field_index, field) in fields.into_iter().enumerate() {
        let configuration = try_parse_attributes::<FieldConfig>(&field.attrs)?;

        let field_ident = field.ident.unwrap_or_else(|| {
            Ident::new(&("arg".to_owned() + &field_index.to_string()), Span::mixed_site())
        });

        meta.field_idents.push(field_ident.clone());

        for i in 0..method_count {

            let field_ident = field_ident.clone();
            let ft = &field.ty;

            let mut req_field_type = None;
            let mut gen_configuration = None;

            match &configuration {
                None if is_phantom_data(&field.ty) => gen_configuration = Some(FieldConfigProperty::Default),
                None => req_field_type = Some(field.ty.clone()),
                Some(configuration) => {
                    let applications = &configuration.applications;
                    gen_configuration = Some(configuration.property.clone());
                    
                    if applications.is_empty() || applications.contains(&i) {
                        // create a required field type if the configuration requires an additional input parameter
                        req_field_type = match &configuration.property {
                            FieldConfigProperty::Cloned => Some(parse2(quote! { &#ft }).unwrap()),
                            FieldConfigProperty::Into => Some(parse2(quote! { impl Into<#ft> }).unwrap()),
                            FieldConfigProperty::Iter { iter_type } => Some(parse2(quote! { impl IntoIterator<Item=#iter_type> }).unwrap()),
                            FieldConfigProperty::Expression { input_type, .. } if input_type.is_some() => input_type.clone(),
                            FieldConfigProperty::Expression { self_referencing, .. } if *self_referencing => Some(field.ty.clone()),
                            _ => None
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
                meta.generated_fields[i].push(GeneratedField {
                    field_ident: field_ident.clone(),
                    configuration: cfg
                })
            }
            if let Some(field_type) = req_field_type {
                meta.parameter_fields[i].push(ParameterField { field_ident, field_type })
            }
        }
    }
    Ok(meta)
}