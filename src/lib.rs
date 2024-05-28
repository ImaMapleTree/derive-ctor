use proc_macro::TokenStream;
use std::collections::HashSet;

use proc_macro2::{Delimiter, Punct, Span};
use proc_macro2::Spacing::Alone;
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{Attribute, Data, DeriveInput, Error, Fields, Generics, Ident, LitInt, parse::Parse, parse_macro_input, token, Type, Visibility};
use syn::parse::discouraged::AnyDelimiter;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::token::{Comma, Impl};

static FIELD_CONFIG_ERR_MSG: &str =
    "Unexpected property: \"{prop}\" (must be one of the following:\
    \"default\", \"method\", \"value\", \"impl\")";

struct CtorTypeConfiguration {
    definitions: Vec<(Visibility, Ident)>
}

impl Default for CtorTypeConfiguration {
    fn default() -> Self {
        Self { definitions: vec![(Visibility::Inherited, Ident::new("new", Span::mixed_site()))] }
    }
}

/// Represents a configuration on a struct field
///
/// # Example
///
/// ```
/// struct Example {
///     #[ctor(default = [1, 2])]
///    field: i16
/// }
/// ```
#[derive(Clone)]
struct FieldConfig {
    property: FieldConfigProperty,
    applications: HashSet<usize>
}

#[derive(Clone)]
enum FieldConfigProperty {
    Default,
    Impl,
    Method { ident: Ident },
    Value { expression: proc_macro2::TokenStream }
}

struct RequiredStructField {
    field_ident: Ident,
    field_type: Type
}

struct GeneratedStructField {
    field_ident: Ident,
    configuration: FieldConfigProperty
}

impl Parse for CtorTypeConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default())
        }

        let mut definitions = Vec::new();

        loop {
            if !input.peek(syn::Ident) {
                definitions.push((input.parse()?, input.parse()?));
            } else {
                definitions.push((Visibility::Inherited, input.parse()?));
            }

            // Consume a comma to continue looking for constructors
            if input.parse::<Comma>().is_err() {
                break;
            }
        }

        Ok(Self { definitions })
    }
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

        let (delim, span, buffer) = input.parse_any_delimiter()?;
        if delim != Delimiter::Bracket {
            return Err(Error::new(span.span(), "Expected enclosing brackets"))
        }

        // consume constructor specifier ex: 1, 2, 3
        loop {
            config.applications.insert(buffer.parse::<LitInt>()?.base10_parse()?);
            if buffer.parse::<Comma>().is_err() {
                break;
            }
        }

        Ok(config)
    }
}

impl Parse for FieldConfigProperty {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<Impl>().is_ok() {
            return Ok(FieldConfigProperty::Impl);
        }

        let property: Ident = input.parse()?;
        let property_name = property.to_string();
        match property_name.as_str() {
            "default" => Ok(FieldConfigProperty::Default),
            "method" => {
                let (delimiter, span, buffer) = input.parse_any_delimiter()?;

                if delimiter != Delimiter::Parenthesis {
                    return Err(Error::new(span.span(), "Expected enclosing parenthesis"))
                }

                Ok(FieldConfigProperty::Method { ident: buffer.parse()? })
            },
            "value" => {
                let (delimiter, span, buffer) = input.parse_any_delimiter()?;

                if delimiter != Delimiter::Parenthesis {
                    return Err(Error::new(span.span(), "Expected enclosing parenthesis"))
                }

                Ok(FieldConfigProperty::Value {
                    expression: proc_macro2::TokenStream::parse(&buffer)
                        .expect("Unable to convert buffer back into TokenStream")
                })
            },
            _ => Err(Error::new(
                property.span(),
                FIELD_CONFIG_ERR_MSG.replace("{prop}", &property_name)
            ))
        }
    }
}

impl ToTokens for RequiredStructField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.field_ident.to_tokens(tokens);
        tokens.append(Punct::new(':', Alone));
        self.field_type.to_tokens(tokens);
    }
}

