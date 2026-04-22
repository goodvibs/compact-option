use core::hash::{Hash, Hasher};

use proptest::prelude::*;
use proptest::test_runner::Config;

use crate::fixtures::{ByteSlot, Id, OptByte, OptId, OptSmall, SmallEnum};

fn arb_small_enum() -> impl Strategy<Value = SmallEnum> {
    prop_oneof![Just(SmallEnum::Var1), Just(SmallEnum::Var2)]
}

proptest! {
    // Miri runs tests with filesystem isolation; proptest's default file
    // persistence uses `getcwd` when resolving `proptest-regressions/`, which
    // Miri rejects.
    #![proptest_config(Config {
        failure_persistence: None,
        .. Config::default()
    })]
    #[test]
    fn some_try_unwrap_roundtrip(x in arb_small_enum()) {
        let o = OptSmall::some(x);
        prop_assert!(o.is_some());
        prop_assert!(!o.is_none());
        prop_assert_eq!(o.try_unwrap(), Some(x));
    }

    #[test]
    fn some_unwrap_eq(x in arb_small_enum()) {
        prop_assert_eq!(OptSmall::some(x).unwrap(), x);
    }

    #[test]
    fn some_expect_eq(x in arb_small_enum()) {
        prop_assert_eq!(OptSmall::some(x).expect("msg"), x);
    }

    #[test]
    fn map_to_discriminant_byte(x in arb_small_enum()) {
        let mapped = OptSmall::some(x).map(|v| v as u8);
        prop_assert_eq!(mapped, Some(x as u8));
    }

    #[test]
    fn and_then_some_identity(x in arb_small_enum()) {
        let o = OptSmall::some(x).and_then(|v| Some(v));
        prop_assert_eq!(o, Some(x));
    }

    #[test]
    fn and_then_none_branch(x in arb_small_enum()) {
        let o = OptSmall::some(x).and_then(|_| None::<()>);
        prop_assert_eq!(o, None);
    }

    #[test]
    fn hash_eq_for_equal_some(x in arb_small_enum()) {
        let a = OptSmall::some(x);
        let b = OptSmall::some(x);
        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        prop_assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn none_ne_some(x in arb_small_enum()) {
        prop_assert_ne!(OptSmall::NONE, OptSmall::some(x));
    }

    #[test]
    fn byte_slot_roundtrip(
        payload in any::<u8>().prop_filter("not UNUSED_SENTINEL (0xFE)", |&b| b != 0xFE)
    ) {
        let b = ByteSlot(payload);
        let o = OptByte::some(b);
        prop_assert_eq!(o.try_unwrap(), Some(b));
        prop_assert_eq!(o.unwrap(), b);
    }

    #[test]
    fn id_roundtrip(n in any::<u32>().prop_filter("not NONE sentinel", |&n| n != u32::MAX)) {
        let id = Id(n);
        let o = OptId::some(id);
        prop_assert_eq!(o.try_unwrap(), Some(id));
        prop_assert_eq!(o.unwrap(), id);
    }
}
