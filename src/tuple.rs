use core::ops::Add;

use crate::{RustSpec, Unstable, layout::Robust};

macro_rules! impl_tuple_type_spec {
    (@kind $axis:ident; $ty:ident) => {
        <$ty as RustSpec>::$axis
    };
    (@kind $axis:ident; $head:ident, $($tail:ident),+) => {
        <<$head as RustSpec>::$axis as Add<impl_tuple_type_spec!(@kind $axis; $($tail),+)>>::Output
    };

    (@params for $target:ty [$($all:ident),+] [$($params:tt)*]; $ty:ident) => {
        unsafe impl<$($all),+> RustSpec for $target
        where
            $($params)*
            $ty: RustSpec,
            Robust: Add<impl_tuple_type_spec!(@kind Trap; $($all),+)>,
        {
            type Layout = Unstable;
            type Trap = <Robust as Add<impl_tuple_type_spec!(@kind Trap; $($all),+)>>::Output;
            type Size = impl_tuple_type_spec!(@kind Size; $($all),+);
            type Niche = impl_tuple_type_spec!(@kind Niche; $($all),+);
            type Mutability = impl_tuple_type_spec!(@kind Mutability; $($all),+);
            type __IndirectTrap = impl_tuple_type_spec!(@kind __IndirectTrap; $($all),+);
        }
    };
    (@params for $target:ty [$($all:ident),+] [$($params:tt)*]; $head:ident, $($tail:ident),+) => {
        impl_tuple_type_spec!(
            @params for $target
            [$($all),+]
            [$($params)*
                $head: RustSpec<
                    Trap: Add<impl_tuple_type_spec!(@kind Trap; $($tail),+)>,
                    Size: Add<impl_tuple_type_spec!(@kind Size; $($tail),+)>,
                    Niche: Add<impl_tuple_type_spec!(@kind Niche; $($tail),+)>,
                    Mutability: Add<impl_tuple_type_spec!(@kind Mutability; $($tail),+)>,
                    __IndirectTrap: Add<impl_tuple_type_spec!(@kind __IndirectTrap; $($tail),+)>,
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
        Stable,
        layout::NonRobust,
        mutability::Exclusive,
        niche::{WithNiche, WithoutNiche},
        size::{self, NonZst, Zst},
    };

    #[test]
    fn tuple_size_family_tracks_zst_fields() {
        assert_impl_all!(((), ()):
            RustSpec<
                Layout = Unstable,
                Trap = Robust,
                Size = size::Sized<Zst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(((), (), ()):
            RustSpec<
                Layout = Unstable,
                Trap = Robust,
                Size = size::Sized<Zst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }

    #[test]
    fn tuple_size_family_tracks_non_zst_fields() {
        assert_impl_all!(((), u8):
            RustSpec<
                Layout = Unstable,
                Trap = Robust,
                Size = crate::size::Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!((u8, ()):
            RustSpec<
                Layout = Unstable,
                Trap = Robust,
                Size = crate::size::Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(((), u8, ()):
            RustSpec<
                Layout = Unstable,
                Trap = Robust,
                Size = crate::size::Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }

    #[test]
    fn stored_tuple_3_without_niche() {
        assert_impl_all!((u8, u8, u8):
            RustSpec<
                Layout = Unstable,
                Trap = Robust,
                Size = size::Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(&(u8, u8, u8):
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut (u8, u8, u8):
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&[(u8, u8, u8)]:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [(u8, u8, u8)]:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[(u8, u8, u8)]>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([(u8, u8, u8); 2]:
            RustSpec<
                Layout = Unstable,
                Trap = Robust,
                Size = size::Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }

    #[test]
    fn stored_tuple_3_with_niche() {
        assert_impl_all!((u8, NonZero<u8>, bool):
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(&(u8, NonZero<u8>, bool):
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut (u8, NonZero<u8>, bool):
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<(u8, NonZero<u8>, bool)>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&[(u8, NonZero<u8>, bool)]:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [(u8, NonZero<u8>, bool)]:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[(u8, NonZero<u8>, bool)]>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<(u8, NonZero<u8>, bool)>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([(u8, NonZero<u8>, bool); 2]:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<(u8, NonZero<u8>, bool)>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = size::Sized<NonZst>,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }
}
