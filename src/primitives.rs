//! Logic related to the conversion of primitives to and from FFI-compatible representation

use crate::{
    TypeSpec,
    mutability::Exclusive,
    niche::{WithNiche, WithoutNiche},
    repr::{NonRobust, Robust, Stable},
    size::{MetaSized, SliceLike},
};

macro_rules! primitive_derive {
    ( $primitive:ty ) => {
        unsafe impl TypeSpec for $primitive {
            type Repr = Stable<Robust>;
            type Size = crate::size::Sized<crate::size::NonZst>;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
        }
    };
}

macro_rules! raw_pointer_derive {
    ( $mutability:tt ) => {
        unsafe impl<R: TypeSpec + ?Sized> TypeSpec for *$mutability R {
            type Repr = R::Repr;
            type Size = crate::size::Sized<crate::size::NonZst>;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
        }
    };
}

// FIXME:
macro_rules! impl_fn_types {
    ( $( ( $( $arg:ident ),* ) ),* $(,)? ) => {};
}

macro_rules! fieldless_enum_derive {
    ( $src:ty => $dst:ty: {$niche_val:expr}: $validity_fn:expr ) => {
        unsafe impl TypeSpec for $src {
            type Repr = Stable<NonRobust>;
            type Size = crate::size::Sized<crate::size::NonZst>;
            type Niche = WithNiche<crate::niche::Custom>;
            type Mutability = Exclusive;
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

unsafe impl<R> TypeSpec for [R]
where
    R: TypeSpec,
    WithoutNiche: core::ops::Add<R::Niche>,
{
    type Repr = R::Repr;
    type Size = MetaSized<SliceLike>;
    type Niche = <WithoutNiche as core::ops::Add<R::Niche>>::Output;
    type Mutability = R::Mutability;
}

unsafe impl<R, const N: usize> TypeSpec for [R; N]
where
    R: TypeSpec,
    WithoutNiche: core::ops::Add<R::Niche>,
{
    type Repr = R::Repr;
    type Size = R::Size;
    type Niche = <WithoutNiche as core::ops::Add<R::Niche>>::Output;
    type Mutability = R::Mutability;
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

fieldless_enum_derive! {
    char => u32: {0x110000}:
    |i: &u32| char::from_u32(*i).is_some()
}
fieldless_enum_derive! {
    bool => u8: {2}:
    |i: &u8| *i == 0 || *i == 1
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, vec::Vec};
    use static_assertions::assert_impl_all;

    use super::*;
    use crate::{
        niche::WithNiche,
        repr::{NonRobust, Robust, Unstable},
    };

    #[test]
    fn robust_u8() {
        assert_impl_all!(u8:
            TypeSpec<Repr = Stable<Robust>>,
            TypeSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(&u8:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut u8:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<u8>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[u8]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&mut [u8]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[u8]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<u8>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([u8; 2]:
            TypeSpec<Repr = Stable<Robust>>,
            TypeSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(Option<u8>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }
}
