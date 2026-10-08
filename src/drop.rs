//! Drop behavior classification.
//!
//! Drop behavior specifies whether drop glue runs when the type is dropped and what kind, if any.
use core::{convert::Infallible, ops::Add};

/// Dropping this type does not run drop glue.
pub enum NoDrop {}

/// The type owns a field or content that needs drop glue.
pub enum Inner {}

/// This type declares [`Drop`] and has no inner drop requirements.
pub enum Custom {}

/// A type whose drop glue is required, classified by its source.
pub struct WithDrop<K: DropKind>(core::marker::PhantomData<K>, Infallible);

#[sealed::sealed]
pub trait DropKind {}

#[sealed::sealed]
impl DropKind for Inner {}
#[sealed::sealed]
impl DropKind for Custom {}

impl Add<Self> for NoDrop {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl Add<WithDrop<Custom>> for NoDrop {
    type Output = WithDrop<Inner>;

    fn add(self, _: WithDrop<Custom>) -> Self::Output {
        unreachable!()
    }
}

impl Add<WithDrop<Inner>> for NoDrop {
    type Output = WithDrop<Inner>;

    fn add(self, _: WithDrop<Inner>) -> Self::Output {
        unreachable!()
    }
}

impl<Rhs> Add<Rhs> for WithDrop<Inner> {
    type Output = Self;

    fn add(self, _: Rhs) -> Self::Output {
        unreachable!()
    }
}

impl Add<NoDrop> for WithDrop<Custom> {
    type Output = Self;

    fn add(self, _: NoDrop) -> Self::Output {
        unreachable!()
    }
}

impl Add<Self> for WithDrop<Custom> {
    type Output = WithDrop<Inner>;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl Add<WithDrop<Inner>> for WithDrop<Custom> {
    type Output = WithDrop<Inner>;

    fn add(self, _: WithDrop<Inner>) -> Self::Output {
        unreachable!()
    }
}
