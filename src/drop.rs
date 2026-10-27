//! Drop behavior classification.
//!
//! Drop behavior specifies whether dropping a type runs custom drop cod3, automatic drop glue, or neither.
//!
//! In particular, if a type is [`CustomDrop<NoDrop>>`], it is guaranteed that no inner field of the type
//! will run any drop glue. This can be used inside a custom drop implementation to safely take the value
//! out of a mutable reference in a generic context without a risk of a double free.
use core::{convert::Infallible, ops::Add};

/// Dropping this type does not run drop glue.
pub enum NoDrop {}

/// Owned contents need automatic drop glue.
pub enum AutoDrop {}

/// The type has a custom [`Drop`] implementation, with the given drop behavior for its contents.
pub struct CustomDrop<I: DropKind>(core::marker::PhantomData<I>, Infallible);

#[sealed::sealed]
pub trait DropKind {}

#[sealed::sealed]
impl DropKind for NoDrop {}
#[sealed::sealed]
impl DropKind for AutoDrop {}
#[sealed::sealed]
impl DropKind for CustomDrop<NoDrop> {}
#[sealed::sealed]
impl DropKind for CustomDrop<AutoDrop> {}

impl Add<Self> for NoDrop {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl Add<AutoDrop> for NoDrop {
    type Output = AutoDrop;

    fn add(self, _: AutoDrop) -> Self::Output {
        unreachable!()
    }
}

impl<I: DropKind> Add<CustomDrop<I>> for NoDrop {
    type Output = AutoDrop;

    fn add(self, _: CustomDrop<I>) -> Self::Output {
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
