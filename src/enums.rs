use proc_macro::TokenStream;

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use heck::ToSnakeCase;

use crate::constants::{CONFIG_PROP_ERR_MSG, ENUM_PROP_VIS as VIS, ENUM_PROP_VISIBILITY as VISIBILITY, ENUM_PROP_PREFIX as PREFIX};
use crate::structs::CtorStructConfiguration;
use crate::{adjust_keyword_ident, CtorAttribute, CtorDefinition, try_parse_attributes_with_default};
use crate::fields::generate_ctor_meta;

use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Pub};
use syn::{token, Data, DeriveInput, Error, Fields, Generics, Ident, Variant, Visibility};

const ENUM_CTOR_PROPS: &str = "\"prefix\", \"visibility\", \"vis\"";

enum EnumConfigItem {
    Visibility { visibility: Visibility },
    Prefix { prefix: Ident },
}

struct CtorEnumConfiguration {
    prefix: Option<Ident>,
    default_visibility: Visibility,
}

impl Default for CtorEnumConfiguration {
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
    fn from_variant(configuration: &CtorEnumConfiguration, variant_name: Ident) -> Self {
        Self {
            definitions: vec![CtorDefinition {
                visibility: configuration.default_visibility.clone(),
                ident: match &configuration.prefix {
                    None => variant_name,
                    Some(prefix) => {
                        syn::parse_str(&(prefix.to_string() + "_" + &variant_name.to_string())).unwrap()
                    }
                },
                attrs: Default::default(),
            }],
            is_none: false,
        }
    }
}

impl Parse for CtorEnumConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut configuration = CtorEnumConfiguration::default();
        loop {
            match input.parse::<EnumConfigItem>()? {
                EnumConfigItem::Visibility { visibility } => {
                    configuration.default_visibility = visibility
                }
                EnumConfigItem::Prefix { prefix } => configuration.prefix = Some(prefix),
            }
            if input.parse::<Comma>().is_err() {
                break;
            }
        }
        Ok(configuration)
    }
}

impl Parse for EnumConfigItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let property = input.parse::<Ident>()?;
        let property_name = property.to_string();

        input.parse::<token::Eq>()?;

        Ok(match property_name.as_str() {
            VIS | VISIBILITY => EnumConfigItem::Visibility { visibility: input.parse()?, },
            PREFIX => EnumConfigItem::Prefix { prefix: input.parse()?, },
            _ => {
                return Err(Error::new(property.span(),
                    CONFIG_PROP_ERR_MSG
                        .replace("{prop}", &property_name)
                        .replace("{values}", ENUM_CTOR_PROPS),
                ))
            }
        })
    }
}

pub(crate) fn create_enum_token_stream(derive_input: DeriveInput) -> TokenStream {
    if let Data::Enum(data) = derive_input.data {
        let configuration = match try_parse_attributes_with_default(&derive_input.attrs, || { 
            CtorEnumConfiguration::default()
        }) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error()),
        };

        return create_ctor_enum_impl(
            derive_input.ident,
            derive_input.generics,
            data.variants,
            configuration,
        );
    }
    panic!("Expected Enum data")
}

fn create_ctor_enum_impl(
    ident: Ident,
    generics: Generics,
    variants: Punctuated<Variant, Comma>,
    configuration: CtorEnumConfiguration,
) -> TokenStream {
    let mut methods = Vec::new();
    let mut default_method = None;

    for variant in variants {
        let variant_code = match &variant.fields {
            Fields::Named(_) => 0,
            Fields::Unnamed(_) => 1,
            Fields::Unit => 2,
        };

        let variant_name = variant.ident;
        let variant_config = match try_parse_attributes_with_default(&variant.attrs, || {
            CtorStructConfiguration::from_variant(&configuration, variant_name.clone())
        }) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error()),
        };

        // stop generation of method if none
        if variant_config.is_none {
            continue;
        }

        for (i, def) in variant_config.definitions.into_iter().enumerate() {
            let meta = match generate_ctor_meta(&def.attrs, &variant.fields, i) {
                Ok(meta) => meta,
                Err(err) => return TokenStream::from(err.into_compile_error()),
            };

            let field_idents = meta.field_idents;
            let parameter_fields = meta.parameter_fields;
            let generated_fields = meta.generated_fields;

            let visibility = def.visibility;
            let name = match convert_to_snakecase(def.ident) {
                Ok(snake_case_ident) => snake_case_ident,
                Err(err) => return TokenStream::from(err.to_compile_error()),
            };

            let const_tkn = if def.attrs.contains(&CtorAttribute::Const) {
                quote! { const }
            } else {
                quote! {}
            };

            let enum_generation = if variant_code == 0 {
                quote! { Self::#variant_name { #(#field_idents),* } }
            } else if variant_code == 1 {
                quote! { Self::#variant_name ( #(#field_idents),* ) }
            } else {
                quote! { Self::#variant_name }
            };
            
            let method_token_stream = quote! {
                #visibility #const_tkn fn #name(#(#parameter_fields),*) -> Self {
                    #(#generated_fields)*
                    #enum_generation
                }
            };

            if def.attrs.contains(&CtorAttribute::Default) {
                default_method = Some(method_token_stream);
            } else {
                methods.push(method_token_stream);
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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

fn convert_to_snakecase(method_ident: Ident) -> Result<Ident, Error> {
    let ident_string = method_ident.to_string();
    let trimmed_start_str = ident_string.trim_start_matches('_');
    let trimmed_start_end_str = trimmed_start_str.trim_end_matches('_');

    let leading_underscore_count = ident_string.len() - trimmed_start_str.len();
    let trailing_underscore_count = trimmed_start_str.len() - trimmed_start_end_str.len();

    let snake_case = "_".repeat(leading_underscore_count)
        + &ident_string.to_snake_case()
        + &"_".repeat(trailing_underscore_count);

    syn::parse_str(&adjust_keyword_ident(snake_case))
}

#[test]
fn test_convert_to_snakecase() {
    assert_eq!(
        convert_to_snakecase(Ident::new("A", Span::mixed_site())).unwrap(),
        Ident::new("a", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("Test", Span::mixed_site())).unwrap(),
        Ident::new("test", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("Test1", Span::mixed_site())).unwrap(),
        Ident::new("test1", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("ONETWO", Span::mixed_site())).unwrap(),
        Ident::new("onetwo", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("OneTwo", Span::mixed_site())).unwrap(),
        Ident::new("one_two", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("__Abc__", Span::mixed_site())).unwrap(),
        Ident::new("__abc__", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("A_B", Span::mixed_site())).unwrap(),
        Ident::new("a_b", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("A_b", Span::mixed_site())).unwrap(),
        Ident::new("a_b", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("abCdEf", Span::mixed_site())).unwrap(),
        Ident::new("ab_cd_ef", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("one_2_three", Span::mixed_site())).unwrap(),
        Ident::new("one_2_three", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("ending_", Span::mixed_site())).unwrap(),
        Ident::new("ending_", Span::mixed_site())
    );
    assert_eq!(
        convert_to_snakecase(Ident::new("endinG_", Span::mixed_site())).unwrap(),
        Ident::new("endin_g_", Span::mixed_site())
    );
}
