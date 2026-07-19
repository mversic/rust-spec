use core::ops::Add;

use crate::{
    TypeSpec,
    repr::{Robust, Unstable},
};

macro_rules! impl_tuple_type_spec {
    (@kind $axis:ident; $ty:ident) => {
        <$ty as TypeSpec>::$axis
    };
    (@kind $axis:ident; $head:ident, $($tail:ident),+) => {
        <<$head as TypeSpec>::$axis as Add<impl_tuple_type_spec!(@kind $axis; $($tail),+)>>::Output
    };

    (@params for $target:ty [$($all:ident),+] [$($params:tt)*]; $ty:ident) => {
        unsafe impl<$($params)* $ty: TypeSpec + ?Sized> TypeSpec for $target
        where
            Unstable<Robust>: Add<impl_tuple_type_spec!(@kind Repr; $($all),+)>,
        {
            type Repr = <Unstable<Robust> as Add<impl_tuple_type_spec!(@kind Repr; $($all),+)>>::Output;
            type Size = impl_tuple_type_spec!(@kind Size; $($all),+);
            type Niche = impl_tuple_type_spec!(@kind Niche; $($all),+);
            type Mutability = impl_tuple_type_spec!(@kind Mutability; $($all),+);
        }
    };
    (@params for $target:ty [$($all:ident),+] [$($params:tt)*]; $head:ident, $($tail:ident),+) => {
        impl_tuple_type_spec!(
            @params for $target
            [$($all),+]
            [$($params)*
                $head: TypeSpec<
                    Repr: Add<impl_tuple_type_spec!(@kind Repr; $($tail),+)>,
                    Size: Add<impl_tuple_type_spec!(@kind Size; $($tail),+)>,
                    Niche: Add<impl_tuple_type_spec!(@kind Niche; $($tail),+)>,
                    Mutability: Add<impl_tuple_type_spec!(@kind Mutability; $($tail),+)>,
                >,
            ]
            ; $($tail),+
        );
    };

    ($(($($ty:ident),+)),+ $(,)?) => { $(
        impl_tuple_type_spec!(@params for ($($ty,)+) [$($ty),+] []; $($ty),+); )+
    };
}

impl_tuple_type_spec! {
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
    (A, B, C, D, E, F, G, H, I, J, K, L)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, vec::Vec};
    use core::num::NonZero;

    use static_assertions::assert_impl_all;

    use super::*;
    use crate::{
        niche::WithoutNiche,
        repr::NonRobust,
        rust_spec::{
            niche::WithNiche,
            size::{NonZst, Sized as Co3Sized, Zst},
        },
    };

    #[test]
    fn tuple_size_family_tracks_zst_fields() {
        assert_impl_all!(((), ()): TypeSpec<Size = Co3Sized<Zst>>);
        assert_impl_all!(((), (), ()): TypeSpec<Size = Co3Sized<Zst>>);
    }

    #[test]
    fn tuple_size_family_tracks_non_zst_fields() {
        assert_impl_all!(((), u8): TypeSpec<Size = rust_spec::size::Sized<NonZst>>);
        assert_impl_all!((u8, ()): TypeSpec<Size = rust_spec::size::Sized<NonZst>>);
        assert_impl_all!(((), u8, ()): TypeSpec<Size = rust_spec::size::Sized<NonZst>>);
    }

    #[test]
    fn stored_tuple_3_without_niche() {
        assert_impl_all!((u8, u8, u8):
            TypeSpec<Repr = Unstable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );

        assert_impl_all!(&(u8, u8, u8):
            TypeSpec<Repr = Unstable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut (u8, u8, u8):
            TypeSpec<Repr = Unstable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<(u8, u8, u8)>:
            TypeSpec<Repr = Unstable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&[(u8, u8, u8)]:
            TypeSpec<Repr = Unstable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [(u8, u8, u8)]:
            TypeSpec<Repr = Unstable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[(u8, u8, u8)]>:
            TypeSpec<Repr = Unstable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<(u8, u8, u8)>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([(u8, u8, u8); 2]:
            TypeSpec<Repr = Unstable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(Option<(u8, u8, u8)>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }

    #[test]
    fn stored_tuple_3_with_niche() {
        assert_impl_all!((u8, NonZero<u8>, bool):
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );

        assert_impl_all!(&(u8, NonZero<u8>, bool):
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut (u8, NonZero<u8>, bool):
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<(u8, NonZero<u8>, bool)>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&[(u8, NonZero<u8>, bool)]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [(u8, NonZero<u8>, bool)]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[(u8, NonZero<u8>, bool)]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<(u8, NonZero<u8>, bool)>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([(u8, NonZero<u8>, bool); 2]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(Option<(u8, NonZero<u8>, bool)>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            // TODO: Depends on: https://github.com/mversic/co3/issues/33
            //TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }
}
