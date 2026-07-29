//! Logic related to the conversion of [`Option<T>`] to and from FFI-compatible representation

use core::{convert::Infallible, ops::Add};

use crate::{Stable, Unstable};

#[sealed::sealed]
pub trait NicheStabilityKind {}

/// Marker for a type that has no trap representations and therefore no niche value
pub enum WithoutNiche {}

/// Marker for a type that has a niche value.
pub struct WithNiche<K: NicheStabilityKind>(core::marker::PhantomData<K>, Infallible);

#[sealed::sealed]
impl NicheStabilityKind for Stable {}

#[sealed::sealed]
impl NicheStabilityKind for Unstable {}

impl Add for WithoutNiche {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}
impl<K: NicheStabilityKind> Add<WithNiche<K>> for WithoutNiche {
    type Output = WithNiche<Unstable>;

    fn add(self, _: WithNiche<K>) -> Self::Output {
        unreachable!()
    }
}
impl<K: NicheStabilityKind> Add<WithoutNiche> for WithNiche<K> {
    type Output = WithNiche<Unstable>;

    fn add(self, _: WithoutNiche) -> Self::Output {
        unreachable!()
    }
}
impl<K: NicheStabilityKind, U: NicheStabilityKind> Add<WithNiche<U>> for WithNiche<K> {
    type Output = WithNiche<Unstable>;

    fn add(self, _: WithNiche<U>) -> Self::Output {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZero;

    use static_assertions::assert_impl_all;

    use super::*;
    use crate::{
        RustSpec, Unstable,
        layout::{NonRobust, Robust},
        mutability::Exclusive,
        size::NonZst,
    };

    #[test]
    fn nested_option_niche_family() {
        assert_impl_all!(Option<bool>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = crate::size::Sized<NonZst>,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(Option<Option<bool>>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = crate::size::Sized<NonZst>,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(Option<(u8, NonZero<u8>)>:
            RustSpec<
                Layout = Unstable,
                Trap = NonRobust,
                Size = crate::size::Sized<NonZst>,
                // FIXME: The type should be WithoutNiche
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }
}
