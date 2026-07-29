//! Logic related to the conversion of primitives to and from FFI-compatible representation

use core::{cmp::Ordering, ops::Add};

use crate::{
    RustSpec,
    layout::Stable,
    layout::{NonRobust, Robust},
    mutability::Exclusive,
    niche::{self, WithNiche, WithoutNiche},
    size::{self, MetaSized, NonZst, SliceLike},
};

macro_rules! primitive_derive {
    ( $primitive:ty ) => {
        unsafe impl RustSpec for $primitive {
            type Layout = Stable<Robust>;
            type Size = size::Sized<NonZst>;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
            type __IndirectLayout = Stable<Robust>;
        }
    };
}

macro_rules! raw_pointer_derive {
    ( $mutability:tt ) => {
        unsafe impl<R: ?Sized> RustSpec for *$mutability R {
            type Layout = Stable<Robust>;
            type Size = size::Sized<NonZst>;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
            type __IndirectLayout = Stable<Robust>;

        }
    };
}

macro_rules! impl_fn_types {
    ( $( ( $( $arg:ident ),* ) ),* $(,)? ) => {
        // FIXME:
        //unsafe impl<$($arg,)* R> RustSpec for fn($($arg),*) -> R {
        //    type Layout = Stable<NonRobust>;
        //    type Size = size::Sized<NonZst>;
        //    type Niche = WithNiche<niche::Stable>;
        //    type Mutability = Exclusive;
        //    type __IndirectLayout = Stable<Robust>;
        //}

        //unsafe impl<$($arg,)* R> RustSpec for unsafe fn($($arg),*) -> R {
        //    type Layout = Stable<NonRobust>;
        //    type Size = size::Sized<NonZst>;
        //    type Niche = WithNiche<niche::Stable>;
        //    type Mutability = Exclusive;
        //    type __IndirectLayout = Stable<Robust>;
        //}

        //unsafe impl<$($arg,)* R> RustSpec for extern "C" fn($($arg),*) -> R {
        //    type Layout = Stable<NonRobust>;
        //    type Size = size::Sized<NonZst>;
        //    type Niche = WithNiche<niche::Stable>;
        //    type Mutability = Exclusive;
        //    type __IndirectLayout = Stable<Robust>;
        //}

        //unsafe impl<$($arg,)* R> RustSpec for unsafe extern "C" fn($($arg),*) -> R {
        //    type Layout = Stable<NonRobust>;
        //    type Size = size::Sized<NonZst>;
        //    type Niche = WithNiche<niche::Stable>;
        //    type Mutability = Exclusive;
        //    type __IndirectLayout = Stable<Robust>;
        //}
    };
}

macro_rules! fieldless_enum_derive {
    ( $src:ty ) => {
        unsafe impl RustSpec for $src {
            type Layout = Stable<NonRobust>;
            type Size = size::Sized<NonZst>;
            type Niche = WithNiche<niche::Unstable>;
            type Mutability = Exclusive;
            type __IndirectLayout = Stable<Robust>;
        }
    };
}

primitive_derive! { usize }
primitive_derive! { isize }
primitive_derive! { u8 }
primitive_derive! { i8 }
primitive_derive! { u16 }
primitive_derive! { i16 }
primitive_derive! { u32 }
primitive_derive! { i32 }
primitive_derive! { u64 }
primitive_derive! { i64 }
primitive_derive! { u128 }
primitive_derive! { i128 }
primitive_derive! { f32 }
primitive_derive! { f64 }

raw_pointer_derive! { const }
raw_pointer_derive! { mut }

unsafe impl<R> RustSpec for [R]
where
    R: RustSpec,
    WithoutNiche: Add<R::Niche>,
{
    type Layout = R::Layout;
    type Size = MetaSized<SliceLike>;
    // TODO: This should not be set at all
    type Niche = <WithoutNiche as Add<R::Niche>>::Output;
    type Mutability = Exclusive;
    type __IndirectLayout = R::__IndirectLayout;
}

// FIXME: N == 0 is a special case with a different spec
unsafe impl<R, const N: usize> RustSpec for [R; N]
where
    R: RustSpec,
    WithoutNiche: Add<R::Niche>,
{
    type Layout = R::Layout;
    type Size = R::Size;
    type Niche = <WithoutNiche as Add<R::Niche>>::Output;
    type Mutability = R::Mutability;
    type __IndirectLayout = R::__IndirectLayout;
}

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

fieldless_enum_derive! { char }
fieldless_enum_derive! { bool }
fieldless_enum_derive! { Ordering }

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, vec::Vec};
    use static_assertions::assert_impl_all;

    use super::*;
    use crate::{
        layout::Unstable,
        layout::{NonRobust, Robust},
        niche::WithNiche,
    };

    #[test]
    fn robust_u8() {
        assert_impl_all!(u8:
            RustSpec<
                Layout = Stable<Robust>,
                Size = size::Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&u8:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&mut u8:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<u8>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&[u8]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&mut [u8]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[u8]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<u8>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!([u8; 2]:
            RustSpec<
                Layout = Stable<Robust>,
                Size = size::Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<u8>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
    }
}
