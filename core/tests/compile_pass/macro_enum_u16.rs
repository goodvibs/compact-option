#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features, unused_features)]

use compact_option::CompactOption;
use compact_option_proc_macro::compact_option;

#[compact_option(repr(R = u16, sentinel = 0xFFFF))]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wide {
    A = 0,
    B = 1,
}

const _FORCE: CompactOption<u16, Wide> = CompactOption::NONE;

fn main() {}
