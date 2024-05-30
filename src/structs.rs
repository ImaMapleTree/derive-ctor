#[cfg(feature = "no-std")]
extern crate alloc;
#[cfg(feature = "no-std")]
use alloc::vec;
#[cfg(feature = "no-std")]
use alloc::vec::Vec;
#[cfg(feature = "no-std")]
use alloc::collections::BTreeSet as HashSet;
#[cfg(feature = "no-std")]
use alloc::string::ToString;

#[cfg(not(feature = "no-std"))]
use std::collections::HashSet;
#[cfg(not(feature = "no-std"))]
use std::vec;
#[cfg(not(feature = "no-std"))]
use std::vec::Vec;

use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse2, Data, DeriveInput, Error, Fields, Generics, Ident, Visibility};
use syn::parse::{ParseStream, Parse};
use syn::token::{Comma, Const, Pub};

use crate::fields::{FieldConfig, FieldConfigProperty, GeneratedField, ParameterField};
use crate::{is_phantom_data, try_parse_attributes, try_parse_attributes_with_default};

static DEFAULT_CTOR_ERR_MSG: &'static str = "Default constructor requires field to generate its own value.";

pub(crate) struct CtorDefinition {
    pub(crate) visibility: Visibility,
    pub(crate) ident: Ident,
    pub(crate) attributes: HashSet<CtorAttribute>
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CtorAttribute {
    Const,
    Default
}

impl Default for CtorDefinition {
    fn default() -> Self {
        Self {
            visibility: Visibility::Public(Pub { span: Span::call_site() }),
            ident: Ident::new("new", Span::mixed_site()),
            attributes: Default::default()
        }
    }
}

pub(crate) struct CtorStructConfiguration {
    pub(crate) definitions: Vec<CtorDefinition>,
    pub(crate) is_none: bool
}

impl Default for CtorStructConfiguration {
    fn default() -> Self {
        Self { definitions: vec![CtorDefinition::default()], is_none: false }
    }
}

impl Parse for CtorStructConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default())
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
                    attributes
                }
            } else {
                let ident = input.parse::<Ident>()?;
                
                match ident.to_string().as_str() {
                    // check for "none" as first parameter, if exists return early (this is only applicable for enums)
                    "none" if definitions.is_empty() => {
                        return Ok(CtorStructConfiguration { definitions: Default::default(), is_none: true })
                    },
                    "Default" => { attributes.insert(CtorAttribute::Default); },
                    _ => {}
                }
                
                CtorDefinition {
                    visibility: Visibility::Inherited,
                    ident,
                    attributes
                }
            };

            definitions.push(definition);

            // Consume a comma to continue looking for constructors
            if input.parse::<Comma>().is_err() {
                break;
            }
        }

        Ok(Self { definitions, is_none: false })
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
    let mut default_method = None;
    for (i, definition) in configuration.definitions.into_iter().enumerate() {
        let method_req_fields = &meta.parameter_fields[i];
        let method_gen_fields = &meta.generated_fields[i];

        let visibility = definition.visibility;
        let mut name = definition.ident;
        let const_tkn = if definition.attributes.contains(&CtorAttribute::Const) 
            { quote! { const } } else { quote!{} };

        let is_default = definition.attributes.contains(&CtorAttribute::Default);
        if is_default {
            name = syn::parse_str("default").unwrap();
        }

        let method_token_stream = quote! {
            #visibility #const_tkn fn #name(#(#method_req_fields),*) -> Self {
                #(#method_gen_fields)*
                Self { #(#field_idents),* }
            }
        };

        if is_default {
            if !method_req_fields.is_empty() {
                let first_error = Error::new(method_req_fields[0].span, DEFAULT_CTOR_ERR_MSG);
                let errors = method_req_fields.into_iter().skip(1).fold(first_error, |mut e, f| {
                    e.combine(Error::new(f.span, DEFAULT_CTOR_ERR_MSG));
                    e
                });
                return TokenStream::from(errors.to_compile_error())
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
    } else { quote!{} };

    TokenStream::from(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }
        #default_impl
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

        let span = field.span();

        let field_ident = field.ident.unwrap_or_else(|| {
            Ident::new(&("arg".to_string() + &field_index.to_string()), Span::mixed_site())
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
                    configuration: cfg,
                    span
                })
            }
            if let Some(field_type) = req_field_type {
                meta.parameter_fields[i].push(ParameterField { field_ident, field_type, span })
            }
        }
    }
    Ok(meta)
}