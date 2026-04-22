# compact-option

`CompactOption<R, T>` is a niche-packing optional: it stores either `NONE` or a `Some(T)` payload in exactly as much memory as raw `R`, where `T: Copy` via the unsafe `CompactRepr` contract.
It is intended for `R` types with spare bit patterns; the primary use case is `#[repr(u8)]` enums with fewer than 256 variants.

See the [repository](https://github.com/goodvibs/compact-option) for examples and the optional `macros` feature.

Licensed under the MIT license (see the `LICENSE` file in the repository root).
