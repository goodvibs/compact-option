#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features, unused_features)]

use compact_option_proc_macro::compact_option;

#[compact_option(repr(R = u8, sentinel = 0xFF))]
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct WideSlot(pub u16);

fn main() {}
