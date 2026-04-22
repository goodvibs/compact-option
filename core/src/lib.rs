#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![feature(const_cmp)]
#![feature(transmutability)]
#![allow(incomplete_features)]

//! Niche-packing optional: [`CompactOption<R, T>`][CompactOption] uses exactly as much memory as
//! raw `R` to store either [`CompactOption::NONE`] or a `Some(T)` payload, where `T: Copy` via the
//! unsafe [`CompactRepr`] contract.
//!
//! Intended for raw representations `R` with spare bit patterns. Primary use case:
//! `#[repr(u8)]` enums with fewer than 256 variants.
//!
//! - [`CompactOption`] is the safe-ish wrapper API (transmute-based; see docs and Miri).
//! - Implement [`CompactRepr`] manually, or enable the **`macros`** feature for
//!   `#[compact_option(repr(R = …, sentinel = …))]` (see the `compact-option-proc-macro` crate).
//!
//! **Toolchain:** this crate pins a nightly toolchain via `rust-toolchain.toml` and relies on
//! unstable features.

use core::marker::PhantomData;
use core::mem::{Assume, TransmuteFrom};

const TRANSMUTATION_ASSUMPTION: Assume = Assume {
    alignment: false,
    lifetimes: false,
    safety: true,
    validity: true,
};

mod __layout {
    use core::marker::PhantomData;
    use core::mem::{align_of, size_of};

    use crate::CompactRepr;

    pub(crate) struct LayoutInvariant<R, T: ?Sized>(PhantomData<(R, T)>);

    impl<R, T> LayoutInvariant<R, T>
    where
        T: CompactRepr<R>,
    {
        pub(crate) const CHECK: () = {
            assert!(size_of::<T>() == size_of::<R>());
            assert!(align_of::<T>() == align_of::<R>());
        };
    }
}

/// # Safety
/// Implementors must guarantee:
/// 1. For every `T` value stored via [`CompactOption::some`], the transmuted
///    `R` bit pattern must not equal [`CompactRepr::UNUSED_SENTINEL`].
/// 2. Non-sentinel `R` values used as `Some` payloads must be sound to transmute
///    back to `T` under the same `Assume` bundle used by [`CompactOption`] for
///    `TransmuteFrom` between `R` and `T`.
/// 3. If you care about logical round-tripping, transmuting that raw value back
///    yields an equivalent `T`.
///
/// # Choosing `UNUSED_SENTINEL`
///
/// Pick an `R` value that is **not** the transmuted bit pattern of any `T` you
/// will ever store as `Some`. If the sentinel aliases a valid `Some` encoding,
/// `NONE` and `Some` collide and the type becomes logically unusable.
///
/// # Validation
///
/// After changing an `unsafe impl CompactRepr`, run `cargo miri test` (or your
/// project’s Miri CI) to exercise transmute-based paths under the stacked
/// borrows / provenance model.
///
/// ## Procedural macro
///
/// Enable the **`macros`** crate feature for a re-exported `#[compact_option(...)]`
/// attribute, or depend on the **`compact-option-proc-macro`** crate directly.
///
/// The `#[compact_option(repr(R = …, sentinel = …))]` macro only emits `unsafe impl CompactRepr`;
/// it does **not** validate `#[repr]`, discriminants, or sentinel collisions. Structs additionally
/// get `size_of` / `align_of` checks against `R`. See the proc-macro crate’s rustdoc and Miri for
/// safety review.
pub const unsafe trait CompactRepr<R>: Copy + Sized {
    /// Raw value reserved for [`CompactOption::NONE`].
    ///
    /// # Safety (encoding)
    ///
    /// This bit pattern must **never** equal the transmuted `R` encoding of any `T` you store via
    /// [`CompactOption::some`]. If it does, `NONE` and `Some` collide: [`CompactOption::is_none`]
    /// may return `true` for a value you constructed with `some`, and [`CompactOption::try_unwrap`]
    /// returns `None`.
    const UNUSED_SENTINEL: R;
}

/// When built with the `macros` feature, re-exports the `#[compact_option(...)]` attribute.
#[cfg(feature = "macros")]
pub use compact_option_proc_macro::compact_option;

