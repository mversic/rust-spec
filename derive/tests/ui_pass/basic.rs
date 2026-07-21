use core::cell::UnsafeCell;

use rust_spec::{
    RustSpec,
    layout::{NonRobust, Robust, Stable, Unstable},
    mutability::{Exclusive, Interior},
    niche::{Unstable as UnstableNiche, WithNiche, WithoutNiche},
    size::{NonZst, Sized as SpecSized, Zst},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
struct Empty;

#[derive(RustSpec)]
struct RustStruct {
    value: u8,
}

#[derive(RustSpec)]
struct NonRobustRustRepr {
    value: bool,
}

#[derive(RustSpec)]
#[repr(C)]
struct ReprCStruct {
    value: u8,
}

#[derive(RustSpec)]
struct InteriorWrapper {
    left: UnsafeCell<u8>,
    right: UnsafeCell<u16>,
}

#[derive(RustSpec)]
struct MixedMutability {
    cell: UnsafeCell<u8>,
    value: u8,
}

#[derive(RustSpec)]
enum RustEnum {
    A(u8),
    B,
}

#[derive(RustSpec)]
#[repr(u8)]
enum PrimitiveEnum {
    A,
    B,
}

fn main() {
    assert_impl_all!(Empty:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<Zst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(RustStruct:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(NonRobustRustRepr:
        RustSpec<
            Layout = Unstable<NonRobust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(ReprCStruct:
        RustSpec<
            Layout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(InteriorWrapper:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Interior,
        >,
    );
    assert_impl_all!(MixedMutability:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(RustEnum:
        RustSpec<
            Layout = Unstable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(PrimitiveEnum:
        RustSpec<
            Layout = Stable<NonRobust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
        >,
    );
}
