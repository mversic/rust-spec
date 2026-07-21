//! Logic related to the conversion of [`Option<T>`] to and from FFI-compatible representation

use core::{convert::Infallible, ops::Add};

/// Marker for a type that has no trap representations and therefore no niche value
pub enum WithoutNiche {}

/// Marker for a type that has a niche value.
pub struct WithNiche<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for a single stable (compiler guaranteed) niche value (e.g. `&u32`).
///
/// Only a handful of Rust types have a stable niche.
pub enum Stable {}

/// Marker for a niche that exists but is not compiler-guaranteed.
pub enum Unstable {}

impl Add for WithoutNiche {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}
impl<K> Add<WithNiche<K>> for WithoutNiche {
    type Output = WithNiche<Unstable>;

    fn add(self, _: WithNiche<K>) -> Self::Output {
        unreachable!()
    }
}
impl<K> Add<WithoutNiche> for WithNiche<K> {
    type Output = Self;

    fn add(self, _: WithoutNiche) -> Self::Output {
        unreachable!()
    }
}
impl<K, U> Add<WithNiche<U>> for WithNiche<K> {
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
        RustSpec, layout::NonRobust, layout::Unstable, mutability::Exclusive, size::NonZst,
    };

    #[test]
    fn nested_option_niche_family() {
        assert_impl_all!(Option<bool>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = crate::size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Unstable>,
                Mutability = Exclusive,
            >,
        );

        assert_impl_all!(Option<Option<bool>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = crate::size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Unstable>,
                Mutability = Exclusive,
            >,
        );

        assert_impl_all!(Option<(u8, NonZero<u8>)>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = crate::size::Sized<NonZst>,
                Niche = WithNiche<crate::niche::Unstable>,
                Mutability = Exclusive,
            >,
            // TODO: Depends on: https://github.com/mversic/co3/issues/33
            //RustSpec<Niche = WithoutNiche>,
            //Niche<CType = ReprCTuple2<u8, u8>>,
        );
    }
}
