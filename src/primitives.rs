//! Logic related to the conversion of primitives to and from FFI-compatible representation

use core::{cmp::Ordering, ops::Add};

use crate::{
    RustSpec, Stable, Unstable, Zero,
    layout::{NonRobust, Robust},
    mutability::Exclusive,
    niche::{WithNiche, WithoutNiche},
    size::{self, MetaSized, SliceLike},
};

macro_rules! primitive_derive {
    ( $primitive:ty => $alignment:ty ) => {
        unsafe impl RustSpec for $primitive {
            type Layout = Stable;
            type Size = size::Sized<crate::Gt<Zero>>;
            type Alignment = $alignment;
            type Trap = Robust;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    };
}

macro_rules! raw_pointer_derive {
    ( $mutability:tt ) => {
        unsafe impl<R: ?Sized> RustSpec for *$mutability R {
            type Layout = Stable;
            type Size = size::Sized<crate::Gt<Zero>>;
            type Alignment = <usize as RustSpec>::Alignment;
            type Trap = Robust;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;

        }
    };
}

macro_rules! impl_nonempty_array {
    ($($n:literal),* $(,)?) => { $(
        unsafe impl<R: RustSpec> RustSpec for [R; $n]
        where
            WithoutNiche: Add<R::Niche>,
        {
            type Layout = R::Layout;
            type Size = R::Size;
            type Alignment = R::Alignment;
            type Trap = R::Trap;
            type Niche = <WithoutNiche as Add<R::Niche>>::Output;
            type Mutability = R::Mutability;
            type __IndirectTrap = R::__IndirectTrap;
        })*
    };
}

macro_rules! impl_fn_types {
    ( $( ( $( $arg:ident ),* ) ),* $(,)? ) => {
        // FIXME:
        //unsafe impl<$($arg,)* R> RustSpec for fn($($arg),*) -> R {
        //    type Layout = Stable<NonRobust>;
        //    type Size = size::Sized<crate::Gt<Zero>>;
        //    type Niche = WithNiche<Stable>;
        //    type Mutability = Exclusive;
        //    type __IndirectLayout = Stable<Robust>;
        //}

        //unsafe impl<$($arg,)* R> RustSpec for unsafe fn($($arg),*) -> R {
        //    type Layout = Stable<NonRobust>;
        //    type Size = size::Sized<crate::Gt<Zero>>;
        //    type Niche = WithNiche<Stable>;
        //    type Mutability = Exclusive;
        //    type __IndirectLayout = Stable<Robust>;
        //}

        //unsafe impl<$($arg,)* R> RustSpec for extern "C" fn($($arg),*) -> R {
        //    type Layout = Stable<NonRobust>;
        //    type Size = size::Sized<crate::Gt<Zero>>;
        //    type Niche = WithNiche<Stable>;
        //    type Mutability = Exclusive;
        //    type __IndirectLayout = Stable<Robust>;
        //}

        //unsafe impl<$($arg,)* R> RustSpec for unsafe extern "C" fn($($arg),*) -> R {
        //    type Layout = Stable<NonRobust>;
        //    type Size = size::Sized<crate::Gt<Zero>>;
        //    type Niche = WithNiche<Stable>;
        //    type Mutability = Exclusive;
        //    type __IndirectLayout = Stable<Robust>;
        //}
    };
}

macro_rules! fieldless_enum_derive {
    ( $src:ty => $alignment:ty ) => {
        unsafe impl RustSpec for $src {
            type Layout = Stable;
            type Size = size::Sized<crate::Gt<Zero>>;
            type Alignment = $alignment;
            type Trap = NonRobust;
            type Niche = WithNiche<Unstable>;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    };
}

primitive_derive! { u8 => crate::One }
primitive_derive! { i8 => crate::One }
primitive_derive! { u16 => crate::Gt<crate::One> }
primitive_derive! { i16 => crate::Gt<crate::One> }
primitive_derive! { u32 => crate::Gt<crate::One> }
primitive_derive! { i32 => crate::Gt<crate::One> }
primitive_derive! { u64 => crate::Gt<crate::One> }
primitive_derive! { i64 => crate::Gt<crate::One> }
primitive_derive! { u128 => crate::Gt<crate::One> }
primitive_derive! { i128 => crate::Gt<crate::One> }
primitive_derive! { f32 => crate::Gt<crate::One> }
primitive_derive! { f64 => crate::Gt<crate::One> }

#[cfg(target_pointer_width = "16")]
primitive_derive! { usize => crate::Gt<crate::One> }
#[cfg(target_pointer_width = "32")]
primitive_derive! { usize => crate::Gt<crate::One> }
#[cfg(target_pointer_width = "64")]
primitive_derive! { usize => crate::Gt<crate::One> }

primitive_derive! { isize => <usize as RustSpec>::Alignment }

raw_pointer_derive! { const }
raw_pointer_derive! { mut }

unsafe impl<R: RustSpec> RustSpec for [R] {
    type Layout = R::Layout;
    type Size = MetaSized<SliceLike>;
    type Alignment = R::Alignment;
    type Trap = R::Trap;
    // TODO: This should not be set at all
    // however we set it to help some impls
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectTrap = R::__IndirectTrap;
}

unsafe impl<R: RustSpec> RustSpec for [R; 0] {
    type Layout = R::Layout;
    type Size = size::Sized<Zero>;
    type Alignment = R::Alignment;
    type Trap = Robust;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

impl_nonempty_array!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32
);

impl_fn_types! {
    (),
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
}

fieldless_enum_derive! { char => crate::Gt<crate::One> }
fieldless_enum_derive! { bool => crate::One }
fieldless_enum_derive! { Ordering => crate::One }

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, vec::Vec};
    use static_assertions::assert_impl_all;

    use super::*;
    use crate::{
        Unstable,
        layout::{NonRobust, Robust},
        niche::WithNiche,
    };

    #[test]
    fn robust_u8() {
        assert_impl_all!(u8:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&u8:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut u8:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<u8>:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&[u8]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut [u8]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[u8]>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<u8>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<u8>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }
}
