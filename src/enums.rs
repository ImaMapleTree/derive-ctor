use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::{quote};
use syn::{Data, DeriveInput, Error, Fields, Generics, Ident, token, Variant, Visibility};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Pub};
use crate::{CONFIG_PROP_ERR_MSG, try_parse_attributes_with_default};
use crate::structs::{CtorDefinition, CtorStructConfiguration, generate_ctor_meta_from_fields};

static ENUM_CTOR_PROPS: &str = "\"prefix\", \"visibility\", \"vis\"";

enum EnumConfigItem {
    Visibility { visibility: Visibility },
    Prefix { prefix: Ident }
}

struct CtorEnumConfiguration {
    prefix: Option<Ident>,
    default_visibility: Visibility
}

impl Default for CtorEnumConfiguration {
    fn default() -> Self {
        Self { prefix: None, default_visibility: Visibility::Public(Pub { span: Span::mixed_site() }) }
    }
}

impl CtorStructConfiguration {
    fn from_variant(configuration: &CtorEnumConfiguration, variant_name: Ident) -> Self {
        Self { definitions: vec![CtorDefinition {
            visibility: configuration.default_visibility.clone(),
            ident: match &configuration.prefix {
                None => variant_name,
                Some(prefix) => syn::parse_str(&(prefix.to_string() + "_" + &variant_name.to_string())).unwrap()
            },
            is_const: false,
        }], is_none: false }
    }
}

impl Parse for CtorEnumConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut configuration = CtorEnumConfiguration::default();
        loop {
            match input.parse::<EnumConfigItem>()? {
                EnumConfigItem::Visibility { visibility } => configuration.default_visibility = visibility,
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
            "vis" | "visibility" => EnumConfigItem::Visibility { visibility: input.parse()? },
            "prefix" => EnumConfigItem::Prefix { prefix: input.parse()? },
            _ => return Err(Error::new(
                property.span(),
                CONFIG_PROP_ERR_MSG.replace("{prop}", &property_name).replace("{values}", ENUM_CTOR_PROPS)
            ))
        })
    }
}

pub(crate) fn create_enum_token_stream(derive_input: DeriveInput) -> TokenStream {
    if let Data::Enum(data) = derive_input.data {
        let configuration = match try_parse_attributes_with_default(&derive_input.attrs, || CtorEnumConfiguration::default()) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error())
        };

        return create_ctor_enum_impl(
            derive_input.ident,
            derive_input.generics,
            data.variants,
            configuration
        )
    }
    panic!("Expected Enum data")
}

fn create_ctor_enum_impl(
    ident: Ident,
    generics: Generics,
    variants: Punctuated<Variant, Comma>,
    configuration: CtorEnumConfiguration
) -> TokenStream {
    let mut methods = Vec::new();

    for variant in variants {
        let variant_code = match &variant.fields {
            Fields::Named(_) => 0,
            Fields::Unnamed(_) => 1,
            Fields::Unit => 2
        };

        let variant_name = variant.ident;
        let variant_config = match try_parse_attributes_with_default(&variant.attrs, || CtorStructConfiguration::from_variant(
            &configuration,
            variant_name.clone()
        )) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error())
        };
        
        // stop generation of method if none
        if variant_config.is_none {
            continue;
        }
    
        let meta = match generate_ctor_meta_from_fields(variant.fields, variant_config.definitions.len()) {
            Ok(meta) => meta,
            Err(err) => return TokenStream::from(err.into_compile_error()),
        };

        let field_idents = meta.field_idents;

        for (i, definition) in variant_config.definitions.into_iter().enumerate() {
            let method_req_fields = &meta.parameter_fields[i];
            let method_gen_fields = &meta.generated_fields[i];
    
            let visibility = definition.visibility;
            let name = match convert_to_snakecase(definition.ident) {
                Ok(snake_case_ident) => snake_case_ident,
                Err(err) => return TokenStream::from(err.to_compile_error())
            };

            let const_tkn = if definition.is_const { quote! { const } } else { quote!{} };
    
            let enum_generation = if variant_code == 0 {
                quote! { Self::#variant_name { #(#field_idents),* } }
            } else if variant_code == 1 {
                quote! { Self::#variant_name ( #(#field_idents),* ) }
            } else {
                quote! { Self::#variant_name }
            };

            methods.push(quote! {
                #visibility #const_tkn fn #name(#(#method_req_fields),*) -> Self {
                    #(#method_gen_fields)*
                    #enum_generation
                }
            })
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }
    })
}

fn convert_to_snakecase(method_ident: Ident) -> Result<Ident, Error> {
    let mut snake_case = String::new();
    let mut was_lower = false;

    let ident_string = method_ident.to_string();

    let mut ident_chars = ident_string.chars();
    loop {
        let c = match ident_chars.next() {
            Some(c) if c == '_' => {
                snake_case.push(c.to_ascii_lowercase());
                was_lower = false;
                continue;
            },
            Some(c) => c,
            None => break,
        };

        let lower_or_numeric = c.is_ascii_lowercase() || c.is_numeric();
        
        if was_lower && !lower_or_numeric {
            snake_case.push('_')
        }
        
        snake_case.push(c.to_ascii_lowercase());

        was_lower = lower_or_numeric;
    }

    syn::parse_str(&snake_case)
}

#[test]
fn test_convert_to_snakecase() {
    assert_eq!(convert_to_snakecase(Ident::new("A", Span::mixed_site())).unwrap(), Ident::new("a", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("Test", Span::mixed_site())).unwrap(), Ident::new("test", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("Test1", Span::mixed_site())).unwrap(), Ident::new("test1", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("ONETWO", Span::mixed_site())).unwrap(), Ident::new("onetwo", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("OneTwo", Span::mixed_site())).unwrap(), Ident::new("one_two", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("_Abc", Span::mixed_site())).unwrap(), Ident::new("_abc", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("A_B", Span::mixed_site())).unwrap(), Ident::new("a_b", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("A_b", Span::mixed_site())).unwrap(), Ident::new("a_b", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("abCdEf", Span::mixed_site())).unwrap(), Ident::new("ab_cd_ef", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("one_2_three", Span::mixed_site())).unwrap(), Ident::new("one_2_three", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("ending_", Span::mixed_site())).unwrap(), Ident::new("ending_", Span::mixed_site()));
    assert_eq!(convert_to_snakecase(Ident::new("endinG_", Span::mixed_site())).unwrap(), Ident::new("endin_g_", Span::mixed_site()));
}