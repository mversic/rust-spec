use core::num::NonZeroU8;

use rust_spec::{
    RustSpec,
    layout::{NonRobust, Robust, Stable, Unstable},
    mutability::Exclusive,
    niche::{Stable as StableNiche, Unstable as UnstableNiche, WithNiche, WithoutNiche},
    size::{NonZst, Sized as SpecSized, Zst},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
pub enum RustEnum {
    A(u8),
    B,
}

#[derive(RustSpec)]
#[repr(u8)]
pub enum PrimitiveEnum {
    A,
    B,
}

#[derive(RustSpec)]
enum EmptyEnum {}

#[derive(RustSpec)]
pub enum SingletonEnum {
    Only,
}

#[derive(RustSpec)]
pub enum SingletonWithoutNicheEnum {
    Value(u8),
}

#[derive(RustSpec)]
pub enum SingletonWithNicheEnum {
    Value(NonZeroU8),
}

#[derive(RustSpec)]
pub enum TwoVariantEnum {
    First,
    Second,
}

#[derive(RustSpec)]
#[repr(u8)]
pub enum TaggedSingletonEnum {
    Only,
}

#[derive(RustSpec)]
#[repr(transparent)]
pub enum TransparentNoNicheEnum {
    Value(u8),
}

#[derive(RustSpec)]
#[repr(transparent)]
pub enum TransparentWithNicheEnum<'a> {
    Value(&'a u8),
}

#[test]
fn enum_classification() {
    assert_impl_all!(EmptyEnum:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<Zst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
    assert_impl_all!(SingletonEnum:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<Zst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
    assert_impl_all!(SingletonWithoutNicheEnum:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
    assert_impl_all!(SingletonWithNicheEnum:
        RustSpec<
            Layout = Unstable<NonRobust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
    assert_impl_all!(TwoVariantEnum:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
    assert_impl_all!(TaggedSingletonEnum:
        RustSpec<
            Layout = Stable<NonRobust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
    assert_impl_all!(TransparentNoNicheEnum:
        RustSpec<
            Layout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
    assert_impl_all!(TransparentWithNicheEnum:
        RustSpec<
            Layout = Stable<NonRobust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<StableNiche>,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );

    assert_impl_all!(RustEnum:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
    assert_impl_all!(PrimitiveEnum:
        RustSpec<
            Layout = Stable<NonRobust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
            __IndirectLayout = Stable<Robust>,
        >,
    );
}
