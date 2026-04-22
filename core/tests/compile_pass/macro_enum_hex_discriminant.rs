#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features, unused_features)]

use compact_option::CompactOption;
use compact_option_proc_macro::compact_option;

#[compact_option(repr(R = u8, sentinel = 0xFF))]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HexDisc {
    A = 0x00,
    B = 0x0A,
}

const _FORCE: CompactOption<u8, HexDisc> = CompactOption::NONE;

fn main() {}
