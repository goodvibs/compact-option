//! Attribute macro `#[compact_option(repr(R = …, sentinel = …))]` for emitting
//! `CompactRepr` for enums and structs.
//!
//! Used with `CompactOption` to encode `NONE`/`Some(T)` in exactly as much memory as raw `R`.
//! Intended for `R` types with spare bit patterns, primarily `#[repr(u8)]` enums with fewer than
//! 256 variants.
//!
//! # What this macro does (and does not)
//!
//! **Does:** parses `R` and `sentinel`, re-emits your item, and appends `unsafe impl const CompactRepr<R>`.
//! For structs it also emits `const` `size_of` / `align_of` checks against `R` so layout mismatches
//! fail in rustc const-eval (not token comparison).
//!
//! **Does not:** read `#[repr(...)]`, compute enum discriminants, compare sentinel to discriminants,
//! or validate `repr(transparent)` / field count. Those invariants are the implementer’s responsibility
//! per the `CompactRepr` safety contract; use tests and Miri.
//!
//! A trailing `, verify_discriminants = …` after `repr(...)` is accepted for compatibility and ignored.
//!
//! # Examples
//!
//! ## `repr(u8)` enum
//!
//! ```ignore
//! use compact_option_proc_macro::compact_option;
//!
//! #[compact_option(repr(R = u8, sentinel = 0xFF))]
//! #[repr(u8)]
//! #[derive(Clone, Copy)]
//! pub enum Letter {
//!     A = 0,
//!     B = 1,
//! }
//! ```
//!
//! ## `repr(transparent)` newtype
//!
//! ```ignore
//! use compact_option_proc_macro::compact_option;
//!
//! #[compact_option(repr(R = u8, sentinel = 0xFE))]
//! #[repr(transparent)]
//! #[derive(Clone, Copy)]
//! pub struct ByteSlot(pub u8);
//! ```
//!
//! # Safety
//!
//! This macro only emits `unsafe impl const CompactRepr` plus struct layout asserts. It does **not**
//! prove transmute soundness. Use Miri and the `CompactRepr` documentation.

use proc_macro::TokenStream;
use syn::{Item, parse_macro_input};

mod expand_enum;
mod expand_struct;
mod parse;

use parse::AttrArgs;

/// `#[compact_option(repr(R = <type>, sentinel = <expr>))]` on an enum or struct.
///
/// Optional trailing `, verify_discriminants = <bool>` is parsed and ignored.
///
/// # Limitations
///
/// - No analysis of enum discriminants or whether `sentinel` collides with them.
/// - No verification that `R` matches the enum’s integer `repr` or the newtype’s field type.
/// - For structs, only the emitted `size_of` / `align_of` checks tie layout to `R`.
#[proc_macro_attribute]
pub fn compact_option(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttrArgs);
    let ast: Item = match syn::parse(item.clone()) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = match &ast {
        Item::Enum(e) => expand_enum::expand_enum(&args, e),
        Item::Struct(s) => expand_struct::expand_struct(&args, s),
        _ => Err(syn::Error::new_spanned(
            &ast,
            "#[compact_option] only supports enums and structs",
        )),
    };

    match expanded {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
