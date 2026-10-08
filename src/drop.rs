//! Drop classification markers:
//!
//! - [`NoDrop`]: dropping the type runs no drop glue.
//! - [`CustomDrop`]: the type implements [`core::ops::Drop`] without inner drop requirements.
//! - [`InnerDrop`]: a field or other owned content has drop behavior; the outer type may also implement `Drop`.

use core::ops::Add;

/// Dropping this type does not run drop glue.
pub enum NoDrop {}

/// Dropping this type runs drop glue for a field or owned content at any depth.
pub enum InnerDrop {}

/// This type declares `core::ops::Drop` and has no inner drop requirements.
pub enum CustomDrop {}

/// Closed family of drop classifications.
#[sealed::sealed]
pub trait DropKind: Sized {}

#[sealed::sealed]
impl DropKind for NoDrop {}
#[sealed::sealed]
impl DropKind for InnerDrop {}
#[sealed::sealed]
impl DropKind for CustomDrop {}

impl Add<Self> for NoDrop {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl Add<InnerDrop> for NoDrop {
    type Output = InnerDrop;

    fn add(self, _: InnerDrop) -> Self::Output {
        unreachable!()
    }
}

impl<Rhs: DropKind> Add<Rhs> for InnerDrop {
    type Output = Self;

    fn add(self, _: Rhs) -> Self::Output {
        unreachable!()
    }
}

impl Add<CustomDrop> for NoDrop {
    type Output = InnerDrop;

    fn add(self, _: CustomDrop) -> Self::Output {
        unreachable!()
    }
}

impl Add<NoDrop> for CustomDrop {
    type Output = Self;

    fn add(self, _: NoDrop) -> Self::Output {
        unreachable!()
    }
}

impl Add<InnerDrop> for CustomDrop {
    type Output = InnerDrop;

    fn add(self, _: InnerDrop) -> Self::Output {
        unreachable!()
    }
}

impl Add<Self> for CustomDrop {
    type Output = InnerDrop;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}