/// Niche-packing optional: stores either [`Self::NONE`] or a `Some(T)` payload in exactly as much
/// memory as raw `R`. `T` must be [`Copy`] (via the [`CompactRepr`] contract); the wrapper itself
/// is `Copy` whenever `R` and `T` are.
///
/// ## Layout checks
///
/// `R` and `T` must have identical size and alignment. The same layout assertions run when
/// evaluating [`Self::NONE`] in a `const` context and when calling [`Self::some`] (so `some`
/// cannot silently skip layout validation). A plain `let _ = Self::NONE` in non-const code may
/// not const-evaluate [`Self::NONE`]; prefer `const { CompactOption::<R, T>::NONE }` or similar
/// if you need the check guaranteed at compile time.
///
/// ```compile_fail
/// use compact_option::{CompactOption, CompactRepr};
///
/// #[derive(Clone, Copy)]
/// #[repr(C)]
/// struct Pair(u8, u8);
///
/// unsafe impl CompactRepr<u8> for Pair {
///     const UNUSED_SENTINEL: u8 = 0xFF;
/// }
///
/// const _FORCE_LAYOUT: CompactOption<u8, Pair> = CompactOption::NONE;
///
/// fn main() {}
/// ```
///
/// [`CompactRepr`] requires a `Copy` payload:
///
/// ```compile_fail
/// use compact_option::{CompactOption, CompactRepr};
///
/// #[derive(Clone)]
/// struct Opaque(u8);
///
/// unsafe impl CompactRepr<u8> for Opaque {
///     const UNUSED_SENTINEL: u8 = 0xFF;
/// }
///
/// fn main() {
///     let _ = CompactOption::<u8, Opaque>::NONE;
/// }
/// ```
#[repr(transparent)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct CompactOption<R, T: CompactRepr<R>> {
    raw_value: R,
    _marker: PhantomData<T>,
}

impl<R: Copy, T: CompactRepr<R> + Copy> Copy for CompactOption<R, T> {}

impl<R, T> CompactOption<R, T>
where
    R: Copy + PartialEq,
    T: CompactRepr<R>,
{
    /// Sentinel-backed empty value: the stored `R` equals [`CompactRepr::UNUSED_SENTINEL`].
    ///
    /// Layout of `T` and `R` is checked here (see struct-level **Layout checks**). Using `NONE` in
    /// a `const` context ensures that check runs; a plain `let _ = Self::NONE` in non-const `main`
    /// may not const-evaluate it.
    pub const NONE: Self = {
        let () = __layout::LayoutInvariant::<R, T>::CHECK;
        Self {
            raw_value: T::UNUSED_SENTINEL,
            _marker: PhantomData,
        }
    };

    /// Construct a `Some` by transmuting `T` → `R` using the same `Assume` bundle as
    /// [`try_unwrap`](Self::try_unwrap) / [`unwrap_unchecked`](Self::unwrap_unchecked).
    ///
    /// Layout of `T` and `R` is asserted here (same as [`Self::NONE`]).
    ///
    /// # Sentinel collisions
    ///
    /// If `value`’s transmuted bit pattern equals [`CompactRepr::UNUSED_SENTINEL`], this value is
    /// indistinguishable from [`Self::NONE`]: [`is_none`](Self::is_none) may be `true` and
    /// [`try_unwrap`](Self::try_unwrap) returns `None`. A correct [`CompactRepr`] must rule that
    /// out for all stored `T`.
    ///
    /// Not `const` because `TransmuteFrom::transmute` is not a `const fn` on this toolchain.
    pub fn some(value: T) -> Self
    where
        T: CompactRepr<R>,
        R: TransmuteFrom<T, { TRANSMUTATION_ASSUMPTION }>,
    {
        let () = __layout::LayoutInvariant::<R, T>::CHECK;
        Self {
            raw_value: unsafe {
                <R as TransmuteFrom<T, { TRANSMUTATION_ASSUMPTION }>>::transmute(value)
            },
            _marker: PhantomData,
        }
    }

    /// Returns `true` when this value encodes [`Self::NONE`] (raw equals [`CompactRepr::UNUSED_SENTINEL`]).
    pub const fn is_none(self) -> bool
    where
        R: [const] PartialEq,
    {
        self.raw_value == T::UNUSED_SENTINEL
    }

    /// Returns `true` when this value encodes `Some` (raw differs from [`CompactRepr::UNUSED_SENTINEL`]).
    pub const fn is_some(self) -> bool
    where
        R: [const] PartialEq,
    {
        !self.is_none()
    }

    /// If this is `Some`, transmute the raw `R` back to `T`. If raw equals [`CompactRepr::UNUSED_SENTINEL`],
    /// returns `None` (including sentinel-collision cases described on [`Self::some`]).
    pub fn try_unwrap(self) -> Option<T>
    where
        T: TransmuteFrom<R, { TRANSMUTATION_ASSUMPTION }>,
    {
        if self.raw_value == T::UNUSED_SENTINEL {
            None
        } else {
            debug_assert!(
                self.raw_value != T::UNUSED_SENTINEL,
                "CompactOption::try_unwrap: raw must differ from UNUSED_SENTINEL"
            );
            // SAFETY: `CompactRepr` requires non-sentinel `R` values used as
            // `Some` to transmute to a bit-valid `T`.
            Some(unsafe { self.unwrap_unchecked() })
        }
    }

    /// Like [`Option::unwrap`]: returns the payload or panics if this is [`Self::NONE`].
    pub fn unwrap(self) -> T
    where
        T: TransmuteFrom<R, { TRANSMUTATION_ASSUMPTION }>,
    {
        match self.try_unwrap() {
            Some(t) => t,
            None => panic!("called `CompactOption::unwrap` on a `NONE` value"),
        }
    }

    /// Like [`Option::expect`]: returns the payload or panics with `msg` if this is [`Self::NONE`].
    pub fn expect(self, msg: &str) -> T
    where
        T: TransmuteFrom<R, { TRANSMUTATION_ASSUMPTION }>,
    {
        match self.try_unwrap() {
            Some(t) => t,
            None => panic!("{msg}"),
        }
    }

    /// If `Some`, applies `f` to the payload; if [`Self::NONE`], returns `None` without calling `f`.
    pub fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
        T: TransmuteFrom<R, { TRANSMUTATION_ASSUMPTION }>,
    {
        self.try_unwrap().map(f)
    }

    /// If `Some`, runs `f` on the payload; if [`Self::NONE`], returns `None` without calling `f`.
    pub fn and_then<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> Option<U>,
        T: TransmuteFrom<R, { TRANSMUTATION_ASSUMPTION }>,
    {
        self.try_unwrap().and_then(f)
    }

    /// # Safety
    /// `self` must not be `NONE`, and `self.raw_value` must satisfy the
    /// `CompactRepr` encoding invariant for `T`.
    pub unsafe fn unwrap_unchecked(self) -> T
    where
        T: TransmuteFrom<R, { TRANSMUTATION_ASSUMPTION }>,
    {
        debug_assert!(
            self.raw_value != T::UNUSED_SENTINEL,
            "CompactOption::unwrap_unchecked: self must not be NONE (raw != UNUSED_SENTINEL)"
        );
        unsafe { <T as TransmuteFrom<R, { TRANSMUTATION_ASSUMPTION }>>::transmute(self.raw_value) }
    }
}

