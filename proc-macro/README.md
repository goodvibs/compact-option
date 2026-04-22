# compact-option-proc-macro

Procedural-macro companion to [`compact-option`](https://crates.io/crates/compact-option): `#[compact_option(repr(R = …, sentinel = …))]`.
It targets the same core use case: a niche-packing optional that uses exactly as much memory as raw `R`, intended for `R` values with spare bit patterns (primarily `#[repr(u8)]` enums with fewer than 256 variants).

See the [repository](https://github.com/goodvibs/compact-option) for usage and safety notes.

Licensed under the MIT license (see the `LICENSE` file in the repository root).
