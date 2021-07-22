//! This library implements the derivation from GrrVertex.  
//!
//! Deriving this trait allows one to automatically create attributes
//! descriptions to pass on to GrrDevice::create_vertex_array, based
//! on the fields defined in the structure.
//!
//! The static `attribs` method provides to these attributes, based on
//! a binding index and initial location index.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use std::iter::Extend;
use syn::spanned::Spanned;

#[proc_macro_derive(GrrVertex)]
pub fn grr_vertex_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let ast_span = ast.span();

    let data = if let syn::Data::Struct(d) = ast.data {
        d
    } else {
        return quote_spanned! {
            ast_span=>
            compile_error!("GrrVertex can only be auto-derived on a struct.")
        }
        .into();
    };
    let struct_ident = &ast.ident;

    let mut res = TokenStream::new();
    let mut attrib_offset = proc_macro2::TokenStream::new();

    // make sure we have named fields.
    let _fields = if let syn::Fields::Named(ref fields) = data.fields {
        fields
    } else {
        return quote_spanned! {
            ast_span=>
            compile_error!("GrrVertex can only be auto-derived on a struct with named fields.")
        }
        .into();
    };

    for (i, field) in data.fields.iter().enumerate() {
        let span = field.span();
        let ty = &field.ty;

        // Assert that each field has a type that implements the GrrVertexField trait.
        let assert_trait_ident = match &field.ident {
            Some(x) => format_ident!("_AssertImpl_{}_{}", ast.ident, x),
            None => format_ident!("_AssertImpl_{}_{}", ast.ident, i),
        };
        res.extend( proc_macro::TokenStream::from(quote_spanned!{span=>
	    #[allow(non_camel_case_types)] struct #assert_trait_ident where #ty: grr_util::vertex::GrrVertexField {}
	}));

        attrib_offset.extend(field_attrib(struct_ident, &field));
    }

    res.extend(proc_macro::TokenStream::from(quote! {
    impl grr_util::vertex::GrrVertex for #struct_ident {
        fn attribs(binding: u32, location_start: u32) -> Vec<grr::VertexAttributeDesc> {
        let mut offset: u32 = 0;
        let mut attrib_props = std::vec::Vec::new();
        #attrib_offset

        attrib_props
            .iter()
            .enumerate()
            .map(|(i, (vf, offset))|
             grr::VertexAttributeDesc {
                 binding,
                 format: *vf,
                 location: location_start + i as u32,
                 offset: *offset as _,
             })
            .collect()
        }
    }
    }));
    res
}

/// Return a collection of token stream of attributes for a particular field.
fn field_attrib(struct_ident: &syn::Ident, f: &syn::Field) -> proc_macro2::TokenStream {
    let mut res = proc_macro2::TokenStream::new();
    let field_ident = &f.ident;
    let field_ty = &f.ty;
    res.extend::<proc_macro2::TokenStream>(quote! {
    offset = memoffset::offset_of!(#struct_ident, #field_ident) as _;
    let (vf, nc) = <#field_ty as grr_util::vertex::GrrVertexField>::format();
    for i in 0..nc {
        attrib_props.push((vf, offset));
        offset += <#field_ty as grr_util::vertex::GrrVertexField>::COMPONENT_SIZE;
    }
    });

    res
}