#[cfg(test)]
mod fixtures {
    use crate::{CompactOption, CompactRepr};

    /// `repr(u8)` payload backed by `u8` storage; sentinel `0xFF`.
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) enum SmallEnum {
        Var1 = 0,
        Var2 = 1,
    }

    unsafe impl const CompactRepr<u8> for SmallEnum {
        const UNUSED_SENTINEL: u8 = 0xFF;
    }

    pub(crate) type OptSmall = CompactOption<u8, SmallEnum>;

    /// `repr(transparent)` single-byte struct (same pattern as a newtype over `u8`).
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct ByteSlot(pub u8);

    unsafe impl const CompactRepr<u8> for ByteSlot {
        const UNUSED_SENTINEL: u8 = 0xFE;
    }

    pub(crate) type OptByte = CompactOption<u8, ByteSlot>;

    /// Non-scalar raw `R`: transparent `u32` handle, payload is another `u32` newtype.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct Handle(pub u32);

    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct Id(pub u32);

    unsafe impl const CompactRepr<Handle> for Id {
        const UNUSED_SENTINEL: Handle = Handle(u32::MAX);
    }

    pub(crate) type OptId = CompactOption<Handle, Id>;

    /// Sentinel equals a valid discriminant: `NONE` collides with `some(A)`.
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum BadSentinel {
        A = 0,
    }

    unsafe impl const CompactRepr<u8> for BadSentinel {
        const UNUSED_SENTINEL: u8 = 0;
    }

    pub(crate) type OptBad = CompactOption<u8, BadSentinel>;

    pub(crate) const NONE_IS_NONE: bool = OptSmall::NONE.is_none();
    pub(crate) const NONE_NOT_SOME: bool = !OptSmall::NONE.is_some();
}

#[cfg(test)]
mod proptests;

#[cfg(test)]
use core::hash::{Hash, Hasher};

#[cfg(test)]
use fixtures::{BadSentinel, ByteSlot, Id, OptBad, OptByte, OptId, OptSmall, SmallEnum};

