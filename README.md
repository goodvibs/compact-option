# compact-option

[![Crates.io](https://img.shields.io/crates/v/compact-option)](https://crates.io/crates/compact-option)

[![Crates.io](https://img.shields.io/crates/v/compact-option-proc-macro)](https://crates.io/crates/compact-option-proc-macro)

Workspace for **`compact-option`**: a `Copy` optional that uses exactly as much memory as raw `R` while storing `NONE` or `Some(T)`, under the unsafe `CompactRepr` contract.
It is intended for raw representations `R` with spare bit patterns, with `#[repr(u8)]` enums that have fewer than 256 variants as the primary use case.

## Crates

| Crate | Role |
| --- | --- |
| [`compact-option`](core/) | struct `CompactOption<R, T>`, trait `CompactRepr` |
| [`compact-option-proc-macro`](proc-macro/) | `#[compact_option(repr(R = …, sentinel = …))]` code generator |

Enable the **`macros`** feature on `compact-option` to re-export the attribute, or depend on the proc-macro crate directly.

## Requirements

- **Nightly Rust** (see [`rust-toolchain.toml`](rust-toolchain.toml)); the core crate uses unstable features.

## Quick start (manual `CompactRepr`)

```rust
use compact_option::{CompactOption, CompactRepr};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Letter {
    A = 0,
    B = 1,
}

unsafe impl const CompactRepr<u8> for Letter {
    const UNUSED_SENTINEL: u8 = 0xFF;
}

let none = CompactOption::<u8, Letter>::NONE;
assert!(none.is_none());

let some = CompactOption::some(Letter::A);
assert_eq!(some.try_unwrap(), Some(Letter::A));
```

## Quick start (macro)

```rust
use compact_option::CompactOption;
use compact_option_proc_macro::compact_option;

#[compact_option(repr(R = u8, sentinel = 0xFF))]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Letter {
    A = 0,
    B = 1,
}

const _CHECK: CompactOption<u8, Letter> = CompactOption::NONE;
```

The macro is intentionally minimal: it does **not** validate enum discriminants vs `sentinel`, `#[repr]`, or `R` consistency for enums. For structs it emits `size_of` / `align_of` checks against `R`. See the proc-macro rustdocs for details.

## Safety model (read this)

- `CompactRepr` is **`unsafe`**: you must ensure the sentinel never aliases a valid `Some` encoding, and that transmutes between `R` and `T` are sound under the crate’s assumptions.
- If those invariants break, `NONE` / `Some` can collide logically even though the code still compiles.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo test --workspace --doc
```

Miri (CI also runs doctests under Miri):

```bash
yes | cargo miri setup
cargo miri nextest run --workspace -j 2
cargo miri test --workspace --doc
```

## License

MIT — see [`LICENSE`](LICENSE).
