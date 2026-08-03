use core::{convert::Infallible, ops::Add};

pub use crate::{Gt, Zero};

#[sealed::sealed]
pub trait SizedKind {}

#[sealed::sealed]
pub trait MetadataKind {}

/// Marker for types with a constant size known at compile time.
///
/// See [`core::marker::Sized`].
pub struct Sized<K: SizedKind>(core::marker::PhantomData<K>, Infallible);

/// Marker for types with a size that can be determined from pointer metadata.
///
/// Type parameter can be set to either [`SliceLike`] or [`DynTraitLike`] only.
///
/// See [`core::marker::MetaSized`]
pub struct MetaSized<K: MetadataKind>(core::marker::PhantomData<K>, Infallible);

/// Marker for [Slices](https://doc.rust-lang.org/core/primitive.slice.html) or DSTs whose last field is a slice.
///
/// See [`core::slice`].
pub enum SliceLike {}

/// Marker for [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html) or DSTs whose last field is a trait object.
pub enum DynTraitLike {}

/// Marker for [Extern types](https://rust-lang.github.io/rfcs/1861-extern-types.html) and DSTs whose last field is an extern type.
///
/// Pointers to extern types are thin.
pub enum ExternTypeLike {}

#[sealed::sealed]
impl SizedKind for Zero {}

#[sealed::sealed]
impl SizedKind for crate::Gt<crate::Zero> {}

#[sealed::sealed]
impl MetadataKind for SliceLike {}

#[sealed::sealed]
impl MetadataKind for DynTraitLike {}

/// Pointers to types implementing this trait alias are “thin”.
///
/// See [`core::ptr::Thin`]
pub(crate) trait Thin {}
impl<K: SizedKind> Thin for Sized<K> {}
impl Thin for ExternTypeLike {}

pub(crate) trait Dst {}
impl Dst for ExternTypeLike {}
impl<K: MetadataKind> Dst for MetaSized<K> {}

impl<K: SizedKind> Add<Sized<K>> for Sized<Zero> {
    type Output = Sized<K>;

    fn add(self, _: Sized<K>) -> Self::Output {
        unreachable!()
    }
}

impl<K: SizedKind> Add<Sized<K>> for Sized<crate::Gt<crate::Zero>> {
    type Output = Self;

    fn add(self, _: Sized<K>) -> Self::Output {
        unreachable!()
    }
}

impl<K: Dst, U: SizedKind> Add<K> for Sized<U> {
    type Output = K;

    fn add(self, _: K) -> Self::Output {
        unreachable!()
    }
}

impl<K: MetadataKind, U: SizedKind> Add<Sized<U>> for MetaSized<K> {
    type Output = Self;

    fn add(self, _: Sized<U>) -> Self::Output {
        unreachable!()
    }
}

impl<U: SizedKind> Add<Sized<U>> for ExternTypeLike {
    type Output = Self;

    fn add(self, _: Sized<U>) -> Self::Output {
        unreachable!()
    }
}
