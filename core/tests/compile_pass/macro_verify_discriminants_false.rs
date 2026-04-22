#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features, unused_features)]

use compact_option::CompactOption;
use compact_option_proc_macro::compact_option;

// `verify_discriminants` is accepted for compatibility and ignored (macro does not check discriminants).
#[compact_option(repr(R = u8, sentinel = 0), verify_discriminants = false)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptedOut {
    A = 0,
}

const _FORCE: CompactOption<u8, OptedOut> = CompactOption::NONE;

fn main() {}
