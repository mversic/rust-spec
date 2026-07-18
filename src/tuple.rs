use core::ops::Add;

use crate::{
    mutability::MutabilityFamily,
    niche::NicheFamily,
    repr::{ReprFamily, Unstable},
    size::SizeFamily,
};

macro_rules! impl_tuple_repr_family {
    (@split [$($head:ident,)*] $last:ident) => {
        impl<$($head,)* $last: ?Sized> ReprFamily for ($($head,)* $last,) {
            type Kind = Unstable;
        }
    };

    (@split [$($head:ident,)*] $next:ident, $($tail:ident),+) => {
        impl_tuple_repr_family!(@split [$($head,)* $next,] $($tail),+);
    };

    ($(($($ty:ident),+)),+ $(,)?) => { $(
        impl_tuple_repr_family!(@split [] $($ty),+); )+
    };
}

macro_rules! impl_tuple_families {
    (@kind $family:ident; $ty:ident) => {
        <$ty as $family>::Kind
    };
    (@kind $family:ident; $head:ident, $($tail:ident),+) => {
        <<$head as $family>::Kind as Add<impl_tuple_families!(@kind $family; $($tail),+)>>::Output
    };

    (@impl SizeFamily for $target:ty; $($all:ident),+) => {
        impl_tuple_families!(@params unsafe SizeFamily for $target [$($all),+] []; $($all),+);
    };
    (@impl $family:ident for $target:ty; $($all:ident),+) => {
        impl_tuple_families!(@params safe $family for $target [$($all),+] []; $($all),+);
    };

    (@params unsafe SizeFamily for $target:ty [$($all:ident),+] [$($params:tt)*]; $ty:ident) => {
        unsafe impl<$($params)* $ty: SizeFamily + ?Sized> SizeFamily for $target {
            type Kind = impl_tuple_families!(@kind SizeFamily; $($all),+);
        }
    };
    (@params safe $family:ident for $target:ty [$($all:ident),+] [$($params:tt)*]; $ty:ident) => {
        impl<$($params)* $ty: $family + ?Sized> $family for $target {
            type Kind = impl_tuple_families!(@kind $family; $($all),+);
        }
    };
    (@params $safety:ident $family:ident for $target:ty [$($all:ident),+] [$($params:tt)*]; $head:ident, $($tail:ident),+) => {
        impl_tuple_families!(
            @params $safety $family for $target
            [$($all),+]
            [$($params)* $head: $family<Kind: Add<impl_tuple_families!(@kind $family; $($tail),+)>>,]
            ; $($tail),+
        );
    };

    ($(($($ty:ident),+)),+ $(,)?) => { $(
        impl_tuple_families!(@impl MutabilityFamily for ($($ty,)+); $($ty),+);
        impl_tuple_families!(@impl NicheFamily for ($($ty,)+); $($ty),+);
        impl_tuple_families!(@impl SizeFamily for ($($ty,)+); $($ty),+); )+
    };
}

impl_tuple_families! {
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

impl_tuple_repr_family! {
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
        rust_spec::{
            niche::WithNiche,
            size::{NonZst, Sized as Co3Sized, Zst},
        },
    };

    #[test]
    fn tuple_size_family_tracks_zst_fields() {
        assert_impl_all!(((), ()): SizeFamily<Kind = Co3Sized<Zst>>);
        assert_impl_all!(((), (), ()): SizeFamily<Kind = Co3Sized<Zst>>);
    }

    #[test]
    fn tuple_size_family_tracks_non_zst_fields() {
        assert_impl_all!(((), u8): SizeFamily<Kind = rust_spec::size::Sized<NonZst>>);
        assert_impl_all!((u8, ()): SizeFamily<Kind = rust_spec::size::Sized<NonZst>>);
        assert_impl_all!(((), u8, ()): SizeFamily<Kind = rust_spec::size::Sized<NonZst>>);
    }

    #[test]
    fn stored_tuple_3_without_niche() {
        assert_impl_all!((u8, u8, u8):
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithoutNiche>,
        );

        assert_impl_all!(&(u8, u8, u8):
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut (u8, u8, u8):
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<(u8, u8, u8)>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&[(u8, u8, u8)]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [(u8, u8, u8)]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[(u8, u8, u8)]>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<(u8, u8, u8)>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([(u8, u8, u8); 2]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithoutNiche>,
        );
        assert_impl_all!(Option<(u8, u8, u8)>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
    }

    #[test]
    fn stored_tuple_3_with_niche() {
        assert_impl_all!((u8, NonZero<u8>, bool):
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );

        assert_impl_all!(&(u8, NonZero<u8>, bool):
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut (u8, NonZero<u8>, bool):
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<(u8, NonZero<u8>, bool)>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&[(u8, NonZero<u8>, bool)]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [(u8, NonZero<u8>, bool)]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[(u8, NonZero<u8>, bool)]>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<(u8, NonZero<u8>, bool)>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([(u8, NonZero<u8>, bool); 2]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(Option<(u8, NonZero<u8>, bool)>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            // TODO: Depends on: https://github.com/mversic/co3/issues/33
            //NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
    }
}
