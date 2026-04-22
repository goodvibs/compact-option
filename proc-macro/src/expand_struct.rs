use quote::quote;
use syn::{ItemStruct, Result};

use crate::parse::AttrArgs;

pub fn expand_struct(args: &AttrArgs, item: &ItemStruct) -> Result<proc_macro2::TokenStream> {
    let ident = &item.ident;
    let r_ty = &args.r_ty;
    let sentinel = &args.sentinel;

    Ok(quote! {
        #item
        const _: () = {
            ::core::assert!(
                ::core::mem::size_of::<#ident>() == ::core::mem::size_of::<#r_ty>(),
                "transparent newtype size must match `R` (layout identity for `CompactRepr`)"
            );
            ::core::assert!(
                ::core::mem::align_of::<#ident>() == ::core::mem::align_of::<#r_ty>(),
                "transparent newtype alignment must match `R` (layout identity for `CompactRepr`)"
            );
        };
        unsafe impl const ::compact_option::CompactRepr<#r_ty> for #ident {
            const UNUSED_SENTINEL: #r_ty = #sentinel;
        }
    })
}
