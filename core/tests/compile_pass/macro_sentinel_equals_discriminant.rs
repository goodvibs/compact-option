#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features, unused_features)]

// Dumb macro: no sentinel vs discriminant check; this compiles but violates `CompactRepr` invariants.
use compact_option::CompactOption;
use compact_option_proc_macro::compact_option;

#[compact_option(repr(R = u8, sentinel = 0))]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Bad {
    A = 0,
}

const _FORCE: CompactOption<u8, Bad> = CompactOption::NONE;

fn main() {}
