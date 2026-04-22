#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features, unused_features)]

use compact_option::CompactOption;
use compact_option_proc_macro::compact_option;

pub type InnerByte = u8;

#[compact_option(repr(R = u8, sentinel = 0xFE))]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AliasSlot(pub InnerByte);

const _FORCE: CompactOption<u8, AliasSlot> = CompactOption::NONE;

fn main() {}
