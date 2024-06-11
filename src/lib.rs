#![no_std]
#![doc = include_str!("../README.md")]
#![allow(dead_code)]

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::collections::BTreeSet as HashSet;

use crate::constants::CTOR_WORD;
#[cfg(feature = "enums")]
use crate::enums::create_enum_token_stream;
#[cfg(feature = "structs")]
use crate::structs::create_struct_token_stream;
#[cfg(feature = "unions")]
use crate::unions::create_union_token_stream;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Ident, Span};
use quote::ToTokens;
use syn::parse::discouraged::AnyDelimiter;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::{parse_macro_input, Visibility};
use syn::spanned::Spanned;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::token::Pub;
use syn::Type;


pub(crate) mod constants;
#[cfg(feature = "enums")]
pub(crate) mod enums;
#[cfg(any(feature = "enums", feature = "structs", feature = "unions"))]
pub(crate) mod fields;
#[cfg(feature = "structs")]
pub(crate) mod structs;
#[cfg(feature = "unions")]
pub(crate) mod unions;

pub(crate) struct CtorDefinition {
    pub(crate) visibility: Visibility,
    pub(crate) ident: Ident,
    pub(crate) attrs: HashSet<CtorAttribute>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CtorAttribute {
    Const,
    DefaultAll,
    Default,
    IntoAll,
}

impl Default for CtorDefinition {
    fn default() -> Self {
        Self {
            visibility: Visibility::Public(Pub {
                span: Span::call_site(),
            }),
            ident: Ident::new("new", Span::mixed_site()),
            attrs: Default::default(),
        }
    }
}

#[cfg(not(feature = "enums"))]
pub(crate) fn create_enum_token_stream(_derive_input: DeriveInput) -> TokenStream {
    use proc_macro2::Span;
    TokenStream::from(Error::new(Span::call_site(),
        "\"enums\" feature must be enabled to use #[derive(ctor)] on enums.").to_compile_error())
}

#[cfg(not(feature = "structs"))]
pub(crate) fn create_struct_token_stream(_derive_input: DeriveInput) -> TokenStream {
    TokenStream::from(Error::new(Span::call_site(),
        "\"structs\" feature must be enabled to use #[derive(ctor)] on structs.").to_compile_error())
}

#[cfg(not(feature = "unions"))]
pub(crate) fn create_union_token_stream(_derive_input: DeriveInput) -> TokenStream {
    TokenStream::from(Error::new(Span::call_site(),
        "\"unions\" feature must be enabled to use #[derive(ctor)] on unions.").to_compile_error())
}

#[cfg(feature = "shorthand")]
#[proc_macro_derive(ctor, attributes(ctor, cloned, default, expr, into, iter))]
pub fn derive_ctor(input: TokenStream) -> TokenStream {
    derive_ctor_internal(input)
}

#[cfg(not(feature = "shorthand"))]
#[proc_macro_derive(ctor, attributes(ctor))]
pub fn derive_ctor(input: TokenStream) -> TokenStream {
    derive_ctor_internal(input)
}


fn derive_ctor_internal(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    match &derive_input.data {
        Data::Struct(_) => create_struct_token_stream(derive_input),
        Data::Enum(_) => create_enum_token_stream(derive_input),
        Data::Union(_) => create_union_token_stream(derive_input)
    }
}




pub(crate) fn try_parse_attributes_with_default<T: Parse, F: Fn() -> T>(
    attributes: &[Attribute],
    default: F,
) -> Result<T, Error> {
    for attribute in attributes {
        if attribute.path().is_ident(CTOR_WORD) {
            return attribute.parse_args::<T>();
        }
    }
    Ok(default())
}

pub(crate) fn try_parse_attributes<T: Parse>(attributes: &[Attribute]) -> Result<Option<T>, Error> {
    for attribute in attributes {
        if attribute.path().is_ident(CTOR_WORD) {
            return attribute.parse_args::<T>().map(Some);
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

pub(crate) fn consume_delimited<T, F>(
    stream: ParseStream,
    expected: Delimiter,
    expression: F,
) -> Result<T, Error>
where
    F: Fn(ParseStream) -> Result<T, Error>,
{
    let (delimiter, span, buffer) = stream.parse_any_delimiter()?;
    if delimiter != expected {
        return Err(Error::new(span.span(),
            format!("Expected enclosing {:?}", expected),
        ));
    }
    expression(&buffer)
}

pub(crate) fn adjust_keyword_ident(name: String) -> String {
    if syn::parse_str::<Ident>(&name).is_ok() {
        return name;
    }
    format!("r#{}", name)
}

#[test]
fn test_is_phantom_data() {
    assert!(is_phantom_data(&syn::parse_str::<Type>("PhantomData").unwrap()));
    assert!(is_phantom_data(&syn::parse_str::<Type>("&mut PhantomData<&'static str>").unwrap()));
    assert!(!is_phantom_data(&syn::parse_str::<Type>("i32").unwrap()));
}

#[test]
fn test_adjust_keyword_ident() {
    assert_eq!("abc".to_string(), adjust_keyword_ident("abc".to_string()));
    assert_eq!("r#break".to_string(), adjust_keyword_ident("break".to_string()));
    assert_eq!("r#fn".to_string(), adjust_keyword_ident("fn".to_string()));
    assert_eq!("r#const".to_string(), adjust_keyword_ident("const".to_string()));
}