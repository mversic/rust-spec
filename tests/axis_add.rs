use core::ops::Add;

use rust_spec::{
    Stable, Unstable, Zero,
    drop::{AutoDrop, CustomDrop, NoDrop},
    layout::{NonRobust, Robust},
    mutability::{Exclusive, WithInterior},
    niche::{WithNiche, WithoutNiche},
    size::{self, DynTraitLike, ExternTypeLike, NulTerminated, SliceLike},
};

fn assert_add_pair<L, R, Output>()
where
    L: Add<R, Output = Output>,
    R: Add<L, Output = Output>,
{
}

#[test]
fn drop_add_is_commutative() {
    type CustomPlain = CustomDrop<NoDrop>;
    type CustomInner = CustomDrop<AutoDrop>;

    assert_add_pair::<NoDrop, NoDrop, NoDrop>();
    assert_add_pair::<NoDrop, AutoDrop, AutoDrop>();
    assert_add_pair::<NoDrop, CustomPlain, AutoDrop>();
    assert_add_pair::<NoDrop, CustomInner, AutoDrop>();
    assert_add_pair::<AutoDrop, AutoDrop, AutoDrop>();
    assert_add_pair::<AutoDrop, CustomPlain, AutoDrop>();
    assert_add_pair::<AutoDrop, CustomInner, AutoDrop>();
    assert_add_pair::<CustomPlain, CustomPlain, AutoDrop>();
    assert_add_pair::<CustomPlain, CustomInner, AutoDrop>();
    assert_add_pair::<CustomInner, CustomInner, AutoDrop>();
}

#[test]
fn other_axis_add_operations_are_commutative() {
    assert_add_pair::<Stable, Stable, Stable>();
    assert_add_pair::<Stable, Unstable, Unstable>();
    assert_add_pair::<Unstable, Unstable, Unstable>();

    assert_add_pair::<Robust, Robust, Robust>();
    assert_add_pair::<Robust, NonRobust, NonRobust>();
    assert_add_pair::<NonRobust, NonRobust, NonRobust>();

    assert_add_pair::<Exclusive, Exclusive, Exclusive>();
    assert_add_pair::<Exclusive, WithInterior, WithInterior>();
    assert_add_pair::<WithInterior, WithInterior, WithInterior>();

    type StableNiche = WithNiche<Stable>;
    type UnstableNiche = WithNiche<Unstable>;
    assert_add_pair::<WithoutNiche, WithoutNiche, WithoutNiche>();
    assert_add_pair::<WithoutNiche, StableNiche, UnstableNiche>();
    assert_add_pair::<WithoutNiche, UnstableNiche, UnstableNiche>();
    assert_add_pair::<StableNiche, StableNiche, UnstableNiche>();
    assert_add_pair::<StableNiche, UnstableNiche, UnstableNiche>();
    assert_add_pair::<UnstableNiche, UnstableNiche, UnstableNiche>();

    type ZeroSize = size::Sized<Zero>;
    type NonzeroSize = size::Sized<rust_spec::Gt<Zero>>;
    type SliceSize = size::MetaSized<SliceLike>;
    type DynSize = size::MetaSized<DynTraitLike>;
    assert_add_pair::<ZeroSize, ZeroSize, ZeroSize>();
    assert_add_pair::<ZeroSize, NonzeroSize, NonzeroSize>();
    assert_add_pair::<NonzeroSize, NonzeroSize, NonzeroSize>();
    assert_add_pair::<ZeroSize, SliceSize, SliceSize>();
    assert_add_pair::<NonzeroSize, SliceSize, SliceSize>();
    assert_add_pair::<ZeroSize, DynSize, DynSize>();
    assert_add_pair::<NonzeroSize, DynSize, DynSize>();
    assert_add_pair::<ZeroSize, ExternTypeLike, ExternTypeLike>();
    assert_add_pair::<NonzeroSize, ExternTypeLike, ExternTypeLike>();
    assert_add_pair::<ZeroSize, NulTerminated, NulTerminated>();
    assert_add_pair::<NonzeroSize, NulTerminated, NulTerminated>();
}