impl ToTokens for GeneratedStructField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.field_ident;

        let token_stream = quote! {
            let #ident =
        };

        tokens.extend(token_stream);

        tokens.extend(match &self.configuration {
            FieldConfigProperty::Default => quote! { Default::default() },
            FieldConfigProperty::Method { ident } => quote! { #ident() },
            FieldConfigProperty::Value { expression } => expression.clone(),
            FieldConfigProperty::Impl => quote! { #ident.into() }
        });

        tokens.append(Punct::new(';', Alone))
    }
}


#[proc_macro_derive(ctor, attributes(ctor))]
pub fn derive_ctor(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let configuration = match try_parse_type_attributes(&derive_input.attrs) {
        Ok(config) => config,
        Err(err) => return TokenStream::from(err.to_compile_error())
    };

    match derive_input.data {
        Data::Struct(data) => create_ctor_struct_impl(
            derive_input.ident,
            derive_input.generics,
            data.fields,
            configuration
        ),
        Data::Enum(_) => TokenStream::from(Error::new(derive_input.span(), "Enums are not yet supported by ctor").to_compile_error()),
        Data::Union(_) => TokenStream::from(Error::new(derive_input.span(), "Unions are not yet supported by ctor").to_compile_error())
    }
}

fn create_ctor_struct_impl(
    ident: Ident,
    generics: Generics,
    fields: Fields,
    struct_configuration: CtorTypeConfiguration
) -> TokenStream {
    let mut required_fields = Vec::new();
    let mut generated_fields = Vec::new();

    let mut field_idents= Vec::new();

    let method_count = struct_configuration.definitions.len();
    for _i in 0..method_count {
        required_fields.push(Vec::new());
        generated_fields.push(Vec::new());
    }

    for field in fields {
        let configuration = match try_parse_field_attributes(&field.attrs) {
            Ok(config) => config,
            Err(err) => return TokenStream::from(err.to_compile_error())
        };

        let field_ident = field.ident.expect("Missing struct field name");
        field_idents.push(field_ident.clone());

        for i in 0..method_count {

            let field_ident = field_ident.clone();
            let field_type = field.ty.clone();

            match &configuration {
                None => required_fields[i].push(RequiredStructField { field_ident, field_type }),
                Some(configuration) => {
                    if matches!(configuration.property, FieldConfigProperty::Impl) {
                        generated_fields[i].push(GeneratedStructField {
                            field_ident: field_ident.clone(),
                            configuration: configuration.property.clone()
                        });
                        required_fields[i].push(RequiredStructField { field_ident,
                            field_type: syn::parse2(quote! { impl Into<#field_type> }).unwrap()
                        });
                        continue;
                    }

                    let applications = &configuration.applications;
                    if applications.is_empty() || applications.contains(&i) {
                        generated_fields[i].push(GeneratedStructField {
                            field_ident,
                            configuration: configuration.property.clone()
                        })
                    } else {
                        required_fields[i].push(RequiredStructField { field_ident, field_type })
                    }
                }
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut methods = Vec::new();
    for (i, (visibility, name)) in struct_configuration.definitions.into_iter().enumerate() {
        let method_req_fields = &required_fields[i];
        let method_gen_fields = &generated_fields[i];
        methods.push(quote! {
            #visibility fn #name(#(#method_req_fields),*) -> Self {
                #(#method_gen_fields)*
                Self { #(#field_idents),* }
            }
        })
    }


    let ts = TokenStream::from(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }
    });
    println!("Output: {}", ts);
    ts
}

fn try_parse_type_attributes(attributes: &[Attribute]) -> Result<CtorTypeConfiguration, Error> {
    for attribute in attributes {
        if attribute.path().is_ident("ctor") {
            return attribute.parse_args::<CtorTypeConfiguration>();
        }
    }
    Ok(CtorTypeConfiguration::default())
}

fn try_parse_field_attributes(attributes: &[Attribute]) -> Result<Option<FieldConfig>, Error> {
    for attribute in attributes {
        if attribute.path().is_ident("ctor") {
            return attribute.parse_args::<FieldConfig>()
                .map(|config| Some(config))
        }
    }
    Ok(None)
}