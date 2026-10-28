//! Logic related to the conversion of [`Option<T>`] to and from FFI-compatible representation

use core::{convert::Infallible, ops::Add};

use crate::{
    Stable, Unstable,
    layout::{LayoutKind, NonRobust, Robust},
};

/// Marker for a type with no classified niche value.
pub enum WithoutNiche {}

/// Marker for a type that has a niche value.
// TODO: Removed the bound `: Layout` because the API will be modified
pub struct WithNiche<K>(core::marker::PhantomData<K>, Infallible);

/// Representation contributions when adding an empty alternative to a payload.
///
/// Add `Layout` to the payload layout and `Trap` to its indirect trap
/// classification. `Niche` describes the resulting nullable representation.
#[sealed::sealed]
pub trait NullableRepresentation {
    type Layout: LayoutKind;
    type Trap;
    type Niche;
}

#[sealed::sealed]
impl NullableRepresentation for WithoutNiche {
    type Layout = Unstable;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
}

#[sealed::sealed]
impl NullableRepresentation for WithNiche<Unstable> {
    // Without a count of available niches, consuming one cannot establish
    // robustness or whether any guaranteed niche remains.
    type Layout = Unstable;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
}

#[sealed::sealed]
impl NullableRepresentation for WithNiche<Stable> {
    type Layout = Stable;
    type Trap = Robust;
    type Niche = WithoutNiche;
}

impl Add for WithoutNiche {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}
impl<K: LayoutKind> Add<WithNiche<K>> for WithoutNiche {
    type Output = WithNiche<Unstable>;

    fn add(self, _: WithNiche<K>) -> Self::Output {
        unreachable!()
    }
}
impl<K: LayoutKind> Add<WithoutNiche> for WithNiche<K> {
    type Output = WithNiche<Unstable>;

    fn add(self, _: WithoutNiche) -> Self::Output {
        unreachable!()
    }
}
impl<K: LayoutKind, U: LayoutKind> Add<WithNiche<U>> for WithNiche<K> {
    type Output = WithNiche<Unstable>;

    fn add(self, _: WithNiche<U>) -> Self::Output {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZero as StdNonZero;

    use static_assertions::assert_impl_all;

    use super::*;
    use crate::{
        RustSpec,
        mutability::{Exclusive, WithoutInterior},
    };

    #[test]
    fn nested_option_niche_family() {
        assert_impl_all!(Option<bool>:
            RustSpec<
                Layout = Unstable,
                Size = crate::size::Sized<crate::Gt<crate::Zero>>,
                __Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(Option<Option<bool>>:
            RustSpec<
                Layout = Unstable,
                Size = crate::size::Sized<crate::Gt<crate::Zero>>,
                __Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(Option<(u8, StdNonZero<u8>)>:
            RustSpec<
                Layout = Unstable,
                Size = crate::size::Sized<crate::Gt<crate::Zero>>,
                __Alignment = crate::One,
                Trap = NonRobust,
                // FIXME: The type should be WithoutNiche
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
    }
}
