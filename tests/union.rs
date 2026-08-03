use core::{cell::UnsafeCell, mem::ManuallyDrop};

use rust_spec::{
    RustSpec, Stable, Unstable,
    layout::Robust,
    mutability::Exclusive,
    niche::WithoutNiche,
    size::{Gt, Sized as SpecSized, Zero},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
pub union RustUnion {
    pub byte: u8,
    _flag: bool,
}

#[derive(RustSpec)]
#[repr(C)]
union ReprCStableUnion {
    byte: u8,
    word: u16,
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
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCUnion:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCStableUnion:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::Gt<rust_spec::One>,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
}
