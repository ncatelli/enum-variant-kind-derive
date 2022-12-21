use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DataEnum, DeriveInput, Ident};

/// Stores all metadata necessary for a given Ident.
struct VariantMetadata {
    _span: Span,
    ident: Ident,
}

impl VariantMetadata {
    fn new(span: Span, ident: Ident) -> Self {
        Self { _span: span, ident }
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

            Ok(VariantMetadata::new(span, ident))
        })
        .collect::<Result<_, _>>()
        .map(|enriched_token_variants| {
            Variants::new(input_span, tok_enum_name, enriched_token_variants)
        })
}

fn codegen(variants: Variants) -> syn::Result<TokenStream> {
    let enum_ident = &variants.enum_ident;
    let enum_kind_ident = TokenStream::from_str(&format!("{}Kind", enum_ident))?;

    let variant_strs = variants
        .variant_metadata
        .iter()
        .map(|var| var.ident.to_string())
        .collect::<Vec<_>>();

    let joined_variants = TokenStream::from_str(&variant_strs.join(",\n"))?;

    let enum_kind_stream = quote! {
        /// A enum representing a copy of all variants representable for the
        /// type `#enum_ident`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum #enum_kind_ident {
           #joined_variants
        }
    };

    Ok(enum_kind_stream)
}

#[proc_macro_derive(EnumVariantKind)]
pub fn generate_variant_iter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    parse(input)
        .and_then(codegen)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
