//! Mutability classification of Rust types.
use core::{convert::Infallible, marker::PhantomData, ops::Add};

/// No reachable interior-mutable storage.
pub enum WithoutInterior {}

/// Closed family of reachable-interior classifications accepted by [`Exclusive`].
#[sealed::sealed]
pub trait ExclusiveKind {}

#[sealed::sealed]
impl ExclusiveKind for WithoutInterior {}

#[sealed::sealed]
impl<R> ExclusiveKind for WithInterior<R> {}

/// Immediate storage requires exclusive access to mutate. `I` describes whether
/// interior-mutable storage is reachable through indirection.
pub struct Exclusive<I: ExclusiveKind>(PhantomData<I>, Infallible);

/// Storage mutable through shared access. `R` aggregates the robustness of all
/// interior-mutable parts, including those reachable through indirection,
/// independently of the whole value's `RustSpec::Trap` classification.
pub struct WithInterior<R>(PhantomData<R>, Infallible);

/// Convert a pointee classification to the classification of its pointer.
pub trait Indirect {
    type Output;
}
impl<I: ExclusiveKind> Indirect for Exclusive<I> {
    type Output = Self;
}
impl<R> Indirect for WithInterior<R> {
    type Output = Exclusive<Self>;
}

impl Add<Self> for WithoutInterior {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}
impl<R> Add<WithInterior<R>> for WithoutInterior {
    type Output = WithInterior<R>;

    fn add(self, _: WithInterior<R>) -> Self::Output {
        unreachable!()
    }
}
impl<R> Add<WithoutInterior> for WithInterior<R> {
    type Output = Self;

    fn add(self, _: WithoutInterior) -> Self::Output {
        unreachable!()
    }
}
impl<L: Add<R>, R> Add<WithInterior<R>> for WithInterior<L> {
    type Output = WithInterior<L::Output>;

    fn add(self, _: WithInterior<R>) -> Self::Output {
        unreachable!()
    }
}
impl<L: ExclusiveKind + Add<R, Output: ExclusiveKind>, R: ExclusiveKind> Add<Exclusive<R>>
    for Exclusive<L>
{
    type Output = Exclusive<L::Output>;

    fn add(self, _: Exclusive<R>) -> Self::Output {
        unreachable!()
    }
}
impl<I: ExclusiveKind + Add<WithInterior<R>>, R> Add<WithInterior<R>> for Exclusive<I> {
    type Output = <I as Add<WithInterior<R>>>::Output;

    fn add(self, _: WithInterior<R>) -> Self::Output {
        unreachable!()
    }
}
impl<I: ExclusiveKind, R> Add<Exclusive<I>> for WithInterior<R>
where
    Self: Add<I>,
{
    type Output = <Self as Add<I>>::Output;

    fn add(self, _: Exclusive<I>) -> Self::Output {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, vec::Vec};

    use static_assertions::assert_impl_all;

    use crate::{
        RustSpec, Stable, Unstable,
        layout::{NonRobust, Robust},
        mutability::{Exclusive, WithInterior, WithoutInterior},
        niche::{WithNiche, WithoutNiche},
        size::Sized as RustSpecSized,
    };

    #[test]
    fn exclusive_kind_accepts_only_reachability_markers() {
        use super::{ExclusiveKind, WithoutInterior};
        assert_impl_all!(WithoutInterior: ExclusiveKind);
        assert_impl_all!(WithInterior<Robust>: ExclusiveKind);
        assert_impl_all!(WithInterior<NonRobust>: ExclusiveKind);
        static_assertions::assert_not_impl_any!(Exclusive<WithoutInterior>: ExclusiveKind);
        static_assertions::assert_not_impl_any!(u8: ExclusiveKind);
    }

    #[test]
    fn indirect_and_aggregate_mutability() {
        use core::cell::UnsafeCell;
        use core::ptr::NonNull;
        assert_impl_all!(&&UnsafeCell<u8>: RustSpec<Mutability = Exclusive<WithInterior<Robust>>>);
        assert_impl_all!(*const UnsafeCell<u8>: RustSpec<Mutability = Exclusive<WithInterior<Robust>>>);
        assert_impl_all!(NonNull<UnsafeCell<u8>>: RustSpec<Mutability = Exclusive<WithInterior<Robust>>>);
        assert_impl_all!((u8, &UnsafeCell<u8>): RustSpec<Mutability = Exclusive<WithInterior<Robust>>>);
        assert_impl_all!((UnsafeCell<u8>, &UnsafeCell<u8>): RustSpec<Mutability = WithInterior<Robust>>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<Box<UnsafeCell<u8>>>: RustSpec<Mutability = Exclusive<WithInterior<Robust>>>);
    }

    #[test]
    fn mixed_mutability_preserves_reachable_robustness() {
        use core::cell::UnsafeCell;

        type Mixed = (UnsafeCell<u8>, &'static UnsafeCell<bool>);
        type Reversed = (&'static UnsafeCell<bool>, UnsafeCell<u8>);
        assert_impl_all!(Mixed: RustSpec<Mutability = WithInterior<NonRobust>>);
        assert_impl_all!(Reversed: RustSpec<Mutability = WithInterior<NonRobust>>);
        assert_impl_all!(&&Mixed: RustSpec<Mutability = Exclusive<WithInterior<NonRobust>>>);
        assert_impl_all!((u8, &Mixed): RustSpec<Mutability = Exclusive<WithInterior<NonRobust>>>);
        assert_impl_all!((UnsafeCell<u8>, &Mixed): RustSpec<Mutability = WithInterior<NonRobust>>);
        assert_impl_all!((&Mixed, UnsafeCell<u8>): RustSpec<Mutability = WithInterior<NonRobust>>);
        assert_impl_all!((bool, UnsafeCell<u8>): RustSpec<Trap = NonRobust, Mutability = WithInterior<Robust>>);
    }

    #[test]
    fn mutability_family_tracks_shared_mutability() {
        use core::{
            cell::{Cell, UnsafeCell},
            ptr::NonNull,
        };

        assert_impl_all!(u8:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior<Robust>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Cell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior<Robust>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!((UnsafeCell<u8>, UnsafeCell<u8>):
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior<Robust>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!((UnsafeCell<u8>, u8):
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior<Robust>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([UnsafeCell<u8>; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior<Robust>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithInterior<Robust>>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithInterior<Robust>>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(*const UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithInterior<Robust>>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(*mut UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithInterior<Robust>>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(NonNull<UnsafeCell<u8>>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithInterior<Robust>>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<UnsafeCell<u8>>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = WithInterior<Robust>,
                __IndirectTrap = Robust,
            >,
        );

        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithInterior<Robust>>,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithInterior<Robust>>,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithInterior<Robust>>,
                __IndirectTrap = Robust,
            >,
        );
    }
}
