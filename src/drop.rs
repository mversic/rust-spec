//! Drop behavior classification.
//!
//! Drop behavior specifies whether dropping a type runs custom drop cod3, automatic drop glue, or neither.
//!
//! In particular, if a type is [`CustomDrop<NoDrop>>`], it is guaranteed that no inner field of the type
//! will run any drop glue. This can be used inside a custom drop implementation to safely take the value
//! out of a mutable reference in a generic context without a risk of a double free.
use core::{convert::Infallible, ops::Add};

#[sealed::sealed]
pub trait DropKind {
    /// Drop glue required by a container that owns a field of this kind.
    /// A field's custom destructor is automatic drop glue for its container.
    type NeedsDrop: DropKind;
}

/// Dropping this type does not run drop glue.
pub enum NoDrop {}

/// Owned contents need automatic drop glue.
pub enum AutoDrop {}

/// The type has a custom [`Drop`] implementation, with the given drop behavior for its contents.
pub struct CustomDrop<I: DropKind>(core::marker::PhantomData<I>, Infallible);

#[sealed::sealed]
impl DropKind for NoDrop {
    type NeedsDrop = NoDrop;
}
#[sealed::sealed]
impl DropKind for AutoDrop {
    type NeedsDrop = AutoDrop;
}
#[sealed::sealed]
impl<K: DropKind> DropKind for CustomDrop<K> {
    type NeedsDrop = AutoDrop;
}

impl<I: DropKind> Add<I> for NoDrop {
    type Output = I::NeedsDrop;

    fn add(self, _: I) -> Self::Output {
        unreachable!()
    }
}

impl<Rhs> Add<Rhs> for AutoDrop {
    type Output = Self;

    fn add(self, _: Rhs) -> Self::Output {
        unreachable!()
    }
}

impl Add<NoDrop> for CustomDrop<NoDrop> {
    type Output = AutoDrop;

    fn add(self, _: NoDrop) -> Self::Output {
        unreachable!()
    }
}

impl Add<AutoDrop> for CustomDrop<NoDrop> {
    type Output = AutoDrop;

    fn add(self, _: AutoDrop) -> Self::Output {
        unreachable!()
    }
}

impl<I: DropKind> Add<CustomDrop<I>> for CustomDrop<NoDrop> {
    type Output = AutoDrop;

    fn add(self, _: CustomDrop<I>) -> Self::Output {
        unreachable!()
    }
}

impl<Rhs> Add<Rhs> for CustomDrop<AutoDrop> {
    type Output = AutoDrop;

    fn add(self, _: Rhs) -> Self::Output {
        unreachable!()
    }
}
