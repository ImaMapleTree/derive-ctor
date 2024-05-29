#![cfg_attr(feature = "no-std", no_std, doc = "Test")]
#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::Delimiter;
use quote::ToTokens;
use structs::create_struct_token_stream;
use syn::parse::discouraged::AnyDelimiter;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Type;
use crate::enums::create_enum_token_stream;

pub(crate) mod enums;
pub(crate) mod structs;
pub(crate) mod fields;

pub(crate) static CONFIG_PROP_ERR_MSG: &str =
    "Unexpected property: \"{prop}\" (must be one of the following: \"{values}\")";

#[proc_macro_derive(ctor, attributes(ctor))]
pub fn derive_ctor(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    match &derive_input.data {
        Data::Struct(_) => create_struct_token_stream(derive_input),
        Data::Enum(_) => create_enum_token_stream(derive_input),
        Data::Union(_) => TokenStream::from(Error::new(derive_input.span(), "Unions are not yet supported by ctor").to_compile_error())
    }
}

pub(crate) fn try_parse_attributes_with_default<T: Parse, F: Fn() -> T>(attributes: &[Attribute], default: F) -> Result<T, Error> {
    for attribute in attributes {
        if attribute.path().is_ident("ctor") {
            return attribute.parse_args::<T>()
        }
    }
    Ok(default())
}

pub(crate) fn try_parse_attributes<T: Parse>(attributes: &[Attribute]) -> Result<Option<T>, Error> {
    for attribute in attributes {
        if attribute.path().is_ident("ctor") {
            return attribute.parse_args::<T>().map(|t| Some(t))
        }
    }
    Ok(None)
}

pub(crate) fn is_phantom_data(typ: &Type) -> bool {
    for token in typ.to_token_stream() {
        if token.to_string() == "PhantomData" {
            return true;
        }
    }
    false
}

pub(crate) fn consume_delimited<T, F>(stream: ParseStream, expected: Delimiter, expression: F) -> Result<T, Error>
where F: Fn(ParseStream) -> Result<T, Error> {
    let (delimiter, span, buffer) = stream.parse_any_delimiter()?;
    if delimiter != expected {
        return Err(Error::new(span.span(), format!("Expected enclosing {:?}", expected)))
    }
    expression(&buffer)
}


#[test]
fn test_is_phantom_data() {
    assert!(is_phantom_data(&syn::parse_str::<Type>("PhantomData").unwrap()));
    assert!(is_phantom_data(&syn::parse_str::<Type>("&mut PhantomData<&'static str>").unwrap()));
    assert!(!is_phantom_data(&syn::parse_str::<Type>("i32").unwrap()));
}