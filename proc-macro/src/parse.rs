use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitBool, Result, Token, Type};

/// Parsed `#[compact_option(repr(R = …, sentinel = …)[, verify_discriminants = bool])]`.
///
/// Trailing `verify_discriminants` is accepted for backward compatibility and **ignored**; the
/// macro does not inspect discriminants or emit collision checks.
pub struct AttrArgs {
    pub r_ty: Type,
    pub sentinel: Expr,
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let repr_kw: Ident = input.parse()?;
        if repr_kw != "repr" {
            return Err(syn::Error::new_spanned(
                repr_kw,
                "expected `repr` — use #[compact_option(repr(R = …, sentinel = …))]",
            ));
        }
        let inner;
        syn::parenthesized!(inner in input);
        let r_ident: Ident = inner.parse()?;
        if r_ident != "R" {
            return Err(syn::Error::new_spanned(r_ident, "expected `R`"));
        }
        inner.parse::<Token![=]>()?;
        let r_ty: Type = inner.parse()?;
        inner.parse::<Token![,]>()?;
        let sen_ident: Ident = inner.parse()?;
        if sen_ident != "sentinel" {
            return Err(syn::Error::new_spanned(sen_ident, "expected `sentinel`"));
        }
        inner.parse::<Token![=]>()?;
        let sentinel: Expr = inner.parse()?;
        if inner.peek(Token![,]) {
            inner.parse::<Token![,]>()?;
        }
        if !inner.is_empty() {
            return Err(syn::Error::new(
                inner.span(),
                "unexpected tokens in `repr(...)` — only `R = …` and `sentinel = …` are allowed inside the parentheses",
            ));
        }

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if key == "verify_discriminants" {
                let _: LitBool = input.parse()?;
            } else {
                return Err(syn::Error::new_spanned(
                    key,
                    "unknown `#[compact_option]` flag (supported for parsing only: `verify_discriminants`, ignored)",
                ));
            }
        }

        Ok(AttrArgs { r_ty, sentinel })
    }
}
