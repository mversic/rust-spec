//! Field order and fold grouping must not change an aggregate classification.
use core::ops::Add;
use rust_spec::{
    Gt, Stable, Unstable, Zero,
    drop::{AutoDrop, CustomDrop, NoDrop},
    layout::{NonRobust, Robust},
    mutability::{Exclusive, WithInterior, WithoutInterior},
    niche::{WithNiche, WithoutNiche},
    size::{DynTraitLike, ExternTypeLike, MetaSized, NulTerminated, Sized as SpecSized, SliceLike},
};
use static_assertions::assert_type_eq_all;

type Sum<A, B> = <A as Add<B>>::Output;

macro_rules! check_triple {
    ($a:ty, $b:ty, $c:ty) => {
        assert_type_eq_all!(Sum<$a, $b>, Sum<$b, $a>);
        assert_type_eq_all!(Sum<Sum<$a, $b>, $c>, Sum<$a, Sum<$b, $c>>);
    };
}

// Cartesian product without duplicating each family's truth table.
macro_rules! check_family {
    ([$($all:ty),+]) => { check_family!(@a [$($all),+] ; $($all),+); };
    (@a $all:tt ; $($a:ty),+) => { $(check_family!(@b $a, $all ; $all);)+ };
    (@b $a:ty, $all:tt ; [$($b:ty),+]) => { $(check_family!(@c $a, $b ; $all);)+ };
    (@c $a:ty, $b:ty ; [$($c:ty),+]) => { $(check_triple!($a, $b, $c);)+ };
}

#[test]
fn field_order_and_grouping_preserve_classification() {
    check_family!([Robust, NonRobust]);
    check_family!([Stable, Unstable]);
    check_family!([NoDrop, AutoDrop, CustomDrop<NoDrop>, CustomDrop<AutoDrop>]);
    check_family!([WithoutNiche, WithNiche<Stable>, WithNiche<Unstable>]);
    check_family!([
        Exclusive<WithoutInterior>,
        Exclusive<WithInterior<Robust>>,
        Exclusive<WithInterior<NonRobust>>,
        WithInterior<Robust>,
        WithInterior<NonRobust>
    ]);
    check_family!([SpecSized<Zero>, SpecSized<Gt<Zero>>]);
}

#[test]
fn sized_prefix_preserves_dst_shape() {
    macro_rules! check_tail {
        ($($tail:ty),+) => { $(
            assert_type_eq_all!(Sum<SpecSized<Zero>, $tail>, $tail);
            assert_type_eq_all!(Sum<SpecSized<Gt<Zero>>, $tail>, $tail);
            check_triple!(SpecSized<Zero>, SpecSized<Gt<Zero>>, $tail);
            check_triple!(SpecSized<Gt<Zero>>, SpecSized<Zero>, $tail);
        )+ };
    }
    check_tail!(
        MetaSized<SliceLike>,
        MetaSized<DynTraitLike>,
        ExternTypeLike,
        NulTerminated
    );
}
