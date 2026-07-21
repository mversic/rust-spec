use core::{cell::UnsafeCell, num::NonZero};

use rust_spec::{
    RustSpec,
    layout::{NonRobust, Robust, Stable, Unstable},
    mutability::{Exclusive, Interior},
    niche::{Stable as StableNiche, Unstable as UnstableNiche, WithNiche, WithoutNiche},
    size::{self, NonZst},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
struct Wrapper<T> {
    value: T,
}

#[repr(C)]
#[derive(RustSpec)]
struct Pair<T, U> {
    first: T,
    second: U,
}

fn main() {
    assert_impl_all!(Wrapper<u8>:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = size::Sized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >
    );
    assert_impl_all!(Wrapper<UnsafeCell<u8>>:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = size::Sized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Interior,
        >
    );
    assert_impl_all!(Pair<u8, NonZero<u8>>:
        RustSpec<
            Layout = Stable<NonRobust>,
            Size = size::Sized<NonZst>,
            Niche = WithNiche<StableNiche>,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(Pair<bool, u8>:
        RustSpec<
            Layout = Stable<NonRobust>,
            Size = size::Sized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
        >,
    );
}