#[cfg(test)]
#[test]
fn const_predicates_on_none() {
    const { assert!(fixtures::NONE_IS_NONE) };
    const { assert!(fixtures::NONE_NOT_SOME) };
    assert!(OptSmall::some(SmallEnum::Var1).is_some());
    assert!(!OptSmall::some(SmallEnum::Var1).is_none());
    assert!(OptSmall::some(SmallEnum::Var2).is_some());
    assert!(!OptSmall::some(SmallEnum::Var2).is_none());
}

#[cfg(test)]
#[test]
fn repr_u8_enum_roundtrip_and_combinators() {
    let foo = OptSmall::some(SmallEnum::Var1);
    assert_eq!(foo.raw_value, SmallEnum::Var1 as u8);
    assert_eq!(foo.try_unwrap(), Some(SmallEnum::Var1));
    assert_eq!(OptSmall::some(SmallEnum::Var1).unwrap(), SmallEnum::Var1);

    let bar = OptSmall::some(SmallEnum::Var2);
    assert_eq!(bar.map(|x| x as u8), Some(1u8));
    assert_eq!(bar.and_then(Some), Some(SmallEnum::Var2));
    assert_eq!(bar.and_then(|_| None::<()>), None);

    assert_eq!(OptSmall::NONE.try_unwrap(), None);
    assert_eq!(
        OptSmall::some(SmallEnum::Var1).expect("some"),
        SmallEnum::Var1
    );

    unsafe {
        assert_eq!(
            OptSmall::some(SmallEnum::Var2).unwrap_unchecked(),
            SmallEnum::Var2
        );
    }
}

#[cfg(test)]
#[test]
fn map_and_then_skip_closure_on_none() {
    assert_eq!(OptSmall::NONE.map::<(), _>(|_| panic!("map on NONE")), None);
    assert_eq!(
        OptSmall::NONE.and_then::<(), _>(|_| panic!("and_then on NONE")),
        None
    );
}

#[cfg(test)]
#[test]
fn transparent_struct_payload_roundtrip() {
    let b = ByteSlot(7);
    let o = OptByte::some(b);
    assert_eq!(o.try_unwrap(), Some(ByteSlot(7)));
    assert_eq!(o.unwrap(), ByteSlot(7));
}

#[cfg(test)]
#[test]
fn non_integer_handle_roundtrip() {
    let id = Id(42);
    let o = OptId::some(id);
    assert_eq!(o.try_unwrap(), Some(Id(42)));
    assert_eq!(o.unwrap().0, 42);
}

#[cfg(test)]
#[test]
fn sentinel_collision_some_equals_none() {
    let none = OptBad::NONE;
    let some_a = OptBad::some(BadSentinel::A);
    assert_eq!(none.raw_value, some_a.raw_value);
    assert_eq!(none, some_a);
    assert!(none.is_none());
    assert!(!some_a.is_some());
    assert_eq!(some_a.try_unwrap(), None);
}

#[cfg(test)]
#[test]
fn derives_clone_partial_eq_hash_debug() {
    assert_eq!(OptSmall::NONE, OptSmall::NONE);
    assert_eq!(
        OptSmall::some(SmallEnum::Var1),
        OptSmall::some(SmallEnum::Var1)
    );
    assert_ne!(
        OptSmall::some(SmallEnum::Var1),
        OptSmall::some(SmallEnum::Var2)
    );

    let a = OptSmall::some(SmallEnum::Var1);
    let b = a;
    assert_eq!(a, b);
    assert_eq!(a.clone(), b);
    assert_eq!(OptSmall::NONE.clone(), OptSmall::NONE);
    assert_ne!(a, OptSmall::NONE);
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    a.hash(&mut h1);
    b.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
    let s = format!("{a:?}");
    assert!(s.contains("CompactOption"));
}

#[cfg(test)]
#[test]
#[should_panic(expected = "called `CompactOption::unwrap` on a `NONE` value")]
fn none_unwrap_panics() {
    let _ = OptSmall::NONE.unwrap();
}

#[cfg(test)]
#[test]
#[should_panic(expected = "empty")]
fn none_expect_panics() {
    let _ = OptSmall::NONE.expect("empty");
}

/// `unwrap_unchecked` on `NONE` is UB for `SmallEnum` + `0xFF` sentinel; run
/// `cargo miri test -- --ignored` to let Miri flag it.
#[cfg(test)]
#[test]
#[ignore = "undefined behavior; run under Miri with --ignored"]
fn miri_ub_unwrap_unchecked_on_none() {
    unsafe {
        let _ = OptSmall::NONE.unwrap_unchecked();
    }
}
