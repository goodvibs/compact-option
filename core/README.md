# compact-option

`CompactOption` stores either a `Copy` sentinel (`NONE`) or a `Some` payload in a raw value `R`, using exactly as much memory as `R` under the unsafe `CompactRepr` contract.
It is intended for `R` types with spare bit patterns; the primary use case is `#[repr(u8)]` enums with fewer than 256 variants.

See the [repository](https://github.com/goodvibs/compact-option) for examples and the optional `macros` feature.

Licensed under the MIT license (see the `LICENSE` file in the repository root).
