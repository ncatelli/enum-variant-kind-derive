use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DataEnum, DeriveInput, Fields, Ident};

/// Stores all metadata necessary for a given Ident.
struct VariantMetadata {
    _span: Span,
    ident: Ident,
    fields: Fields,
}

impl VariantMetadata {
    fn new(span: Span, ident: Ident, fields: Fields) -> Self {
        Self {
            _span: span,
            ident,
            fields,
        }
    }
}

impl From<VariantMetadata> for Ident {
    fn from(value: VariantMetadata) -> Self {
        value.ident
    }
}

struct Variants {
    _span: Span,
    enum_ident: Ident,
    variant_metadata: Vec<VariantMetadata>,
}

impl Variants {
    fn new(span: Span, enum_ident: Ident, variant_metadata: Vec<VariantMetadata>) -> Self {
        Self {
            _span: span,
            enum_ident,
            variant_metadata,
        }
    }
}

fn parse(input: DeriveInput) -> Result<Variants, syn::Error> {
    let input_span = input.span();
    let tok_enum_name = input.ident;
    let enum_variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            return Err(syn::Error::new(
                input_span,
                "derive macro only works on enums",
            ))
        }
    };

    enum_variants
        .into_iter()
        .map(|variant| {
            let span = variant.span();
            let ident = variant.ident;
            let fields = variant.fields;

            Ok(VariantMetadata::new(span, ident, fields))
        })
        .collect::<Result<_, _>>()
        .map(|enriched_token_variants| {
            Variants::new(input_span, tok_enum_name, enriched_token_variants)
        })
}

fn codegen(variants: Variants) -> syn::Result<TokenStream> {
    let enum_ident = &variants.enum_ident;
    let enum_kind_ident = TokenStream::from_str(&format!("{}Kind", enum_ident))?;

    let joined_variants = variants
        .variant_metadata
        .iter()
        .map(|var| format!("{},\n", var.ident))
        .map(|s| TokenStream::from_str(&s))
        .collect::<Result<TokenStream, _>>()?;

    let enum_kind_stream = quote! {
        /// A enum representing a copy of all variants representable for the
        /// type `#enum_ident`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum #enum_kind_ident {
           #joined_variants
        }
    };

    let enum_kind_conversion_impl = variants
        .variant_metadata
        .iter()
        .map(|var| match &var.fields {
            Fields::Named(_) => format!(
                "{}::{} {{ .. }} => {}::{},\n",
                enum_ident, var.ident, enum_kind_ident, var.ident
            ),
            Fields::Unnamed(f) => {
                let field_cnt = f.unnamed.len();
                let field_underscores = ["_"]
                    .into_iter()
                    .cycle()
                    .take(field_cnt)
                    .collect::<Vec<_>>()
                    .join(", ");

                format!(
                    "{}::{}({}) => {}::{},\n",
                    enum_ident, var.ident, field_underscores, enum_kind_ident, var.ident
                )
            }
            Fields::Unit => format!(
                "{}::{} => {}::{},\n",
                enum_ident, var.ident, enum_kind_ident, var.ident
            ),
        })
        .map(|s| TokenStream::from_str(&s))
        .collect::<Result<TokenStream, _>>()?;

    let conversion_stream = quote! {
        impl #enum_ident {
            /// Returns the corresponding `Copy`-able variant kind for a given enum variant.
            pub fn as_variant_kind(&self) -> #enum_kind_ident {
                match self {
                    #enum_kind_conversion_impl
                }
            }
        }

        impl From<#enum_ident> for #enum_kind_ident {
            fn from(src: #enum_ident) -> Self {
                src.as_variant_kind()
            }
        }
    };

    Ok(enum_kind_stream
        .into_iter()
        .chain(conversion_stream.to_token_stream())
        .collect())
}

#[proc_macro_derive(EnumVariantKind)]
pub fn generate_variant_iter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    parse(input)
        .and_then(codegen)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
