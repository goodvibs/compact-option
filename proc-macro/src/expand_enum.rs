use quote::quote;
use syn::{ItemEnum, Result};

use crate::parse::AttrArgs;

pub fn expand_enum(args: &AttrArgs, item: &ItemEnum) -> Result<proc_macro2::TokenStream> {
    let ident = &item.ident;
    let r_ty = &args.r_ty;
    let sentinel = &args.sentinel;

    Ok(quote! {
        #item
        unsafe impl const ::compact_option::CompactRepr<#r_ty> for #ident {
            const UNUSED_SENTINEL: #r_ty = #sentinel;
        }
    })
}
