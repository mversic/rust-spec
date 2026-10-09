//! Drop behavior classification.
//!
//! Drop behavior specifies whether dropping a type runs custom drop code, automatic drop glue,
//! or both.
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
    type Output = Self;

    fn add(self, _: NoDrop) -> Self::Output {
        unreachable!()
    }
}

impl Add<AutoDrop> for CustomDrop<NoDrop> {
    type Output = CustomDrop<AutoDrop>;

    fn add(self, _: AutoDrop) -> Self::Output {
        unreachable!()
    }
}

impl<I: DropKind> Add<CustomDrop<I>> for CustomDrop<NoDrop> {
    type Output = CustomDrop<AutoDrop>;

    fn add(self, _: CustomDrop<I>) -> Self::Output {
        unreachable!()
    }
}

impl<Rhs> Add<Rhs> for CustomDrop<AutoDrop> {
    type Output = Self;

    fn add(self, _: Rhs) -> Self::Output {
        unreachable!()
    }
}
