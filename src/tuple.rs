use core::ops::Add;

use crate::{Max, RustSpec, Unstable, layout::Robust};

macro_rules! impl_tuple_type_spec {
    (@alignment $ty:ident) => {
        <$ty as RustSpec>::Alignment
    };
    (@alignment $head:ident, $($tail:ident),+) => {
        <<$head as RustSpec>::Alignment as Max<impl_tuple_type_spec!(@alignment $($tail),+)>>::Output
    };
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
            type Size = impl_tuple_type_spec!(@kind Size; $($all),+);
            type Alignment = impl_tuple_type_spec!(@alignment $($all),+);
            type Trap = <Robust as Add<impl_tuple_type_spec!(@kind Trap; $($all),+)>>::Output;
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
                    Size: Add<impl_tuple_type_spec!(@kind Size; $($tail),+)>,
                    Alignment: Max<
                        impl_tuple_type_spec!(@alignment $($tail),+),
                    >,
                    Trap: Add<impl_tuple_type_spec!(@kind Trap; $($tail),+)>,
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
    use core::num::NonZero as StdNonZero;

    use static_assertions::assert_impl_all;

    use super::*;
    use crate::{
        Stable,
        layout::NonRobust,
        mutability::Exclusive,
        niche::{WithNiche, WithoutNiche},
        size::{self, Zero},
    };

    #[test]
    fn tuple_size_family_tracks_zst_fields() {
        assert_impl_all!(((), ()):
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<Zero>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(((), (), ()):
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<Zero>,
                Alignment = crate::One,
                Trap = Robust,
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
                Size = crate::size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!((u8, ()):
            RustSpec<
                Layout = Unstable,
                Size = crate::size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(((), u8, ()):
            RustSpec<
                Layout = Unstable,
                Size = crate::size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
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
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(&(u8, u8, u8):
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut (u8, u8, u8):
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&[(u8, u8, u8)]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [(u8, u8, u8)]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[(u8, u8, u8)]>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([(u8, u8, u8); 2]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }

    #[test]
    fn stored_tuple_3_with_niche() {
        assert_impl_all!((u8, StdNonZero<u8>, bool):
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(&(u8, StdNonZero<u8>, bool):
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut (u8, StdNonZero<u8>, bool):
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<(u8, StdNonZero<u8>, bool)>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&[(u8, StdNonZero<u8>, bool)]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [(u8, StdNonZero<u8>, bool)]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[(u8, StdNonZero<u8>, bool)]>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<(u8, StdNonZero<u8>, bool)>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([(u8, StdNonZero<u8>, bool); 2]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<(u8, StdNonZero<u8>, bool)>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }
}
