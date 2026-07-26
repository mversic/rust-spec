use core::{cell::UnsafeCell, mem::ManuallyDrop};

use rust_spec::{
    RustSpec,
    layout::{NonRobust, Robust, Stable, Unstable},
    mutability::Exclusive,
    niche::WithoutNiche,
    size::{NonZst, Sized as SpecSized},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
pub union RustUnion {
    pub byte: u8,
    _flag: bool,
}

#[derive(RustSpec)]
#[repr(C)]
union ReprCUnion {
    cell: ManuallyDrop<UnsafeCell<u8>>,
    value: u8,
}

#[test]
fn union_classification() {
    assert_impl_all!(RustUnion:
        RustSpec<
            Layout = Unstable<NonRobust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(ReprCUnion:
        RustSpec<
            Layout = Stable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
}
