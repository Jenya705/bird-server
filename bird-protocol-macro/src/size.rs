use std::env::var;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields, Type};
use crate::shared::{FieldAttributes, ObjectAttributes, parse_attributes};

pub fn impl_derive(item: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let item: DeriveInput = syn::parse(item)?;
    let DeriveInput {
        ident,
        generics,
        attrs,
        data,
        ..
    } = item;
    let object_attributes: ObjectAttributes = parse_attributes(&attrs, "bp")?;
    let size = match data {
        Data::Struct(data_struct) => {
            let (min, max) = fields_size(data_struct.fields)?;
            quote! { (#min .. #max) }
        }
        Data::Enum(data_enum) => {
            let mut min_variants_size = Vec::new();
            let mut max_variants_size = Vec::new();
            for variant in data_enum.variants {
                let (min_variant_size, max_variant_size) = fields_size(variant.fields)?;
                min_variants_size.push(min_variant_size);
                max_variants_size.push(max_variant_size);
            }
            let key_ty = object_attributes.key_variant
                .or_else(|| object_attributes.key_ty)
                .ok_or_else(|| syn::Error::new(Span::call_site(), "You must set ty or variant for key of your enum"))?;
            let min_key = min_size_ts(&key_ty);
            let max_key = max_size_ts(&key_ty);
            quote! { (
                bird_protocol::__private::add_u32_without_overflow_array([
                    #min_key,
                    bird_protocol::__private::min_u32_array([#(#min_variants_size,)*]),
                ])
                ..
                bird_protocol::__private::add_u32_without_overflow_array([
                    #max_key,
                    bird_protocol::__private::max_u32_array([#(#max_variants_size,)*]),
                ])
            ) }
        }
        Data::Union(_) => return Err(syn::Error::new(Span::mixed_site(), "Union type is not supported")),
    };
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics bird_protocol::ProtocolSize for #ident #type_generics #where_clause {
            const SIZE: std::ops::Range<u32> = #size;
        }
    })
}

pub fn fields_size(fields: Fields) -> syn::Result<(TokenStream, TokenStream)> {
    let mut min_size_types = Vec::new();
    let mut max_size_types = Vec::new();
    for field in fields {
        let field_attributes: FieldAttributes = parse_attributes(&field.attrs, "bp")?;
        let ty = field_attributes.variant.unwrap_or_else(|| field.ty.into_token_stream());
        min_size_types.push(min_size_ts(&ty));
        max_size_types.push(max_size_ts(&ty));
    }
    Ok((
        quote! { bird_protocol::__private::add_u32_without_overflow_array([#(#min_size_types,)*]) },
        quote! { bird_protocol::__private::add_u32_without_overflow_array([#(#max_size_types,)*]) }
    ))
}

pub fn min_size_ts(ty: &impl ToTokens) -> TokenStream {
    quote! { <#ty as bird_protocol::ProtocolSize>::SIZE.start }
}

pub fn max_size_ts(ty: &impl ToTokens) -> TokenStream {
    quote! { <#ty as bird_protocol::ProtocolSize>::SIZE.end }
}